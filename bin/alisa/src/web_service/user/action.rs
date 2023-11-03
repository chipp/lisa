use alice::{
    ModeFunction, RangeFunction, StateCapability, StateUpdateResult, ToggleFunction,
    UpdateStateCapability, UpdateStateErrorCode, UpdateStateRequest, UpdateStateResponse,
    UpdatedDeviceState,
};
use serde_json::{json, Value};
use tokio::sync::mpsc::UnboundedSender;
use topics::{Device, ElisaAction, ElizabethState};

use std::str::FromStr;

use bytes::Buf;
use hyper::{Body, Request, Response, StatusCode};

use crate::types::{DeviceId, VacuumFanSpeed};
use crate::web_service::auth::validate_autorization;
use crate::{Action, ActionPayload, Result};

pub async fn action(
    request: Request<Body>,
    perform_action: UnboundedSender<ActionPayload>,
) -> Result<Response<Body>> {
    validate_autorization(request, "devices_action", |request| async move {
        let request_id = String::from(std::str::from_utf8(
            request.headers().get("X-Request-Id").unwrap().as_bytes(),
        )?);

        let body = hyper::body::aggregate(request).await?;
        unsafe {
            println!("[action]: {}", std::str::from_utf8_unchecked(body.chunk()));
        }

        let action: UpdateStateRequest = serde_json::from_slice(body.chunk())?;

        let mut response_devices = vec![];

        for request_device in action.payload.devices {
            let DeviceId { room, device } = DeviceId::from_str(request_device.id).unwrap();
            let mut response_capabilities = vec![];

            for state_capability in request_device.capabilities {
                // TODO: handle None
                let (action, value) =
                    prepare_action_for_device(&state_capability, &device).unwrap();

                let result = match perform_action.send(ActionPayload {
                    device,
                    room,
                    action,
                    value,
                }) {
                    Ok(()) => StateUpdateResult::ok(),
                    Err(err) => StateUpdateResult::error(
                        UpdateStateErrorCode::DeviceUnreachable,
                        err.to_string(),
                    ), // TODO: differentiate errors
                };

                response_capabilities.push(prepare_response_capability(&state_capability, result));
            }

            response_devices.push(UpdatedDeviceState::new(
                request_device.id.to_string(),
                response_capabilities,
            ));
        }

        let response = UpdateStateResponse::new(request_id, response_devices);

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&response)?))?)
    })
    .await
}

fn prepare_action_for_device(
    capability: &StateCapability,
    device: &Device,
) -> Option<(Action, Value)> {
    match (capability, device) {
        (StateCapability::OnOff { value }, Device::Recuperator)
        | (StateCapability::OnOff { value }, Device::Thermostat) => {
            Some((ElizabethState::IsEnabled.into(), json!(value)))
        }
        (
            StateCapability::Mode {
                function: ModeFunction::FanSpeed,
                mode,
            },
            Device::Recuperator,
        ) => Some((ElizabethState::FanSpeed.into(), json!(mode))),
        (
            StateCapability::Mode {
                function: ModeFunction::WorkSpeed,
                mode,
            },
            Device::VacuumCleaner,
        ) => Some((
            ElisaAction::SetFanSpeed.into(),
            json!(VacuumFanSpeed::from(*mode)),
        )),
        // StateCapability::Range {
        //     function: RangeFunction::Temperature,
        //     value,
        //     relative: _, // TODO: implement
        // } => (Capability::Temperature, json!(value)),
        _ => todo!(),
    }
}

fn prepare_response_capability(
    capability: &StateCapability,
    result: StateUpdateResult,
) -> UpdateStateCapability {
    match capability {
        StateCapability::OnOff { value: _ } => UpdateStateCapability::on_off(result),
        StateCapability::Mode {
            function: ModeFunction::FanSpeed,
            mode: _,
        } => UpdateStateCapability::mode(ModeFunction::FanSpeed, result),
        StateCapability::Mode {
            function: ModeFunction::WorkSpeed,
            mode: _,
        } => todo!(),
        StateCapability::Toggle {
            function: ToggleFunction::Pause,
            value: _,
        } => todo!(),
        StateCapability::Range {
            function: RangeFunction::Temperature,
            value: _,
            relative: _, // TODO: implement
        } => UpdateStateCapability::range(RangeFunction::Temperature, result),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alice::Mode;

    #[test]
    fn test_prepare_action_for_device() {
        assert_eq!(
            prepare_action_for_device(
                &StateCapability::OnOff { value: true },
                &Device::Recuperator
            ),
            Some((ElizabethState::IsEnabled.into(), json!(true)))
        );

        assert_eq!(
            prepare_action_for_device(
                &StateCapability::OnOff { value: false },
                &Device::Recuperator
            ),
            Some((ElizabethState::IsEnabled.into(), json!(false)))
        );

        assert_eq!(
            prepare_action_for_device(&StateCapability::OnOff { value: true }, &Device::Thermostat),
            Some((ElizabethState::IsEnabled.into(), json!(true)))
        );

        assert_eq!(
            prepare_action_for_device(
                &StateCapability::OnOff { value: false },
                &Device::Thermostat
            ),
            Some((ElizabethState::IsEnabled.into(), json!(false)))
        );

        assert_eq!(
            prepare_action_for_device(
                &StateCapability::Mode {
                    function: ModeFunction::FanSpeed,
                    mode: Mode::Low
                },
                &Device::Recuperator
            ),
            Some((ElizabethState::FanSpeed.into(), json!(Mode::Low)))
        );

        assert_eq!(
            prepare_action_for_device(
                &StateCapability::Mode {
                    function: ModeFunction::FanSpeed,
                    mode: Mode::Medium
                },
                &Device::Recuperator
            ),
            Some((ElizabethState::FanSpeed.into(), json!(Mode::Medium)))
        );

        assert_eq!(
            prepare_action_for_device(
                &StateCapability::Mode {
                    function: ModeFunction::FanSpeed,
                    mode: Mode::High
                },
                &Device::Recuperator
            ),
            Some((ElizabethState::FanSpeed.into(), json!(Mode::High)))
        );

        assert_eq!(
            prepare_action_for_device(
                &StateCapability::Mode {
                    function: ModeFunction::WorkSpeed,
                    mode: Mode::Quiet
                },
                &Device::VacuumCleaner
            ),
            Some((
                ElisaAction::SetFanSpeed.into(),
                json!(VacuumFanSpeed::Silent)
            ))
        );

        assert_eq!(
            prepare_action_for_device(
                &StateCapability::Mode {
                    function: ModeFunction::WorkSpeed,
                    mode: Mode::Normal
                },
                &Device::VacuumCleaner
            ),
            Some((
                ElisaAction::SetFanSpeed.into(),
                json!(VacuumFanSpeed::Standard)
            ))
        );

        assert_eq!(
            prepare_action_for_device(
                &StateCapability::Mode {
                    function: ModeFunction::WorkSpeed,
                    mode: Mode::Medium
                },
                &Device::VacuumCleaner
            ),
            Some((
                ElisaAction::SetFanSpeed.into(),
                json!(VacuumFanSpeed::Medium)
            ))
        );

        assert_eq!(
            prepare_action_for_device(
                &StateCapability::Mode {
                    function: ModeFunction::WorkSpeed,
                    mode: Mode::Turbo
                },
                &Device::VacuumCleaner
            ),
            Some((
                ElisaAction::SetFanSpeed.into(),
                json!(VacuumFanSpeed::Turbo)
            ))
        );
    }

    #[test]
    fn test_prepare_response_capability() {
        assert_eq!(
            prepare_response_capability(
                &StateCapability::OnOff { value: true },
                StateUpdateResult::ok()
            ),
            UpdateStateCapability::on_off(StateUpdateResult::ok())
        );

        assert_eq!(
            prepare_response_capability(
                &StateCapability::Mode {
                    function: ModeFunction::FanSpeed,
                    mode: Mode::Low
                },
                StateUpdateResult::ok()
            ),
            UpdateStateCapability::mode(ModeFunction::FanSpeed, StateUpdateResult::ok())
        );

        assert_eq!(
            prepare_response_capability(
                &StateCapability::Range {
                    function: RangeFunction::Temperature,
                    value: 20.0,
                    relative: false
                },
                StateUpdateResult::ok()
            ),
            UpdateStateCapability::range(RangeFunction::Temperature, StateUpdateResult::ok())
        );

        assert_eq!(
            prepare_response_capability(
                &StateCapability::OnOff { value: true },
                StateUpdateResult::error(
                    UpdateStateErrorCode::DeviceUnreachable,
                    "device unreachable".to_string()
                )
            ),
            UpdateStateCapability::on_off(StateUpdateResult::error(
                UpdateStateErrorCode::DeviceUnreachable,
                "device unreachable".to_string()
            ))
        );

        assert_eq!(
            prepare_response_capability(
                &StateCapability::Mode {
                    function: ModeFunction::FanSpeed,
                    mode: Mode::Low
                },
                StateUpdateResult::error(
                    UpdateStateErrorCode::DeviceUnreachable,
                    "device unreachable".to_string()
                )
            ),
            UpdateStateCapability::mode(
                ModeFunction::FanSpeed,
                StateUpdateResult::error(
                    UpdateStateErrorCode::DeviceUnreachable,
                    "device unreachable".to_string()
                )
            )
        );

        assert_eq!(
            prepare_response_capability(
                &StateCapability::Range {
                    function: RangeFunction::Temperature,
                    value: 20.0,
                    relative: false
                },
                StateUpdateResult::error(
                    UpdateStateErrorCode::DeviceUnreachable,
                    "device unreachable".to_string()
                )
            ),
            UpdateStateCapability::range(
                RangeFunction::Temperature,
                StateUpdateResult::error(
                    UpdateStateErrorCode::DeviceUnreachable,
                    "device unreachable".to_string()
                )
            )
        );
    }
}

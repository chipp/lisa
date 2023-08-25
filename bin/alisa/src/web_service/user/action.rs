use alice::{
    ModeFunction, RangeFunction, StateCapability, StateUpdateResult, ToggleFunction,
    UpdateStateCapability, UpdateStateErrorCode, UpdateStateRequest, UpdateStateResponse,
    UpdatedDeviceState,
};

use std::str::FromStr;
use std::todo;

use bytes::Buf;
use hyper::{Body, Request, Response, StatusCode};
use serde_json::{json, Value};
use tokio::sync::mpsc::UnboundedSender;

use crate::types::{Capability, DeviceId, UpdatePayload};
use crate::web_service::auth::validate_autorization;
use crate::Result;

pub async fn action(
    request: Request<Body>,
    update_device: UnboundedSender<UpdatePayload>,
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

        for device in action.payload.devices {
            let DeviceId { room, device_type } = DeviceId::from_str(device.id).unwrap();
            let mut response_capabilities = vec![];

            for state_capability in device.capabilities {
                let (capability, value) = prepare_notification_capability(&state_capability);
                let result = match update_device.send(UpdatePayload {
                    device_type,
                    room,
                    capability,
                    value,
                }) {
                    Ok(()) => StateUpdateResult::ok(),
                    Err(err) => StateUpdateResult::error(
                        UpdateStateErrorCode::DeviceUnreachable,
                        err.to_string(),
                    ),
                };

                response_capabilities.push(prepare_response_capability(&state_capability, result));
            }

            response_devices.push(UpdatedDeviceState::new(
                device.id.to_string(),
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

fn prepare_notification_capability(capability: &StateCapability) -> (Capability, Value) {
    match capability {
        StateCapability::OnOff { value } => (Capability::IsEnabled, json!(value)),
        StateCapability::Mode {
            function: ModeFunction::FanSpeed,
            mode,
        } => (Capability::FanSpeed, json!(mode)),
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
            value,
            relative: _, // TODO: implement
        } => (Capability::Temperature, json!(value)),
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
    fn test_prepare_notification_capability() {
        assert_eq!(
            prepare_notification_capability(&StateCapability::OnOff { value: true }),
            (Capability::IsEnabled, Value::Bool(true))
        );

        assert_eq!(
            prepare_notification_capability(&StateCapability::OnOff { value: false }),
            (Capability::IsEnabled, Value::Bool(false))
        );

        assert_eq!(
            prepare_notification_capability(&StateCapability::Mode {
                function: ModeFunction::FanSpeed,
                mode: Mode::Low
            }),
            (Capability::FanSpeed, Value::String("low".to_string()))
        );

        assert_eq!(
            prepare_notification_capability(&StateCapability::Mode {
                function: ModeFunction::FanSpeed,
                mode: Mode::Medium
            }),
            (Capability::FanSpeed, Value::String("medium".to_string()))
        );

        assert_eq!(
            prepare_notification_capability(&StateCapability::Mode {
                function: ModeFunction::FanSpeed,
                mode: Mode::High
            }),
            (Capability::FanSpeed, Value::String("high".to_string()))
        );

        assert_eq!(
            prepare_notification_capability(&StateCapability::Range {
                function: RangeFunction::Temperature,
                value: 20.0,
                relative: false
            }),
            (
                Capability::Temperature,
                Value::Number(serde_json::Number::from_f64(20.0).unwrap())
            )
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

use alice::{
    ModeFunction, RangeFunction, StateCapability, StateUpdateResult, ToggleFunction,
    UpdateStateCapability, UpdateStateErrorCode, UpdateStateRequest, UpdateStateResponse,
    UpdatedDeviceState,
};
use transport::elisa::Action as ElisaAction;
use transport::elizabeth::{Action as ElizabethAction, ActionType as ElizabethActionType};
use transport::{DeviceType, Room};

use std::str::FromStr;

use bytes::Buf;
use hyper::{Body, Request, Response, StatusCode};
use log::{debug, error};
use tokio::sync::mpsc::UnboundedSender;

use crate::types::DeviceId;
use crate::web_service::auth::validate_autorization;
use crate::{Action, Result};

pub async fn action(
    request: Request<Body>,
    perform_action: UnboundedSender<Action>,
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

        let mut elisa_action = None;
        let mut elisa_response = vec![];

        for device in action.payload.devices {
            let DeviceId { room, device_type } = DeviceId::from_str(device.id).unwrap();

            match device_type {
                DeviceType::Recuperator | DeviceType::Thermostat => {
                    let response_capabilities = handle_elizabeth_capabilities(
                        device_type,
                        room,
                        device.capabilities,
                        perform_action.clone(),
                    );

                    response_devices.push(UpdatedDeviceState::new(
                        device.id.to_string(),
                        response_capabilities,
                    ));
                }
                DeviceType::VacuumCleaner => {
                    handle_elisa_capabilities(room, &device.capabilities, &mut elisa_action);

                    elisa_response.push(UpdatedDeviceState::new(
                        device.id.to_string(),
                        device
                            .capabilities
                            .iter()
                            .map(|capability| {
                                prepare_response_capability(capability, StateUpdateResult::ok())
                            })
                            .collect(),
                    ));
                }
                DeviceType::TemperatureSensor => todo!(),
            };
        }

        if let Some(action) = elisa_action {
            let result = match perform_action.send(Action::Elisa(action)) {
                Ok(()) => StateUpdateResult::ok(),
                Err(err) => StateUpdateResult::error(
                    UpdateStateErrorCode::DeviceUnreachable,
                    err.to_string(),
                ), // TODO: differentiate errors
            };

            for mut response_device in elisa_response {
                for capability in response_device.capabilities_mut() {
                    *capability.result_mut() = result.clone();
                }

                response_devices.push(response_device);
            }
        }

        let response = UpdateStateResponse::new(request_id, response_devices);

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&response)?))?)
    })
    .await
}

fn handle_elizabeth_capabilities(
    device_type: DeviceType,
    room: Room,
    capabilities: Vec<StateCapability>,
    perform_action: UnboundedSender<Action>,
) -> Vec<UpdateStateCapability> {
    let mut response = vec![];

    for capability in capabilities {
        let action = map_elizabeth_action(&capability)
            .map(|action_type| {
                ElizabethAction {
                    room,
                    device_type,
                    action_type,
                }
                .into()
            })
            .unwrap();

        debug!("perform action {:?}", action);

        let result = match perform_action.send(action) {
            Ok(()) => StateUpdateResult::ok(),
            Err(err) => {
                StateUpdateResult::error(UpdateStateErrorCode::DeviceUnreachable, err.to_string())
            } // TODO: differentiate errors
        };

        response.push(prepare_response_capability(&capability, result));
    }

    response
}

fn handle_elisa_capabilities(
    room: Room,
    capabilities: &[StateCapability],
    elisa_action: &mut Option<ElisaAction>,
) {
    let actions: Vec<_> = capabilities
        .iter()
        .map(|capability| map_elisa_action(capability, room).unwrap())
        .collect();

    for action in actions {
        match (elisa_action.as_mut(), action) {
            (None, action) => *elisa_action = Some(action),
            (Some(ElisaAction::Start(rooms)), ElisaAction::Start(mut new_rooms)) => {
                rooms.append(&mut new_rooms);
            }
            _ => (),
        }
    }
}

fn map_elizabeth_action(state_capability: &StateCapability) -> Option<ElizabethActionType> {
    match state_capability {
        StateCapability::OnOff { value } => Some(ElizabethActionType::SetIsEnabled(*value)),
        StateCapability::Mode {
            function: ModeFunction::FanSpeed,
            mode,
        } => Some(ElizabethActionType::SetFanSpeed(map_mode_to_fan_speed(
            *mode,
        ))),
        StateCapability::Range {
            function: RangeFunction::Temperature,
            value,
            relative,
        } => Some(ElizabethActionType::SetTemperature(*value, *relative)),
        _ => {
            error!(
                "Unsupported state capability for elizabeth: {:?}",
                state_capability
            );
            None
        }
    }
}

fn map_elisa_action(state_capability: &StateCapability, room: Room) -> Option<ElisaAction> {
    match state_capability {
        StateCapability::OnOff { value } => {
            if *value {
                Some(ElisaAction::Start(vec![room]))
            } else {
                Some(ElisaAction::Stop)
            }
        }
        StateCapability::Mode {
            function: ModeFunction::WorkSpeed,
            mode,
        } => Some(ElisaAction::SetWorkSpeed(map_mode_to_work_speed(*mode))),
        StateCapability::Toggle {
            function: ToggleFunction::Pause,
            value,
        } => {
            if *value {
                Some(ElisaAction::Pause)
            } else {
                Some(ElisaAction::Resume)
            }
        }
        _ => {
            error!(
                "Unsupported state capability for elisa: {:?}",
                state_capability
            );
            None
        }
    }
}

fn map_mode_to_fan_speed(mode: alice::Mode) -> transport::elizabeth::FanSpeed {
    match mode {
        alice::Mode::Low => transport::elizabeth::FanSpeed::Low,
        alice::Mode::Medium => transport::elizabeth::FanSpeed::Medium,
        alice::Mode::High => transport::elizabeth::FanSpeed::High,
        alice::Mode::Quiet | alice::Mode::Normal | alice::Mode::Turbo => {
            panic!("Unsupported mode {} for recuperator", mode)
        }
    }
}

fn map_mode_to_work_speed(mode: alice::Mode) -> transport::elisa::WorkSpeed {
    match mode {
        alice::Mode::Quiet => transport::elisa::WorkSpeed::Silent,
        alice::Mode::Normal => transport::elisa::WorkSpeed::Standard,
        alice::Mode::Medium => transport::elisa::WorkSpeed::Medium,
        alice::Mode::Turbo => transport::elisa::WorkSpeed::Turbo,
        alice::Mode::Low | alice::Mode::High => {
            panic!("Unsupported mode {} for vacuum cleaner", mode)
        }
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
        StateCapability::Range {
            function: RangeFunction::Temperature,
            value: _,
            relative: _,
        } => UpdateStateCapability::range(RangeFunction::Temperature, result),
        StateCapability::Mode {
            function: ModeFunction::WorkSpeed,
            mode: _,
        } => UpdateStateCapability::mode(ModeFunction::WorkSpeed, result),
        StateCapability::Toggle {
            function: ToggleFunction::Pause,
            value: _,
        } => UpdateStateCapability::toggle(ToggleFunction::Pause, result),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alice::Mode;

    #[test]
    fn map_fan_speed() {
        assert_eq!(
            map_mode_to_fan_speed(Mode::Low),
            transport::elizabeth::FanSpeed::Low
        );
        assert_eq!(
            map_mode_to_fan_speed(Mode::Medium),
            transport::elizabeth::FanSpeed::Medium
        );
        assert_eq!(
            map_mode_to_fan_speed(Mode::High),
            transport::elizabeth::FanSpeed::High
        );
    }

    #[test]
    fn map_work_speed() {
        assert_eq!(
            map_mode_to_work_speed(Mode::Quiet),
            transport::elisa::WorkSpeed::Silent
        );
        assert_eq!(
            map_mode_to_work_speed(Mode::Normal),
            transport::elisa::WorkSpeed::Standard
        );
        assert_eq!(
            map_mode_to_work_speed(Mode::Medium),
            transport::elisa::WorkSpeed::Medium
        );
        assert_eq!(
            map_mode_to_work_speed(Mode::Turbo),
            transport::elisa::WorkSpeed::Turbo
        );
    }

    #[test]
    fn enable_recuperator() {
        let state_capability = StateCapability::OnOff { value: true };

        assert_eq!(
            map_elizabeth_action(&state_capability),
            Some(ElizabethActionType::SetIsEnabled(true))
        );
    }

    #[test]
    fn disable_recuperator() {
        let state_capability = StateCapability::OnOff { value: false };

        assert_eq!(
            map_elizabeth_action(&state_capability),
            Some(ElizabethActionType::SetIsEnabled(false))
        );
    }

    #[test]
    fn set_recuperator_fan_speed() {
        let state_capability = StateCapability::Mode {
            function: ModeFunction::FanSpeed,
            mode: Mode::Low,
        };

        assert_eq!(
            map_elizabeth_action(&state_capability),
            Some(ElizabethActionType::SetFanSpeed(
                transport::elizabeth::FanSpeed::Low
            ))
        );
    }

    #[test]
    fn enable_thermostat() {
        let state_capability = StateCapability::OnOff { value: true };

        assert_eq!(
            map_elizabeth_action(&state_capability),
            Some(ElizabethActionType::SetIsEnabled(true))
        );
    }

    #[test]
    fn disable_thermostat() {
        let state_capability = StateCapability::OnOff { value: false };

        assert_eq!(
            map_elizabeth_action(&state_capability),
            Some(ElizabethActionType::SetIsEnabled(false))
        );
    }

    #[test]
    fn set_thermostat_temperature_absolute() {
        let state_capability = StateCapability::Range {
            function: RangeFunction::Temperature,
            value: 20.0,
            relative: false,
        };

        assert_eq!(
            map_elizabeth_action(&state_capability),
            Some(ElizabethActionType::SetTemperature(20.0, false))
        );
    }

    #[test]
    fn set_thermostat_temperature_relative() {
        let state_capability = StateCapability::Range {
            function: RangeFunction::Temperature,
            value: 2.0,
            relative: true,
        };

        assert_eq!(
            map_elizabeth_action(&state_capability),
            Some(ElizabethActionType::SetTemperature(2.0, true))
        );
    }

    #[test]
    fn start_vacuum_cleaner() {
        let state_capability = StateCapability::OnOff { value: true };
        let room = Room::LivingRoom;

        assert_eq!(
            map_elisa_action(&state_capability, room),
            Some(ElisaAction::Start(vec![Room::LivingRoom]))
        );
    }

    #[test]
    fn stop_vacuum_cleaner() {
        let state_capability = StateCapability::OnOff { value: false };
        let room = Room::LivingRoom;

        assert_eq!(
            map_elisa_action(&state_capability, room),
            Some(ElisaAction::Stop)
        );
    }

    #[test]
    fn set_vacuum_cleaner_work_speed() {
        let state_capability = StateCapability::Mode {
            function: ModeFunction::WorkSpeed,
            mode: Mode::Quiet,
        };
        let room = Room::LivingRoom;

        assert_eq!(
            map_elisa_action(&state_capability, room),
            Some(ElisaAction::SetWorkSpeed(
                transport::elisa::WorkSpeed::Silent
            ))
        );
    }

    #[test]
    fn test_prepare_response_capability() {
        assert_eq!(
            prepare_response_capability(
                &StateCapability::OnOff { value: true },
                StateUpdateResult::ok(),
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

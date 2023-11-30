use std::collections::{HashMap, HashSet};
use std::time::Duration;

use alice::{
    ModeFunction, RangeFunction, StateCapability, StateUpdateResult, ToggleFunction,
    UpdateStateCapability, UpdateStateErrorCode, UpdateStateRequest, UpdateStateResponse,
    UpdatedDeviceState,
};
use futures_util::StreamExt;
use transport::elisa::Action as ElisaAction;
use transport::elizabeth::{Action as ElizabethAction, ActionType as ElizabethActionType};
use transport::{DeviceId, DeviceType, Room, Topic};

use bytes::Buf;
use hyper::{Body, Request, Response, StatusCode};
use log::{debug, error};
use paho_mqtt::{AsyncClient, Message, MessageBuilder, Properties, PropertyCode, QOS_1};
use uuid::Uuid;

use crate::web_service::auth::validate_autorization;
use crate::Result;

pub async fn action(
    request: Request<Body>,
    mut mqtt_client: AsyncClient,
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

        let mut response_capabilities = HashMap::new();

        let mut action_ids = HashSet::new();
        let mut actions = vec![];

        let mut elisa_action = None;
        let elisa_action_id = Uuid::new_v4();

        for device in action.payload.devices {
            match device.id.device_type {
                DeviceType::Recuperator | DeviceType::Thermostat => {
                    let result = handle_elizabeth_capabilities(
                        device.id.device_type,
                        device.id.room,
                        &device.capabilities,
                    );

                    for (action, capability) in result {
                        response_capabilities.insert(action.id(), (device.id, capability));
                        actions.push(action);
                    }
                }
                DeviceType::VacuumCleaner => {
                    let result = handle_elisa_capabilities(
                        device.id.room,
                        &device.capabilities,
                        &mut elisa_action,
                    );

                    for capability in result {
                        response_capabilities.insert(elisa_action_id, (device.id, capability));
                    }
                }
                DeviceType::TemperatureSensor => (),
            };
        }

        if let Some(action) = elisa_action {
            action_ids.insert(elisa_action_id);
            actions.push(transport::action::Action::Elisa(action, elisa_action_id));
        }

        let request = transport::action::ActionRequest { actions };

        let request_topic = Topic::ActionRequest.to_string();
        let response_topic = Topic::ActionResponse(request_id.clone()).to_string();

        mqtt_client.subscribe(&response_topic, QOS_1);
        let mut stream = mqtt_client.get_stream(1);

        let mut props = Properties::new();
        props.push_string(PropertyCode::ResponseTopic, &response_topic)?;

        debug!("posting to {} {:?}", request_topic, request);

        let request_msg = MessageBuilder::new()
            .topic(request_topic)
            .properties(props)
            .payload(serde_json::to_vec(&request)?)
            .finalize();

        mqtt_client.publish(request_msg).await?;

        while let Ok(Some(msg_opt)) =
            tokio::time::timeout(Duration::from_secs(10), stream.next()).await
        {
            if let Some(msg) = msg_opt {
                handle_message(msg, &mut action_ids, &mut response_capabilities);
            }

            if action_ids.is_empty() {
                break;
            }
        }

        let response_devices = group(response_capabilities.into_values().into_iter())
            .into_iter()
            .map(|(id, capabilities)| UpdatedDeviceState::new(id, capabilities))
            .collect();

        let response = UpdateStateResponse::new(request_id, response_devices);

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&response)?))?)
    })
    .await
}

fn group<I>(iter: I) -> HashMap<DeviceId, Vec<UpdateStateCapability>>
where
    I: Iterator<Item = (DeviceId, UpdateStateCapability)>,
{
    let mut map = HashMap::new();
    map.reserve(iter.size_hint().0);

    for (device_id, capability) in iter {
        map.entry(device_id)
            .or_insert_with(Vec::new)
            .push(capability);
    }

    map.shrink_to_fit();

    map
}

fn handle_elizabeth_capabilities(
    device_type: DeviceType,
    room: Room,
    capabilities: &[StateCapability],
) -> Vec<(transport::action::Action, UpdateStateCapability)> {
    capabilities
        .iter()
        .filter_map(|capability| {
            map_elizabeth_action(capability).map(|action_type| {
                (
                    transport::action::Action::Elizabeth(
                        ElizabethAction {
                            room,
                            device_type,
                            action_type,
                        },
                        Uuid::new_v4(),
                    ),
                    prepare_response_capability(capability),
                )
            })
        })
        .collect()
}

fn handle_elisa_capabilities(
    room: Room,
    capabilities: &[StateCapability],
    current_action: &mut Option<ElisaAction>,
) -> Vec<UpdateStateCapability> {
    let actions: Vec<_> = capabilities
        .iter()
        .map(|capability| map_elisa_action(capability, room).unwrap())
        .collect();

    for action in actions {
        match (current_action.as_mut(), action) {
            (None, action) => *current_action = Some(action),
            (Some(ElisaAction::Start(rooms)), ElisaAction::Start(mut new_rooms)) => {
                rooms.append(&mut new_rooms);
            }
            _ => (),
        }
    }

    capabilities
        .iter()
        .map(prepare_response_capability)
        .collect()
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

fn handle_message(
    msg: Message,
    action_ids: &mut HashSet<Uuid>,
    devices: &mut HashMap<Uuid, (DeviceId, UpdateStateCapability)>,
) {
    use transport::action::ActionResponse;

    let response: ActionResponse = serde_json::from_slice(msg.payload()).unwrap();

    if action_ids.contains(&response.action_id) {
        action_ids.remove(&response.action_id);
    }

    if let Some((_, capability)) = devices.get_mut(&response.action_id) {
        *capability.result_mut() = match response.result {
            transport::action::ActionResult::Success => StateUpdateResult::ok(),
            transport::action::ActionResult::Failure => {
                StateUpdateResult::error(UpdateStateErrorCode::DeviceUnreachable, String::new())
            }
        }
    }
}

fn prepare_response_capability(capability: &StateCapability) -> UpdateStateCapability {
    let result = StateUpdateResult::error(
        UpdateStateErrorCode::DeviceUnreachable,
        "device unreachable".to_string(),
    );

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

    // #[test]
    // fn test_prepare_response_capability() {
    //     assert_eq!(
    //         prepare_response_capability(
    //             &StateCapability::OnOff { value: true },
    //             StateUpdateResult::ok(),
    //         ),
    //         UpdateStateCapability::on_off(StateUpdateResult::ok())
    //     );

    //     assert_eq!(
    //         prepare_response_capability(
    //             &StateCapability::Mode {
    //                 function: ModeFunction::FanSpeed,
    //                 mode: Mode::Low
    //             },
    //             StateUpdateResult::ok()
    //         ),
    //         UpdateStateCapability::mode(ModeFunction::FanSpeed, StateUpdateResult::ok())
    //     );

    //     assert_eq!(
    //         prepare_response_capability(
    //             &StateCapability::Range {
    //                 function: RangeFunction::Temperature,
    //                 value: 20.0,
    //                 relative: false
    //             },
    //             StateUpdateResult::ok()
    //         ),
    //         UpdateStateCapability::range(RangeFunction::Temperature, StateUpdateResult::ok())
    //     );

    //     assert_eq!(
    //         prepare_response_capability(
    //             &StateCapability::OnOff { value: true },
    //             StateUpdateResult::error(
    //                 UpdateStateErrorCode::DeviceUnreachable,
    //                 "device unreachable".to_string()
    //             )
    //         ),
    //         UpdateStateCapability::on_off(StateUpdateResult::error(
    //             UpdateStateErrorCode::DeviceUnreachable,
    //             "device unreachable".to_string()
    //         ))
    //     );

    //     assert_eq!(
    //         prepare_response_capability(
    //             &StateCapability::Mode {
    //                 function: ModeFunction::FanSpeed,
    //                 mode: Mode::Low
    //             },
    //             StateUpdateResult::error(
    //                 UpdateStateErrorCode::DeviceUnreachable,
    //                 "device unreachable".to_string()
    //             )
    //         ),
    //         UpdateStateCapability::mode(
    //             ModeFunction::FanSpeed,
    //             StateUpdateResult::error(
    //                 UpdateStateErrorCode::DeviceUnreachable,
    //                 "device unreachable".to_string()
    //             )
    //         )
    //     );

    //     assert_eq!(
    //         prepare_response_capability(
    //             &StateCapability::Range {
    //                 function: RangeFunction::Temperature,
    //                 value: 20.0,
    //                 relative: false
    //             },
    //             StateUpdateResult::error(
    //                 UpdateStateErrorCode::DeviceUnreachable,
    //                 "device unreachable".to_string()
    //             )
    //         ),
    //         UpdateStateCapability::range(
    //             RangeFunction::Temperature,
    //             StateUpdateResult::error(
    //                 UpdateStateErrorCode::DeviceUnreachable,
    //                 "device unreachable".to_string()
    //             )
    //         )
    //     );
    // }
}

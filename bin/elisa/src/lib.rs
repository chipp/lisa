mod status;
mod topic;

pub use status::Status;
pub use topic::{actions_topics_and_qos, topic_for_state};

pub use topics::{ElisaAction as Action, ElisaState as State, Room};

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;

pub fn room_id_for_room(room: &Room) -> u8 {
    match room {
        // TODO: read configuration to config file
        Room::Bathroom => 11,
        Room::Bedroom => 13,
        Room::Corridor => 15,
        Room::Hallway => 12,
        Room::HomeOffice => 17,
        Room::Kitchen => 16,
        Room::LivingRoom => 18,
        Room::Nursery => 14,
        Room::Toilet => 10,
    }
}

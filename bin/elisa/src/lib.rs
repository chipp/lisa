mod status;
pub use status::Status;

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;

pub fn room_id_for_room(room: &transport::Room) -> u8 {
    match room {
        // TODO: read configuration to config file
        transport::Room::Bathroom => 11,
        transport::Room::Bedroom => 13,
        transport::Room::Corridor => 15,
        transport::Room::Hallway => 12,
        transport::Room::HomeOffice => 17,
        transport::Room::Kitchen => 16,
        transport::Room::LivingRoom => 18,
        transport::Room::Nursery => 14,
        transport::Room::Toilet => 10,
    }
}

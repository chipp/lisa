pub mod incoming {
    mod update;
    pub use update::UpdateMessageContent;

    use serde::Deserialize;
    use serde_json::Value;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ReceivedMessage {
        pub code: String,
        pub sequence: u32,
        pub message: Value,
    }
}

pub mod outgoing {
    mod register;
    pub use register::RegisterMessage;

    mod sql_request;
    pub use sql_request::SqlRequestMessage;

    mod update_state;
    pub use update_state::{UpdateStateMessage, UpdateStateMessageContent};
}

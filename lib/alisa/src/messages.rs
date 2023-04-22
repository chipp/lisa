pub mod incoming {
    mod update;
    pub use update::UpdateMessageContent;

    use serde::Deserialize;
    use serde_json::Value;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ReceivedMessage {
        pub code: String,
        // pub sequence: Option<u32>,
        pub message: Value,
    }
}

pub mod outgoing {
    mod keep_alive;
    mod register;
    mod sql_request;
    mod update_state;

    pub use keep_alive::KeepAliveMessage;
    pub use register::RegisterMessage;
    pub use sql_request::SqlRequestMessage;
    pub use update_state::{UpdateStateMessage, UpdateStateMessageContent};
}

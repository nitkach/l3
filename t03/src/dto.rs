use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct PostJoinRoom {
    pub(crate) room_key: String,
    pub(crate) user: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PostLeaveRoom {
    pub(crate) room_key: String,
    pub(crate) user: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PostSendMessage {
    pub(crate) room_key: String,
    pub(crate) user: String,
    pub(crate) message: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GetMessages {
    pub(crate) room_key: String,
    pub(crate) user: String,
}

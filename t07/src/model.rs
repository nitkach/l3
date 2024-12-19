use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Event {
    pub(crate) event_type: String,
    pub(crate) data: String,
}

impl Event {
    pub(crate) fn get_type(&self) -> &str {
        &self.event_type
    }

    pub(crate) fn get_data(&self) -> &str {
        &self.data
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Subscription {
    event_type: String,
    user_id: u64,
    webhook_url: String,
}

impl Subscription {
    pub(crate) fn get_event_type(&self) -> &str {
        &self.event_type
    }

    pub(crate) fn get_user_id(&self) -> u64 {
        self.user_id
    }

    pub(crate) fn get_webhook_url(&self) -> &str {
        &self.webhook_url
    }
}

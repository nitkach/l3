use std::{collections::BTreeSet, fmt::Display};

use serde::Serialize;

#[derive(Clone, Debug, Ord, PartialEq, PartialOrd, Eq, Serialize)]
pub(crate) struct User {
    pub(crate) username: String,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.username)
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct Message {
    pub(crate) author: User,
    pub(crate) content: String,
    pub(crate) timestamp: u64,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct RoomKey {
    pub(crate) id: String,
}

impl std::fmt::Display for RoomKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Room {
    pub(crate) users: BTreeSet<User>,
    pub(crate) messages: Vec<Message>,
}

impl Room {
    pub(crate) fn new() -> Self {
        Self {
            users: BTreeSet::new(),
            messages: Vec::new(),
        }
    }

    pub(crate) fn with_user(user: User) -> Self {
        Self {
            users: BTreeSet::from_iter([user]),
            messages: Vec::new(),
        }
    }
}

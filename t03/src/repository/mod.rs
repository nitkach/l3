use crate::{
    error::{AppError, ErrorKind},
    model::{Message, Room, RoomKey, User},
};
use anyhow::Result;
use dashmap::{DashMap, Entry};
use std::sync::{atomic::AtomicUsize, Arc};

#[derive(Clone)]
pub(crate) struct Repository {
    rooms: DashMap<RoomKey, Room>,
    user_count: Arc<AtomicUsize>,
}

impl Repository {
    pub(crate) fn initialize() -> Self {
        Self {
            rooms: DashMap::new(),
            user_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub(crate) fn add_user_to_room(&self, room_key: RoomKey, user: User) -> bool {
        let is_new_entry = match self.rooms.entry(room_key) {
            Entry::Occupied(mut occupied_entry) => occupied_entry.get_mut().users.insert(user),
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(Room::with_user(user));
                true
            }
        };

        self.user_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        is_new_entry
    }

    pub(crate) fn remove_user_from_room(
        &self,
        room_key: RoomKey,
        user: &User,
    ) -> Result<(), AppError> {
        match self.rooms.entry(room_key) {
            Entry::Occupied(mut occupied_entry) => {
                let is_user_removed = occupied_entry.get_mut().users.remove(user);
                let is_empty = occupied_entry.get().users.is_empty();

                match (is_user_removed, is_empty) {
                    // removed user was last, need to remove room
                    (true, true) => {
                        occupied_entry.remove_entry();
                    }
                    // user removed, but there are some users still in room
                    (true, false) => {}
                    // user not found in room, but there are some users in room
                    (false, true) => {
                        let room_key = occupied_entry.into_key();

                        return Err(AppError::conflict(ErrorKind::User(format!(
                            "user '{user}' not found in room '{room_key}'"
                        ))));
                    }
                    // invalid state
                    (false, false) => unreachable!(),
                };
            }
            Entry::Vacant(vacant) => {
                let room_key = vacant.into_key();
                return Err(AppError::not_found(ErrorKind::Room(format!(
                    "room not found with name: '{room_key}'"
                ))));
            }
        };

        self.user_count
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    pub(crate) fn send_message(
        &self,
        room_key: RoomKey,
        user: &User,
        message: Message,
    ) -> Result<(), AppError> {
        match self.rooms.entry(room_key) {
            Entry::Occupied(mut occupied_entry) => {
                if !occupied_entry.get().users.contains(user) {
                    let room_key = occupied_entry.into_key();
                    return Err(AppError::conflict(ErrorKind::User(format!(
                        "user '{user}' not found in room '{room_key}'"
                    ))));
                }
                occupied_entry.get_mut().messages.push(message);
            }
            Entry::Vacant(vacant) => {
                let room_key = vacant.into_key();
                return Err(AppError::not_found(ErrorKind::Room(format!(
                    "room not found with name: '{room_key}'"
                ))));
            }
        };

        Ok(())
    }

    pub(crate) fn get_messages(
        &self,
        room_key: RoomKey,
        user: &User,
    ) -> Result<Vec<Message>, AppError> {
        let messages = match self.rooms.entry(room_key) {
            Entry::Occupied(occupied_entry) => {
                if !occupied_entry.get().users.contains(user) {
                    let room_key = occupied_entry.into_key();
                    return Err(AppError::conflict(ErrorKind::User(format!(
                        "user '{user}' not found in room '{room_key}'"
                    ))));
                }
                occupied_entry.get().messages.clone()
            }
            Entry::Vacant(vacant) => {
                let room_key = vacant.into_key();
                return Err(AppError::not_found(ErrorKind::Room(format!(
                    "room not found with name: '{room_key}'"
                ))));
            }
        };

        Ok(messages)
    }
}

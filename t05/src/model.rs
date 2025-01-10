use ulid::Ulid;

use crate::dto::Direction;

#[derive(Clone)]
pub(crate) struct Player {
    pub(crate) ulid: Ulid,
    pub(crate) name: String,
    pub(crate) position: (i8, i8),
}

impl Player {
    pub(crate) fn new(ulid: Ulid, name: String) -> Self {
        Self {
            ulid,
            name,
            position: (0, 0),
        }
    }

    pub(crate) fn move_player(&mut self, direction: &Direction) {
        match direction {
            Direction::Up => self.position.0 = self.position.0.wrapping_add(1),
            Direction::Down => self.position.0 = self.position.0.wrapping_sub(1),
            Direction::Left => self.position.1 = self.position.1.wrapping_sub(1),
            Direction::Right => self.position.1 = self.position.1.wrapping_add(1),
            Direction::Stay => {}
        }
    }

    pub(crate) fn get_position(&self) -> (i8, i8) {
        (self.position.0, self.position.1)
    }
}

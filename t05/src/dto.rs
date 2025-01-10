use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) enum Action {
    Move { direction: Direction },
    Say { text: String },
    WhoIsNearby {},
}

#[derive(Debug, Deserialize)]
pub(crate) enum Direction {
    Up,
    Down,
    Left,
    Right,
    Stay,
}

impl TryFrom<&str> for Direction {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let direction = match value.to_lowercase().trim() {
            "up" => Self::Up,
            "down" => Self::Down,
            "left" => Self::Left,
            "right" => Self::Right,
            "stay" => Self::Stay,
            incorrect => {
                return Err(format!("incorrect direction: '{incorrect}'"));
            }
        };
        Ok(direction)
    }
}

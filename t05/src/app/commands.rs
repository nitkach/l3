use crate::dto::Direction;

pub(crate) enum Commands {
    Move(Direction),
    Say(String),
    WhoIsNearby,
}

impl TryFrom<&str> for Commands {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trim = value.to_lowercase();
        let command = match trim.split_whitespace().collect::<Vec<_>>().as_slice() {
            ["move", direction, ..] => {
                let direction = Direction::try_from(*direction)?;
                Self::Move(direction)
            }
            ["say", text @ ..] => {
                let text = text.join(" ");
                Self::Say(text)
            }
            ["whoisnearby", ..] => Self::WhoIsNearby,
            incorrect => return Err(incorrect.join(" ").to_string()),
        };
        Ok(command)
    }
}

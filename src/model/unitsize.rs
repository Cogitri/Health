use std::convert::TryFrom;
use std::fmt;

/// A [Unitsize] is so the user can choose to enter different unit sizes (e.g. km vs meter).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Unitsize {
    Big,
    Small,
}

impl fmt::Display for Unitsize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unitsize::Big => write!(f, "big"),
            Unitsize::Small => write!(f, "small"),
        }
    }
}

impl TryFrom<&str> for Unitsize {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "big" => Ok(Unitsize::Big),
            "small" => Ok(Unitsize::Small),
            _ => Err(format!("Unknown unitsize {}", value)),
        }
    }
}

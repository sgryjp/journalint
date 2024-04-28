/// The string value cannot be recognized as a time.
#[derive(thiserror::Error, Debug)]
#[error("Invalid time value `{}`: {}", .value, .msg)]
pub struct InvalidTimeValueError {
    value: String,
    msg: String,
}

impl InvalidTimeValueError {
    pub fn new<S, T>(value: S, msg: T) -> Self
    where
        S: Into<String>,
        T: Into<String>,
    {
        Self {
            value: value.into(),
            msg: msg.into(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Unknown violation rule `{}`", .rule)]
pub struct UnknownRule {
    pub(crate) rule: String,
}

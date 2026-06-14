/// Errors that can occur when triggering a Lua event from Rust.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchError {
    /// The event class was not found as a Lua global.
    NoHandler,
    /// One or more handlers returned `false` or cancelled the event.
    Cancelled,
    /// The handler threw a Lua error.
    HandlerError,
    /// `Event:trigger()` returned a non-boolean value (e.g. `nil` or a
    /// number), which is treated as cancellation by default.
    NoResult,
}

impl std::fmt::Display for DispatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DispatchError::NoHandler => write!(formatter, "no handler registered"),
            DispatchError::Cancelled => write!(formatter, "event was cancelled"),
            DispatchError::HandlerError => write!(formatter, "handler error"),
            DispatchError::NoResult => {
                write!(formatter, "event returned non-boolean value")
            }
        }
    }
}

impl std::error::Error for DispatchError {}

impl From<DispatchError> for mlua::Error {
    fn from(error: DispatchError) -> Self {
        mlua::Error::external(error)
    }
}

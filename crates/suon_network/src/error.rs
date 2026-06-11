#[derive(Debug)]
pub enum NetworkError {
    Bind(u16, std::io::Error),
    Resolve(String, std::io::Error),
    AlreadyRunning(u16),
    NotRunning(u16),
    Shutdown,
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkError::Bind(port, error) => {
                write!(formatter, "failed to bind port {port}: {error}")
            }
            NetworkError::Resolve(address, error) => {
                write!(formatter, "failed to resolve address {address}: {error}")
            }
            NetworkError::AlreadyRunning(port) => {
                write!(formatter, "server on port {port} is already running")
            }
            NetworkError::NotRunning(port) => {
                write!(formatter, "no server running on port {port}")
            }
            NetworkError::Shutdown => write!(formatter, "server is shutting down"),
        }
    }
}

impl std::error::Error for NetworkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NetworkError::Bind(_, error) => Some(error),
            NetworkError::Resolve(_, error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn display_bind() {
        let err = NetworkError::Bind(80, std::io::Error::from(std::io::ErrorKind::AddrInUse));
        let msg = err.to_string();
        assert!(msg.contains("80"));
        assert!(msg.contains("bind"));
    }

    #[test]
    fn display_resolve() {
        let err = NetworkError::Resolve(
            "invalid".into(),
            std::io::Error::from(std::io::ErrorKind::InvalidInput),
        );
        let msg = err.to_string();
        assert!(msg.contains("invalid"));
        assert!(msg.contains("resolve"));
    }

    #[test]
    fn display_already_running() {
        let err = NetworkError::AlreadyRunning(7171);
        assert_eq!(err.to_string(), "server on port 7171 is already running");
    }

    #[test]
    fn display_not_running() {
        let err = NetworkError::NotRunning(8080);
        assert_eq!(err.to_string(), "no server running on port 8080");
    }

    #[test]
    fn display_shutdown() {
        let err = NetworkError::Shutdown;
        assert_eq!(err.to_string(), "server is shutting down");
    }

    #[test]
    fn debug_format() {
        let err = NetworkError::NotRunning(7171);
        let debug = format!("{err:?}");
        assert!(debug.contains("NotRunning"));
    }

    #[test]
    fn source_bind_returns_inner() {
        let io_err = std::io::Error::from(std::io::ErrorKind::ConnectionRefused);
        let err = NetworkError::Bind(80, io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn source_resolve_returns_inner() {
        let io_err = std::io::Error::from(std::io::ErrorKind::NotFound);
        let err = NetworkError::Resolve("x".into(), io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn source_already_running_is_none() {
        let err = NetworkError::AlreadyRunning(1);
        assert!(err.source().is_none());
    }

    #[test]
    fn source_not_running_is_none() {
        let err = NetworkError::NotRunning(1);
        assert!(err.source().is_none());
    }

    #[test]
    fn source_shutdown_is_none() {
        let err = NetworkError::Shutdown;
        assert!(err.source().is_none());
    }
}

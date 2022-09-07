use crate::client::dsn;
use crate::messages;
use std::{error, fmt};
use tokio::io;

#[derive(Debug)]
pub enum MessageReadError {
    UnrecognizedMessageError(messages::UnrecognizedMessageError),
    IOError(io::Error),
}

impl error::Error for MessageReadError {}

impl fmt::Display for MessageReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MessageReadError::UnrecognizedMessageError(err) => {
                write!(f, "{}", err)
            }
            MessageReadError::IOError(err) => write!(f, "{}", err),
        }
    }
}

impl From<messages::UnrecognizedMessageError> for MessageReadError {
    fn from(err: messages::UnrecognizedMessageError) -> Self {
        MessageReadError::UnrecognizedMessageError(err)
    }
}

impl From<io::Error> for MessageReadError {
    fn from(err: io::Error) -> Self {
        MessageReadError::IOError(err)
    }
}

#[derive(Debug)]
pub enum FetchError {
    MessageReadError(MessageReadError),
    UnexpectedMessageError(messages::BackendMessage),
}

impl error::Error for FetchError {}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FetchError::MessageReadError(err) => {
                write!(f, "{}", err)
            }
            FetchError::UnexpectedMessageError(msg) => {
                write!(f, "unexpected message error: {:?}", msg)
            }
        }
    }
}

impl From<MessageReadError> for FetchError {
    fn from(err: MessageReadError) -> Self {
        FetchError::MessageReadError(err)
    }
}

impl From<io::Error> for FetchError {
    fn from(err: io::Error) -> Self {
        FetchError::MessageReadError(MessageReadError::IOError(err))
    }
}

#[derive(Debug)]
pub struct ServerError {
    pub severity: String,
    pub code: String,
    pub message: String,
}

impl error::Error for ServerError {}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} (code: {})",
            self.severity, self.message, self.code
        )
    }
}

impl From<messages::ErrorResponse> for ServerError {
    fn from(err_resp: messages::ErrorResponse) -> Self {
        ServerError {
            severity: err_resp.severity,
            code: err_resp.code,
            message: err_resp.message,
        }
    }
}

#[derive(Debug)]
pub enum ConnectionError {
    InvalidDsnError(dsn::InvalidDsnError),
    FetchError(FetchError),
    ServerError(ServerError),
}

impl error::Error for ConnectionError {}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::InvalidDsnError(err) => write!(f, "{}", err),
            ConnectionError::FetchError(err) => write!(f, "{}", err),
            ConnectionError::ServerError(err) => write!(f, "{}", err),
        }
    }
}

impl From<io::Error> for ConnectionError {
    fn from(err: io::Error) -> Self {
        ConnectionError::FetchError(FetchError::from(err))
    }
}

impl From<MessageReadError> for ConnectionError {
    fn from(err: MessageReadError) -> Self {
        ConnectionError::FetchError(FetchError::from(err))
    }
}

impl From<FetchError> for ConnectionError {
    fn from(err: FetchError) -> Self {
        ConnectionError::FetchError(err)
    }
}

impl From<dsn::InvalidDsnError> for ConnectionError {
    fn from(err: dsn::InvalidDsnError) -> Self {
        ConnectionError::InvalidDsnError(err)
    }
}

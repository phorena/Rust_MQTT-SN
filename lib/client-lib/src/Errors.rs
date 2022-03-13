#[derive(Debug, thiserror::Error)]
pub enum ExoError {
    #[error("Len Error: {0} (expeted {1})")]
    LenError(usize, usize),
    #[error("Wrong Message Type: {0} (expect {1}")]
    WrongMessageType(u8, u8),

    // return code
    #[error("Congestion: {0}")]
    Congestion(u8),
    #[error("Invalid Topic Id: {0}")]
    InvalidTopicId(u8),
    #[error("Not Supported: {0}")]
    NotSupported(u8),
    #[error("Return Code Reserved: {0}")]
    Reserved(u8),
}


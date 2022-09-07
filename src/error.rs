/// An error encountered during block parsing. This indicates the
/// block data is not structurally valid. Details are provided in
/// a freeform string message.
#[derive(Debug)]
pub struct BlockParseError {
    msg: String,
}

impl BlockParseError {
    pub(crate) fn new(msg: String) -> Self {
        BlockParseError {
            msg,
        }
    }
}

impl std::fmt::Display for BlockParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for BlockParseError {
}

/// An error encountered during block validation. This indicates the
/// block was not sufficiently valid to be added to the blockchain.
/// Details are provided in a freeform string message.
#[derive(Debug)]
pub struct BlockValidationError {
    msg: String,
}

impl BlockValidationError {
    pub(crate) fn new(msg: String) -> Self {
        BlockValidationError {
            msg,
        }
    }
}

impl std::fmt::Display for BlockValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for BlockValidationError {
}

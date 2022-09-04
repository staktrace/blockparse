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

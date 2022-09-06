use crate::{Block, BlockValidationError, Hash};

const MAX_SUPPORTED_BLOCK_VERSION: u32 = 4;

#[derive(Default)]
pub struct BlockValidator {
}

pub enum ValidationResult {
    Valid(Hash),
    Invalid(BlockValidationError),
    Orphan(Block),
}

impl BlockValidator {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_block(&mut self, block: Block) -> ValidationResult {
        // TODO: implement more things here. This is just enough scaffolding to avoid lint errors
        if block.header.version > MAX_SUPPORTED_BLOCK_VERSION {
            return ValidationResult::Invalid(BlockValidationError::new(format!("Block with unknown version: expected {} but got {}", MAX_SUPPORTED_BLOCK_VERSION, block.header.version)));
        }
        ValidationResult::Valid(block.id())
    }
}

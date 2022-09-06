use crate::{Block, BlockValidationError};

const MAX_SUPPORTED_BLOCK_VERSION: u32 = 4;

#[derive(Default)]
pub struct BlockValidator {
}

pub enum BlockValidationResult {
    Valid,
    Invalid(BlockValidationError),
    Orphan(Block),
}

impl BlockValidator {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_block(&mut self, block: Block) -> BlockValidationResult {
        // TODO: implement more things here. This is just enough scaffolding to avoid lint errors
        if block.header.version > MAX_SUPPORTED_BLOCK_VERSION {
            return BlockValidationResult::Invalid(BlockValidationError::new(format!("Block with unknown version: expected {} but got {}", MAX_SUPPORTED_BLOCK_VERSION, block.header.version)));
        }
        BlockValidationResult::Valid
    }
}

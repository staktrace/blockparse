use crate::{Block, BlockValidationError, Hash};
use std::collections::HashMap;

const MAX_SUPPORTED_BLOCK_VERSION: u32 = 4;

#[derive(Default)]
pub struct BlockValidator {
    archived_blocks: HashMap<Hash, usize>,
    active_blocks: HashMap<Hash, ActiveBlock>,
}

pub enum ValidationResult {
    Valid(Hash),
    Invalid(BlockValidationError),
    Orphan(Block),
}

struct ActiveBlock {
}

impl BlockValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_block(&mut self, block: Block) -> ValidationResult {
        if self.archived_blocks.contains_key(&block.header.prev_block_hash) {
            return ValidationResult::Invalid(
                BlockValidationError::new(format!("Candidate block {} has a previous block {} that is archived", block.id(), block.header.prev_block_hash))
            );
        }

        if !self.active_blocks.contains_key(&block.header.prev_block_hash) {
            return ValidationResult::Orphan(block);
        }

        if let Err(e) = self.validate_block(&block) {
            return ValidationResult::Invalid(e);
        }

        // TODO: insert block into active_blocks, and attach it up

        ValidationResult::Valid(block.id())
    }

    fn validate_block(&mut self, block: &Block) -> Result<(), BlockValidationError> {
        // TODO: implement more things here. This is just enough scaffolding to avoid lint errors
        if block.header.version > MAX_SUPPORTED_BLOCK_VERSION {
            return Err(BlockValidationError::new(format!("Block with unknown version: expected {} but got {}", MAX_SUPPORTED_BLOCK_VERSION, block.header.version)));
        }
        Ok(())
    }
}

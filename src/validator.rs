//! A module that exposes a block validation API.

use crate::{Block, BlockValidationError, Hash};
use std::collections::HashMap;

const MAX_SUPPORTED_BLOCK_VERSION: u32 = 4;

/// A state machine to validate blocks as they are received. This structure accepts
/// blocks one at a time, and checks to see if it is valid, updating internal state
/// as necessary. It can handle multiple active chains, such as when competing
/// miners produce different valid blocks for a given block height. It will eventually
/// discard abandoned chains if there is a clear "winner" chain.
#[derive(Default)]
pub struct BlockValidator {
    archived_blocks: HashMap<Hash, usize>,
    active_blocks: HashMap<Hash, ActiveBlock>,
}

/// Result from validation of a single block.
pub enum ValidationResult {
    /// The block was valid and was accepted into one of the active chains.
    Valid(Hash),
    /// The block was invalid, and therefore rejected.
    Invalid(BlockValidationError),
    /// The block could not be validated because the parent could not be found.
    /// In this case the block may have been received out-of-order, and should
    /// be tried again later after the indicated parent block has been validated.
    Orphan(Block),
}

struct ActiveBlock {
}

impl BlockValidator {
    /// Create a new validator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Give the validator one block to validate. If the block is valid, the
    /// validator's internal state gets updated and the block is attached to
    /// one of the active chains. Otherwise there should be no changes to
    /// the internal state.
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

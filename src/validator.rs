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
    block: Block,
    height: usize,
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

        let is_genesis_block = block.header.prev_block_hash == Hash::zero();

        let height = match self.active_blocks.get(&block.header.prev_block_hash) {
            Some(parent) => parent.height + 1,
            None if is_genesis_block => 0,
            None => return ValidationResult::Orphan(block),
        };

        if let Err(e) = self.validate_block(&block, height) {
            return ValidationResult::Invalid(e);
        }

        let hash = block.id();
        let active_block = ActiveBlock {
            block,
            height,
        };
        self.active_blocks.insert(hash, active_block);

        // TODO: archive old active blocks and prune tree

        ValidationResult::Valid(hash)
    }

    fn validate_block(&mut self, block: &Block, height: usize) -> Result<(), BlockValidationError> {
        // TODO: implement more things here. This is just enough scaffolding to avoid lint errors
        if block.header.version > MAX_SUPPORTED_BLOCK_VERSION {
            return Err(BlockValidationError::new(format!("Block with unknown version: expected {} but got {}", MAX_SUPPORTED_BLOCK_VERSION, block.header.version)));
        }
        if block.computed_merkle_root() != block.header.merkle_root {
            return Err(BlockValidationError::new(format!("Block with incorrect merkle root: expected {} but got {}", block.computed_merkle_root(), block.header.merkle_root)));
        }
        let target = match Hash::from_bits(block.header.bits) {
            None => return Err(BlockValidationError::new(format!("Target difficulty could not be computed from {:#x}", block.header.bits))),
            Some(target) => target,
        };
        if block.id() >= target {
            return Err(BlockValidationError::new(format!("Block header hash {} was not less than the target hash {}", block.id(), target)));
        }

        // For the genesis block, the above checks are all that we need to do.
        if height == 0 {
            return Ok(());
        }

        // All other blocks have a parent
        let parent = self.active_blocks.get(&block.header.prev_block_hash).unwrap();

        if block.header.time <= parent.block.header.time {
            return Err(BlockValidationError::new(format!("Block with time {} was not newer than parent block with time {}", block.header.time, parent.block.header.time)));
        }

        Ok(())
    }
}

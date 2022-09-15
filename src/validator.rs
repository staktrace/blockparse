//! A module that exposes a block validation API.

use crate::{Block, BlockValidationError, Hash};
use log::info;
use std::collections::HashMap;
use std::fmt;
use std::time::SystemTime;

const MAX_SUPPORTED_BLOCK_VERSION: u32 = 4;
const TWO_HOURS_IN_SECONDS: u64 = 2 * 60 * 60;
const MAX_ACTIVE_HEIGHT: usize = 144; // One day's worth of blocks

/// A state machine to validate blocks as they are received. This structure accepts
/// blocks one at a time, and checks to see if it is valid, updating internal state
/// as necessary. It can handle multiple active chains, such as when competing
/// miners produce different valid blocks for a given block height. It will eventually
/// discard abandoned chains if there is a clear "winner" chain.
#[derive(Default)]
pub struct BlockValidator {
    /// Map from block id to block height for archived blocks. Genesis block is height 0.
    /// Archived blocks are always a linear chain; branches will have been pruned away.
    archived_blocks: HashMap<Hash, usize>,
    /// Map from hash to block and associated metadata for active blocks. Active blocks
    /// are recent blocks that have been validated and connected to the chain. Active
    /// blocks form a tree rooted at the most recent archived block. Generally the longest
    /// path in the tree is the one with the most proof-of-work, and therefore the
    /// canonical blockchain, but that may change. Once the longest path in the active
    /// block tree is longer than MAX_ACTIVE_HEIGHT, the oldest active blocks on that
    /// path are archived and shorter branches emanating from those archived blocks
    /// get pruned away.
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

impl fmt::Debug for ValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            ValidationResult::Valid(h) => write!(f, "ValidationResult::Valid({})", h),
            ValidationResult::Invalid(e) => write!(f, "ValidationResult::Invalid({})", e),
            ValidationResult::Orphan(b) => write!(f, "ValidationResult::Orphan({})", b.id()),
        }
    }
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
        info!("Adding block {} to chain at height {}", hash, height);
        self.active_blocks.insert(hash, active_block);

        if height - self.archived_blocks.len() >= MAX_ACTIVE_HEIGHT {
            self.archive_old_blocks(&hash);
        }

        ValidationResult::Valid(hash)
    }

    fn archive_old_blocks(&mut self, leaf_hash: &Hash) {
        let mut iter_hash = *leaf_hash;
        let mut active_root = iter_hash;
        // Walk up following the parent links such that active_root and iter_hash are
        // separated by the new archiving boundary. active_root will remain active and
        // iter_hash (plus any active ancestors) will get archived.
        for _i in 0..MAX_ACTIVE_HEIGHT {
            active_root = iter_hash;
            iter_hash = self.active_blocks.get(&iter_hash).unwrap().block.header.prev_block_hash;
        }

        // Archive iter_hash and active ancestors until there are no more active ancestors.
        loop {
            iter_hash = match self.active_blocks.remove(&iter_hash) {
                Some(removed) => {
                    info!("Archiving {} with height {}", &iter_hash, removed.height);
                    self.archived_blocks.insert(iter_hash, removed.height);
                    removed.block.header.prev_block_hash
                }
                None => break,
            };
        }

        // Next we want to prune away the dead branches (i.e. any node where following the
        // parent links takes you to an archived node without passing through active_root.
        // We implement this by making a new replacement map, retained_active_blocks, and
        // moving nodes we want to keep into there. Since we seed retained_active_blocks with
        // active_root, the "nodes we want to keep" are simply the ones where walking the
        // parent links takes you to a node already in retained_active_blocks. Everything else
        // is discarded.

        let mut retained_active_blocks = HashMap::new();
        retained_active_blocks.insert(active_root, self.active_blocks.remove(&active_root).unwrap());
        // seeding done, now walk the rest of the active blocks and keep anything in the
        // subtree rooted at active_root.
        let active_block_hashes = self.active_blocks.keys().copied().collect::<Vec<Hash>>();
        for hash in active_block_hashes {
            let root = self.get_active_root(&hash);
            if retained_active_blocks.contains_key(&root) {
                retained_active_blocks.insert(hash, self.active_blocks.remove(&hash).unwrap());
            }
        }

        // Pruning done, now swap our final result back in
        std::mem::swap(&mut self.active_blocks, &mut retained_active_blocks);
    }

    // Returns the leafmost node that is an ancestor of the given hash but that is NOT in
    // the self.active_blocks set.
    fn get_active_root(&self, hash: &Hash) -> Hash {
        let mut root = *hash;
        loop {
            root = match self.active_blocks.get(&root) {
                Some(parent) => parent.block.header.prev_block_hash,
                None => break,
            };
        }
        root
    }

    fn validate_block(&mut self, block: &Block, height: usize) -> Result<(), BlockValidationError> {
        // TODO: implement more things here. This is just enough scaffolding to avoid lint errors
        if block.header.version > MAX_SUPPORTED_BLOCK_VERSION {
            return Err(BlockValidationError::new(format!("Block with unknown version: expected {} but got {}", MAX_SUPPORTED_BLOCK_VERSION, block.header.version)));
        }
        if block.computed_merkle_root() != block.header.merkle_root {
            return Err(BlockValidationError::new(format!("Block with incorrect merkle root: expected {} but got {}", block.computed_merkle_root(), block.header.merkle_root)));
        }
        let seconds_since_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| BlockValidationError::new(String::from("Unable to compute current time relative to the UNIX epoch!")))?
            .as_secs();
        if u64::from(block.header.time) > seconds_since_epoch + TWO_HOURS_IN_SECONDS {
            return Err(BlockValidationError::new(format!("Block timestamp {} was more than two hours in the future from current timestamp {}", block.header.time, seconds_since_epoch)));
        }

        let target = match Hash::from_bits(block.header.bits) {
            None => return Err(BlockValidationError::new(format!("Target difficulty could not be computed from {:#x}", block.header.bits))),
            Some(target) => target,
        };
        // TODO: check against difficulty 1 values (network-dependent) https://developer.bitcoin.org/reference/block_chain.html#target-nbits
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

        if (height % 2016) == 0 {
            // TODO: recompute new difficulty and ensure it matches
        } else if block.header.bits != parent.block.header.bits {
            return Err(BlockValidationError::new(format!("Block changed the difficulty threshold prematurely; height {} is {} mod 2016", height, height % 2016)));
        }

        Ok(())
    }
}

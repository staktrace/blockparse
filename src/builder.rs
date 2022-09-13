//! A high-level module to build a validation pipeline.

use crate::{Block, Hash, LittleEndianSerialization, Network};
use crate::validator::{BlockValidator, ValidationResult};
use log::{trace, warn};
use std::collections::HashSet;
use std::sync::mpsc::{channel, Sender};
use std::thread::{JoinHandle, self};

const ARBITRARY_ORPHANAGE_SIZE: usize = 128;

enum ValidatorMessage {
    NewBlock(Block),
    Shutdown,
}

enum OrphanageMessage {
    NewOrphan(Block),
    NewParent(Hash, Sender<ValidatorMessage>),
    Shutdown,
}

/// The main entry point for the validation pipeline. This struct, when
/// instantiated, sets up the different components needed to go from raw
/// byte arrays (generally obtained via network communication or from
/// files on disk) to a validated blockchain.
pub struct BlockChainBuilder {
    network: Network,
    deduplicator: HashSet<Hash>,
    orphanage_tx: Sender<OrphanageMessage>,
    orphanage_join: JoinHandle<()>,
    validator_tx: Sender<ValidatorMessage>,
    validator_join: JoinHandle<()>,
}

impl BlockChainBuilder {
    /// Create a validation pipeline for the given network.
    pub fn new(network: Network) -> Self {
        let (orphanage_tx, orphanage_join) = Self::spawn_orphanage();
        let (validator_tx, validator_join) = Self::spawn_validator(orphanage_tx.clone());
        BlockChainBuilder {
            network,
            deduplicator: HashSet::new(),
            orphanage_tx,
            orphanage_join,
            validator_tx,
            validator_join,
        }
    }

    fn spawn_orphanage() -> (Sender<OrphanageMessage>, JoinHandle<()>) {
        let (tx, rx) = channel();
        let join_handle = thread::spawn(move|| {
            let mut orphanage = Orphanage::new(ARBITRARY_ORPHANAGE_SIZE);
            loop {
                match rx.recv().unwrap() {
                    OrphanageMessage::NewOrphan(b) => orphanage.take_orphan(b),
                    OrphanageMessage::NewParent(h, validator_tx) => orphanage.find_children(h, validator_tx),
                    OrphanageMessage::Shutdown => break,
                };
            }
        });
        (tx, join_handle)
    }

    fn spawn_validator(orphanage_tx: Sender<OrphanageMessage>) -> (Sender<ValidatorMessage>, JoinHandle<()>) {
        let (tx, rx) = channel();
        let validator_tx = tx.clone();
        let join_handle = thread::spawn(move|| {
            let mut validator = BlockValidator::new();
            while let ValidatorMessage::NewBlock(block) = rx.recv().unwrap() {
                let validation_result = validator.handle_block(block);
                trace!("Validation result: {:?}", &validation_result);
                match validation_result {
                    ValidationResult::Valid(id) => orphanage_tx.send(OrphanageMessage::NewParent(id, validator_tx.clone())).unwrap(),
                    ValidationResult::Invalid(_) => (),
                    ValidationResult::Orphan(b) => orphanage_tx.send(OrphanageMessage::NewOrphan(b)).unwrap(),
                };
            };
        });
        (tx, join_handle)
    }

    /// Feed some data into the validation pipeline. The bytes provided should be one or more
    /// blocks in the standard protocol format (starting with the network magic header).
    /// If multiple blocks are present they are assumed to be concatenated in the byte array
    /// and are parsed as such.
    ///
    /// The returned `usize` is the index at which parsing stopped or was interrupted. If this
    /// is not equal to `bytes.len()` this is likely due to `bytes` containing data that was not
    /// syntactically-valid block data. It may also occur if this function is called after
    /// shutdown.
    ///
    /// Note that blocks that are syntactically valid but are otherwise invalid (e.g. for a
    /// different network, or attempt to spend unspendable outputs) will still be accepted
    /// by this function, but will not end up in the final blockchain.
    pub fn ingest(&mut self, bytes: &[u8]) -> usize {
        let mut ix = 0;
        while ix < bytes.len() {
            let last_good_ix = ix;
            match Block::deserialize_le(bytes, &mut ix) {
                Ok(block) => {
                    if block.network != self.network {
                        continue;
                    }

                    // Note we hash the raw bytes rather than using block.id() because the merkle
                    // root hash isn't perfect (doesn't cover witness data and can be fooled by
                    // duplicating transactions, see https://github.com/bitcoin/bitcoin/blob/0ebd4db32b39cb7c505148f090df4b7ac778c307/src/consensus/merkle.cpp#L8)
                    let bytes_hash = Hash(hmac_sha256::Hash::hash(&bytes[last_good_ix..ix]));
                    if !self.deduplicator.insert(bytes_hash) {
                        // We've already seen this block
                        continue;
                    }
                    if self.validator_tx.send(ValidatorMessage::NewBlock(block)).is_err() {
                        // validator has shut down. handle it gracefully
                        self.deduplicator.remove(&bytes_hash);
                        return last_good_ix;
                    }
                }
                Err(_) => return last_good_ix,
            };
        }
        ix
    }

    /// Perform an orderly shutdown of the various components for this pipeline.
    pub fn shutdown(self) {
        self.validator_tx.send(ValidatorMessage::Shutdown).unwrap();
        self.orphanage_tx.send(OrphanageMessage::Shutdown).unwrap();
        self.validator_join.join().unwrap();
        self.orphanage_join.join().unwrap();
    }
}

/// An orphanage stores blocks that are currently orphans in the hope that they
/// are received out-of-order and can be attached to the chain later. It has a
/// maximum size and evicts entries in FIFO order if they do not get parented.
struct Orphanage {
    size: usize,
    orphans: Vec<Block>,
}

impl Orphanage {
    fn new(size: usize) -> Self {
        Self {
            size,
            orphans: Vec::with_capacity(size),
        }
    }

    /// Store a new orphan in the orphanage, potentially evicting other orphans
    /// if the orphanage is at capacity.
    fn take_orphan(&mut self, block: Block) {
        while self.orphans.len() >= self.size {
            let evicted = self.orphans.remove(0);
            warn!("Orphanage evicting block {}", evicted.id());
        }
        self.orphans.push(block);
    }

    /// Ask the orphanage to find orphans that are children of the given parent,
    /// and send those blocks for validation to the validator. The orphans that
    /// are identified are removed from the orphanage.
    fn find_children(&mut self, parent_id: Hash, validator_tx: Sender<ValidatorMessage>) {
        // TODO: Replace this with self.orphans.drain_filter once that is stable
        let mut i = 0;
        while i < self.orphans.len() {
            if self.orphans[i].header.prev_block_hash == parent_id {
                // The validator shuts down before the orphanage, so make sure not to discard
                // orphans that fail to get sent.
                let child = self.orphans.get(i).unwrap();
                if validator_tx.send(ValidatorMessage::NewBlock(child.clone())).is_ok() {
                    self.orphans.remove(i);
                } else {
                    break;
                }
            } else {
                i += 1;
            }
        }
    }
}

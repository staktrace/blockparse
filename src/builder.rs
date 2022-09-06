use crate::{Block, Hash, LittleEndianSerialization, Network};
use crate::validator::{BlockValidator, ValidationResult};
use std::collections::HashSet;
use std::sync::mpsc::{channel, Sender};
use std::thread;

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

pub struct BlockChainBuilder {
    network: Network,
    deduplicator: HashSet<Hash>,
    orphanage_tx: Sender<OrphanageMessage>,
    validator_tx: Sender<ValidatorMessage>,
}

impl BlockChainBuilder {
    pub fn new(network: Network) -> Self {
        let orphanage_tx = Self::spawn_orphanage();
        let validator_tx = Self::spawn_validator(orphanage_tx.clone());
        BlockChainBuilder {
            network,
            deduplicator: HashSet::new(),
            orphanage_tx,
            validator_tx,
        }
    }

    fn spawn_orphanage() -> Sender<OrphanageMessage> {
        let (tx, rx) = channel();
        let _join_handle = thread::spawn(move|| {
            let mut orphanage = Orphanage::new(ARBITRARY_ORPHANAGE_SIZE);
            loop {
                match rx.recv().unwrap() {
                    OrphanageMessage::NewOrphan(b) => orphanage.take_orphan(b),
                    OrphanageMessage::NewParent(h, validator_tx) => orphanage.find_children(h, validator_tx),
                    OrphanageMessage::Shutdown => break,
                };
            }
        });
        tx
    }

    fn spawn_validator(orphanage_tx: Sender<OrphanageMessage>) -> Sender<ValidatorMessage> {
        let (tx, rx) = channel();
        let validator_tx = tx.clone();
        let _join_handle = thread::spawn(move|| {
            let mut validator = BlockValidator::new();
            while let ValidatorMessage::NewBlock(block) = rx.recv().unwrap() {
                match validator.handle_block(block) {
                    ValidationResult::Valid(id) => orphanage_tx.send(OrphanageMessage::NewParent(id, validator_tx.clone())).unwrap(),
                    ValidationResult::Invalid(_) => (),
                    ValidationResult::Orphan(b) => orphanage_tx.send(OrphanageMessage::NewOrphan(b)).unwrap(),
                };
            };
        });
        tx
    }

    pub fn ingest(&mut self, bytes: &[u8]) -> usize {
        let mut ix = 0;
        while ix < bytes.len() {
            let last_good_ix = ix;
            match Block::deserialize_le(bytes, &mut ix) {
                Ok(block) => {
                    if block.network != self.network {
                        continue;
                    }

                    let bytes_hash = Hash(hmac_sha256::Hash::hash(&bytes[last_good_ix..ix]));
                    if self.deduplicator.insert(bytes_hash) {
                        self.validator_tx.send(ValidatorMessage::NewBlock(block)).unwrap();
                    }
                }
                Err(_) => return last_good_ix,
            };
        }
        ix
    }

    pub fn shutdown(&mut self) {
        self.validator_tx.send(ValidatorMessage::Shutdown).unwrap();
        self.orphanage_tx.send(OrphanageMessage::Shutdown).unwrap();
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
            self.orphans.remove(0);
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

use crate::{Block, Hash, LittleEndianSerialization, Network};
use crate::validator::{BlockValidator, ValidationResult};
use std::collections::HashSet;
use std::sync::mpsc::{channel, Sender};
use std::thread;

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
            let mut orphanage = Orphanage {};
            loop {
                match rx.recv().unwrap() {
                    OrphanageMessage::NewOrphan(b) => orphanage.take_orphan(b),
                    OrphanageMessage::NewParent(h, validator_tx) => orphanage.validate_orphans(h, validator_tx),
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

struct Orphanage {
}

impl Orphanage {
    fn take_orphan(&mut self, _block: Block) {
    }

    fn validate_orphans(&mut self, _parent_id: Hash, _validator_tx: Sender<ValidatorMessage>) {
    }
}

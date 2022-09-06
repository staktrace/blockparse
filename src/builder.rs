use crate::{Block, Hash, LittleEndianSerialization, Network};
use crate::validator::BlockValidator;
use std::collections::HashSet;
use std::sync::mpsc::{channel, Sender};
use std::thread;

enum ValidatorMessage {
    NewBlock(Block),
    Shutdown,
}

pub struct BlockChainBuilder {
    network: Network,
    deduplicator: HashSet<Hash>,
    validator_tx: Sender<ValidatorMessage>,
}

impl Default for BlockChainBuilder {
    fn default() -> Self {
        BlockChainBuilder {
            network: Network::MainNet,
            deduplicator: HashSet::new(),
            validator_tx: Self::spawn_validator(),
        }
    }
}

impl BlockChainBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    fn spawn_validator() -> Sender<ValidatorMessage> {
        let (tx, rx) = channel();
        let _join_handle = thread::spawn(move|| {
            let mut validator = BlockValidator::new();
            while let ValidatorMessage::NewBlock(block) = rx.recv().unwrap() {
                validator.handle_block(block);
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
    }
}

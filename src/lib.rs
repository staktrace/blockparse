#![deny(warnings)]
#![deny(clippy::all)]
#![forbid(unsafe_code)]

#[macro_use]
extern crate bitflags;

use std::fmt;

pub mod parse;

#[derive(Debug)]
pub enum Network {
    MAINNET,
    TESTNET3,
    REGTEST,
}

impl Network {
    fn from(magic: u32) -> Option<Self> {
        match magic {
            0xd9b4bef9 => Some(Network::MAINNET),
            0x0709110b => Some(Network::TESTNET3),
            0xdab5bffa => Some(Network::REGTEST),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Hash([u8; 32]);

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for v in self.0 {
            write!(f, "{:02x}", v)?;
        }
        Ok(())
    }
}

bitflags! {
    pub struct TransactionFlags : u8 {
        const WITNESS = 0x1;
    }
}

#[derive(Debug)]
pub struct TransactionInput {
    pub txid: Hash,
    pub vout: u32,
    pub scriptsig: Vec<u8>,
    pub sequence: u32,
    pub witness_stuff: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct TransactionOutput {
    pub value: u64,
    pub scriptpubkey: Vec<u8>,
}

#[derive(Debug)]
pub struct Transaction {
    pub version: u32,
    pub flags: TransactionFlags,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub locktime: u32,
}

#[derive(Debug)]
pub struct Block {
    pub network: Network,
    pub version: u32,
    pub prev_block_hash: Hash,
    pub merkle_root: Hash,
    pub time: u32,
    pub bits: u32,
    pub nonce: u32,
    pub transactions: Vec<Transaction>,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "time:{} prev:{} merkle:{} bits:{} nonce:{}", self.time, self.prev_block_hash, self.merkle_root, self.bits, self.nonce)
    }
}

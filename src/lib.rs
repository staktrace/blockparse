#![deny(warnings)]
#![deny(clippy::all)]
#![forbid(unsafe_code)]

#[macro_use]
extern crate bitflags;
extern crate hmac_sha256;

pub mod error;
pub mod hash;
pub mod parse;
pub mod script;

pub use error::BlockParseError;

use std::fmt;

pub trait LittleEndianSerialization {
    fn serialize_le(&self, dest: &mut Vec<u8>);
    fn deserialize_le(bytes: &[u8], ix: &mut usize) -> Result<Self, BlockParseError> where Self: Sized;
}

#[derive(Debug)]
pub enum Network {
    MainNet,
    TestNet3,
    RegTest,
}

#[derive(Clone, Copy, Debug)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn zero() -> Self {
        Hash([0; 32])
    }

    pub fn reverse(&self) -> Self {
        let mut hash_bytes = self.0;
        hash_bytes.reverse();
        Hash(hash_bytes)
    }
}

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
pub enum Opcode {
    PushArray(Vec<u8>), // 0x00 - 0x4e
    PushNumber(i8), // 0x4f, 0x51 - 0x60

    Reserved(u8), // 0x50, 0x89 - 0x8a
    Nop(u8), // 0x61, 0xb0, 0xb3 - 0xb9

    Ver, // 0x62
    If, // 0x63
    NotIf, // 0x64
    VerIf, // 0x65
    VerNotIf, // 0x66
    Else, // 0x67
    EndIf, // 0x68
    Verify, // 0x69
    Return, // 0x6a

    ToAltStack, // 0x6b
    FromAltStack, // 0x6c
    Drop2, // 0x6d
    Dup2, // 0x6e
    Dup3, // 0x6f
    Over2, // 0x70
    Rot2, // 0x71
    Swap2, // 0x72
    IfDup, // 0x73
    Depth, // 0x74
    Drop, // 0x75
    Dup, // 0x76
    Nip, // 0x77
    Over, // 0x78
    Pick, // 0x79
    Roll, // 0x7a
    Rot, // 0x7b
    Swap, // 0x7c
    Tuck, // 0x7d

    Cat, // 0x7e, disabled
    Substr, // 0x7f, disabled
    Left, // 0x80, disabled
    Right, // 0x81, disabled
    Size, // 0x82

    Invert, // 0x83, disabled
    And, // 0x84, disabled
    Or, // 0x85, disabled
    Xor, // 0x86, disabled
    Equal, // 0x87
    EqualVerify, // 0x88

    Add1, // 0x8b
    Sub1, // 0x8c
    Mul2, // 0x8d, disabled
    Div2, // 0x8e, disabled
    Negate, // 0x8f
    Abs, // 0x90
    Not, // 0x91
    NotEqual0, // 0x92
    Add, // 0x93
    Sub, // 0x94
    Mul, // 0x95, disabled
    Div, // 0x96, disabled
    Mod, // 0x97, disabled
    LeftShift, // 0x98, disabled
    RightShift, // 0x99, disabled

    BoolAnd, // 0x9a
    BoolOr, // 0x9b
    NumEqual, // 0x9c
    NumEqualVerify, // 0x9d
    NumNotEqual, // 0x9e
    LessThan, // 0x9f
    GreaterThan, // 0xa0
    LessThanOrEqual, // 0xa1
    GreaterThanOrEqual, // 0xa2
    Min, // 0xa3
    Max, // 0xa4
    Within, // 0xa5

    RIPEMD160, // 0xa6
    SHA1, // 0xa7
    SHA256, // 0xa8
    Hash160, // 0xa9
    Hash256, // 0xaa
    CodeSeparator, // 0xab
    CheckSig, // 0xac
    CheckSigVerify, // 0xad
    CheckMultisig, // 0xae
    CheckMultisigVerify, // 0xaf

    CheckLockTimeVerify, // 0xb1
    CheckSequenceVerify, // 0xb2

    Invalid(u8), // 0xba - 0xff
}

#[derive(Debug)]
pub struct Script {
    pub opcodes: Vec<Opcode>,
}

#[derive(Debug)]
pub struct TransactionInput {
    pub txid: Hash,
    pub vout: u32,
    pub unlock_script: Vec<u8>,
    pub sequence: u32,
    pub witness_stuff: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct TransactionOutput {
    pub value: u64,
    pub lock_script: Vec<u8>,
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
pub struct BlockHeader {
    pub version: u32,
    pub prev_block_hash: Hash,
    pub merkle_root: Hash,
    pub time: u32,
    pub bits: u32,
    pub nonce: u32,
}

#[derive(Debug)]
pub struct Block {
    pub network: Network,
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn id(&self) -> Hash {
        hash::double_sha256(&self.header)
    }

    pub fn computed_merkle_root(&self) -> Hash {
        if self.transactions.is_empty() {
            return Hash::zero();
        }

        let adjust_count = |count| {
            match count {
                1 => 1,
                c if (c % 2) == 1 => c + 1,
                c => c,
            }
        };

        let mut layer_size = adjust_count(self.transactions.len());
        let mut layer_hashes = Vec::with_capacity(layer_size);
        for transaction in &self.transactions {
            layer_hashes.push(hash::double_sha256(transaction).reverse());
        }
        while layer_size > layer_hashes.len() {
            layer_hashes.push(*layer_hashes.last().unwrap());
        }

        while layer_size > 1 {
            let next_layer_size = adjust_count(layer_size / 2);
            let mut next_hashes = Vec::with_capacity(next_layer_size);
            for i in 0..next_layer_size {
                let mut next_hash = hmac_sha256::Hash::new();
                next_hash.update(layer_hashes[i * 2].0);
                next_hash.update(layer_hashes[i * 2 + 1].0);
                let first_hash = next_hash.finalize();
                let second_hash = hmac_sha256::Hash::hash(&first_hash);
                next_hashes.push(Hash(second_hash));
            }

            layer_size = next_layer_size;
            layer_hashes = next_hashes;
        }

        layer_hashes.first().unwrap().reverse()
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "time:{} id:{} prev:{} merkle:{} bits:{} nonce:{}", self.header.time, self.id(), self.header.prev_block_hash, self.header.merkle_root, self.header.bits, self.header.nonce)
    }
}

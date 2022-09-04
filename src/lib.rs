#![deny(warnings)]
#![deny(clippy::all)]
#![forbid(unsafe_code)]

#[macro_use]
extern crate bitflags;
extern crate hmac_sha256;

use std::fmt;

pub mod parse;
pub mod script;

pub(crate) trait SerializeLittleEndian {
    fn serialize_le(&self) -> Vec<u8>;
}

#[derive(Debug)]
pub enum Network {
    MainNet,
    TestNet3,
    RegTest,
}

impl Network {
    fn from(magic: u32) -> Option<Self> {
        match magic {
            0xd9b4bef9 => Some(Network::MainNet),
            0x0709110b => Some(Network::TestNet3),
            0xdab5bffa => Some(Network::RegTest),
            _ => None,
        }
    }
}

impl SerializeLittleEndian for Network {
    fn serialize_le(&self) -> Vec<u8> {
        match self {
            Network::MainNet => vec![0xf9, 0xbe, 0xb4, 0xd9],
            Network::TestNet3 => vec![0x0b, 0x11, 0x09, 0x07],
            Network::RegTest => vec![0xfa, 0xbf, 0xb5, 0xda],
        }
    }
}

impl SerializeLittleEndian for u32 {
    fn serialize_le(&self) -> Vec<u8> {
        vec![
            (self & 0xff) as u8,
            ((self >> 8) & 0xff) as u8,
            ((self >> 16) & 0xff) as u8,
            ((self >> 24) & 0xff) as u8,
        ]
    }
}

#[derive(Debug)]
pub struct Hash([u8; 32]);

impl SerializeLittleEndian for Hash {
    fn serialize_le(&self) -> Vec<u8> {
        let mut little_endian = self.0;
        little_endian.reverse();
        little_endian.into()
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

impl Block {
    pub fn hash(&self) -> Hash {
        let mut hasher = hmac_sha256::Hash::new();
        hasher.update(&self.version.serialize_le());
        hasher.update(&self.prev_block_hash.serialize_le());
        hasher.update(&self.merkle_root.serialize_le());
        hasher.update(&self.time.serialize_le());
        hasher.update(&self.bits.serialize_le());
        hasher.update(&self.nonce.serialize_le());
        let first_hash = hasher.finalize();
        let mut second_hash = hmac_sha256::Hash::hash(&first_hash);
        second_hash.reverse();
        Hash(second_hash)
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "time:{} hash:{} prev:{} merkle:{} bits:{} nonce:{}", self.time, self.hash(), self.prev_block_hash, self.merkle_root, self.bits, self.nonce)
    }
}

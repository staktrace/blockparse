#![deny(warnings, missing_docs, clippy::all)]
#![forbid(unsafe_code)]

//! This crate provides a full validation node for the Bitcoin protocol.

pub mod builder;
mod error;
mod hash;
pub mod parse;
mod script;
pub mod validator;

pub use error::{BlockParseError, BlockValidationError, ScriptError};

use bitflags::bitflags;
use std::fmt;

/// Trait implemented by most of the data structures that are part of the
/// network protocol (Block, BlockHeader, etc.). This allows convenient
/// serialization and deserialization from a Rust-friendly data structure
/// to/from the protocol byte format.
pub trait LittleEndianSerialization {
    /// Serializes the object in little-endian format to the given byte
    /// vector. The bytes are appended to the end of the Vec.
    fn serialize_le(&self, dest: &mut Vec<u8>);

    /// Constructs an object given serialized bytes in little-endian format.
    /// This is the reverse operation of the serialize_le function, although
    /// it takes a byte array and an index into the array, and mutates the
    /// index so that it points to whatever is after the serialized object.
    fn deserialize_le(bytes: &[u8], ix: &mut usize) -> Result<Self, BlockParseError> where Self: Sized;
}

/// The network being operated on. This is part of the block header.
#[allow(missing_docs)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Network {
    MainNet,
    TestNet3,
    RegTest,
}

impl Default for Network {
    fn default() -> Self {
        Self::RegTest
    }
}

/// Object representing a SHA256 hash. Contains the raw 32-byte array that
/// is the hash.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd)]
pub struct Hash([u8; 32]);

impl Hash {
    /// Returns a zero hash
    pub fn zero() -> Self {
        Hash([0; 32])
    }

    /// Reverses the byte order of the hash
    pub fn reverse(&self) -> Self {
        let mut hash_bytes = self.0;
        hash_bytes.reverse();
        Hash(hash_bytes)
    }

    /// Computes a target hash given the "bits" value from a block header.
    /// The high byte of `bits` is used as an exponent and the remaining
    /// bytes are used as the coefficient in the target calculation formula.
    /// target = coefficient * 2^(8*(exponent–3)). This function will return
    /// None in case of overflow. Note that the Bitcoin Core reference
    /// implementation is much more strict here about what is allowed.
    pub fn from_bits(bits: u32) -> Option<Self> {
        let mut target_bytes = [0; 32];
        let mut coefficient = bits & 0x00ffffff;
        if coefficient == 0 {
            // No amount of shifting will produce a nonzero or overflow
            // result, so early-exit here.
            return Some(Hash(target_bytes));
        }

        let exponent = bits >> 24;
        if exponent > 34 {
            // This will definitely overflow
            return None;
        }
        let mut ix = (34 - exponent) as usize;
        // loop from the bottom byte of the coefficient to the top, placing them
        // into the hash array.
        for _ in 0..3 {
            let coeff_byte = (coefficient & 0xff) as u8;
            if ix < target_bytes.len() {
                target_bytes[ix] = coeff_byte;
            } // else it's off bottom end of the hash array and we drop it

            // set up for next loop iteration

            coefficient >>= 8;
            if ix == 0 {
                // We're going off the top end of the hash array
                break;
            }
            ix -= 1;
        }

        if coefficient != 0 {
            // We only get here if we exited the loop prematurely due to going
            // off the top end of the array. This means we overflowed nonzero
            // bytes, so fail.
            return None;
        }

        Some(Hash(target_bytes))
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
    #[allow(missing_docs)]
    pub struct TransactionFlags : u8 {
        /// Indicates whether or not this transaction has segregated witness data.
        const WITNESS = 0x1;
    }
}

#[allow(missing_docs)]
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

    Disabled(u8), // 0x7e-0x81, 0x83-0x86, 0x8d-0x8e, 0x95-0x99
    // Cat, // 0x7e, disabled
    // Substr, // 0x7f, disabled
    // Left, // 0x80, disabled
    // Right, // 0x81, disabled
    Size, // 0x82

    // Invert, // 0x83, disabled
    // And, // 0x84, disabled
    // Or, // 0x85, disabled
    // Xor, // 0x86, disabled
    Equal, // 0x87
    EqualVerify, // 0x88

    Add1, // 0x8b
    Sub1, // 0x8c
    // Mul2, // 0x8d, disabled
    // Div2, // 0x8e, disabled
    Negate, // 0x8f
    Abs, // 0x90
    Not, // 0x91
    NotEqual0, // 0x92
    Add, // 0x93
    Sub, // 0x94
    // Mul, // 0x95, disabled
    // Div, // 0x96, disabled
    // Mod, // 0x97, disabled
    // LeftShift, // 0x98, disabled
    // RightShift, // 0x99, disabled

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

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Script {
    pub opcodes: Vec<Opcode>,
}

#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub struct TransactionInput {
    pub txid: Hash,
    pub vout: u32,
    pub unlock_script: Vec<u8>,
    pub sequence: u32,
    pub witness_stuff: Vec<Vec<u8>>,
}

#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub struct TransactionOutput {
    pub value: u64,
    pub lock_script: Vec<u8>,
}

#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub struct Transaction {
    pub version: u32,
    pub flags: TransactionFlags,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub locktime: u32,
}

impl Transaction {
    fn strip_witness_data(&self) -> Transaction {
        Transaction {
            version: self.version,
            flags: TransactionFlags::empty(),
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            locktime: self.locktime,
        }
    }
}

#[allow(missing_docs)]
#[derive(Clone, Debug, Default)]
pub struct BlockHeader {
    pub version: u32,
    pub prev_block_hash: Hash,
    pub merkle_root: Hash,
    pub time: u32,
    pub bits: u32,
    pub nonce: u32,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, Default)]
pub struct Block {
    pub network: Network,
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Computes the block hash, which is a double SHA-256 hash of the block header.
    pub fn id(&self) -> Hash {
        hash::double_sha256(&self.header)
    }

    /// Computes the merkle root of the block by hashing the transactions in a merkle
    /// tree format. Note that this computes the merkle root and doesn't just return
    /// the merkle root from the header.
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
            layer_hashes.push(hash::double_sha256(&transaction.strip_witness_data()).reverse());
        }

        while layer_size > 1 {
            if layer_size > layer_hashes.len() {
                layer_hashes.push(*layer_hashes.last().unwrap());
            }
            assert!(layer_hashes.len() == layer_size);
            assert!((layer_size % 2) == 0);

            let next_layer_size = adjust_count(layer_size / 2);
            let mut next_hashes = Vec::with_capacity(next_layer_size);
            for i in (0..layer_size).step_by(2) {
                let first_hash = hmac_sha256::Hash::hash(&[layer_hashes[i].0, layer_hashes[i + 1].0].concat());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_to_hash() {
        assert_eq!(Hash::from_bits(0x1903a30c).unwrap().to_string(), "0000000000000003a30c00000000000000000000000000000000000000000000");
        assert_eq!(Hash::from_bits(0x1d00ffff).unwrap().to_string(), "00000000ffff0000000000000000000000000000000000000000000000000000");

        assert_eq!(Hash::from_bits(0x00abcdef).unwrap().to_string(), "0000000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(Hash::from_bits(0x01abcdef).unwrap().to_string(), "00000000000000000000000000000000000000000000000000000000000000ab");
        assert_eq!(Hash::from_bits(0x02abcdef).unwrap().to_string(), "000000000000000000000000000000000000000000000000000000000000abcd");
        assert_eq!(Hash::from_bits(0x03abcdef).unwrap().to_string(), "0000000000000000000000000000000000000000000000000000000000abcdef");

        assert_eq!(Hash::from_bits(0x1fabcdef).unwrap().to_string(), "00abcdef00000000000000000000000000000000000000000000000000000000");
        assert_eq!(Hash::from_bits(0x20abcdef).unwrap().to_string(), "abcdef0000000000000000000000000000000000000000000000000000000000");

        assert_eq!(Hash::from_bits(0x21abcdef), None);
        assert_eq!(Hash::from_bits(0x2101cdef), None);
        assert_eq!(Hash::from_bits(0x2100cdef).unwrap().to_string(), "cdef000000000000000000000000000000000000000000000000000000000000");

        assert_eq!(Hash::from_bits(0x2200cdef), None);
        assert_eq!(Hash::from_bits(0x220001ef), None);
        assert_eq!(Hash::from_bits(0x220000ef).unwrap().to_string(), "ef00000000000000000000000000000000000000000000000000000000000000");

        assert_eq!(Hash::from_bits(0x230000ef), None);
        assert_eq!(Hash::from_bits(0x23000001), None);
        assert_eq!(Hash::from_bits(0x23000000).unwrap().to_string(), "0000000000000000000000000000000000000000000000000000000000000000");

        assert_eq!(Hash::from_bits(0xffabcdef), None);
        assert_eq!(Hash::from_bits(0xff000000).unwrap().to_string(), "0000000000000000000000000000000000000000000000000000000000000000");
    }
}

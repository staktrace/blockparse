//! A module that exposes a block parsing API.

use crate::{Block, BlockHeader, BlockParseError, Hash, LittleEndianSerialization, Network, Transaction, TransactionFlags, TransactionInput, TransactionOutput};

impl LittleEndianSerialization for Network {
    fn serialize_le(&self, dest: &mut Vec<u8>) {
        match self {
            Network::MainNet => dest.extend(vec![0xf9, 0xbe, 0xb4, 0xd9]),
            Network::TestNet3 => dest.extend(vec![0x0b, 0x11, 0x09, 0x07]),
            Network::RegTest => dest.extend(vec![0xfa, 0xbf, 0xb5, 0xda]),
        }
    }

    fn deserialize_le(bytes: &[u8], ix: &mut usize) -> Result<Self, BlockParseError> where Self: Sized {
        match u32::deserialize_le(bytes, ix)? {
            0xd9b4bef9 => Ok(Network::MainNet),
            0x0709110b => Ok(Network::TestNet3),
            0xdab5bffa => Ok(Network::RegTest),
            magic => Err(BlockParseError::new(format!("Unrecognized network magic value {:#x} at index {}", magic, *ix - 4))),
        }
    }
}

impl LittleEndianSerialization for u8 {
    fn serialize_le(&self, dest: &mut Vec<u8>) {
        dest.push(*self);
    }

    fn deserialize_le(bytes: &[u8], ix: &mut usize) -> Result<Self, BlockParseError> where Self: Sized {
        if bytes.len() < *ix + 1 {
            return Err(BlockParseError::new(format!("Unexpected end of input reading 1 byte at index {}", *ix)));
        }
        let result = bytes[*ix];
        *ix += 1;
        Ok(result)
    }
}

impl LittleEndianSerialization for u16 {
    fn serialize_le(&self, dest: &mut Vec<u8>) {
        dest.push((self & 0xff) as u8);
        dest.push(((self >> 8) & 0xff) as u8);
    }

    fn deserialize_le(bytes: &[u8], ix: &mut usize) -> Result<Self, BlockParseError> where Self: Sized {
        if bytes.len() < *ix + 2 {
            return Err(BlockParseError::new(format!("Unexpected end of input reading 2 bytes at index {}", *ix)));
        }
        let result = ((bytes[*ix + 1] as u16) << 8)
            | (bytes[*ix] as u16);
        *ix += 2;
        Ok(result)
    }
}

impl LittleEndianSerialization for u32 {
    fn serialize_le(&self, dest: &mut Vec<u8>) {
        dest.push((self & 0xff) as u8);
        dest.push(((self >> 8) & 0xff) as u8);
        dest.push(((self >> 16) & 0xff) as u8);
        dest.push(((self >> 24) & 0xff) as u8);
    }

    fn deserialize_le(bytes: &[u8], ix: &mut usize) -> Result<Self, BlockParseError> where Self: Sized {
        if bytes.len() < *ix + 4 {
            return Err(BlockParseError::new(format!("Unexpected end of input reading 4 bytes at index {}", *ix)));
        }
        let result = ((bytes[*ix + 3] as u32) << 24)
            | ((bytes[*ix + 2] as u32) << 16)
            | ((bytes[*ix + 1] as u32) << 8)
            | (bytes[*ix] as u32);
        *ix += 4;
        Ok(result)
    }
}

impl LittleEndianSerialization for u64 {
    fn serialize_le(&self, dest: &mut Vec<u8>) {
        dest.push((self & 0xff) as u8);
        dest.push(((self >> 8) & 0xff) as u8);
        dest.push(((self >> 16) & 0xff) as u8);
        dest.push(((self >> 24) & 0xff) as u8);
        dest.push(((self >> 32) & 0xff) as u8);
        dest.push(((self >> 40) & 0xff) as u8);
        dest.push(((self >> 48) & 0xff) as u8);
        dest.push(((self >> 56) & 0xff) as u8);
    }

    fn deserialize_le(bytes: &[u8], ix: &mut usize) -> Result<Self, BlockParseError> where Self: Sized {
        if bytes.len() < *ix + 8 {
            return Err(BlockParseError::new(format!("Unexpected end of input reading 8 bytes at index {}", *ix)));
        }
        let result = ((bytes[*ix + 7] as u64) << 56)
            | ((bytes[*ix + 6] as u64) << 48)
            | ((bytes[*ix + 5] as u64) << 40)
            | ((bytes[*ix + 4] as u64) << 32)
            | ((bytes[*ix + 3] as u64) << 24)
            | ((bytes[*ix + 2] as u64) << 16)
            | ((bytes[*ix + 1] as u64) << 8)
            | (bytes[*ix] as u64);
        *ix += 8;
        Ok(result)
    }
}

impl LittleEndianSerialization for usize {
    fn serialize_le(&self, dest: &mut Vec<u8>) {
        if *self <= 0xfc {
            dest.push(*self as u8);
        } else if *self <= 0xffff {
            dest.push(0xfd);
            (*self as u16).serialize_le(dest);
        } else if *self <= 0xffffffff {
            dest.push(0xfe);
            (*self as u32).serialize_le(dest);
        } else {
            dest.push(0xff);
            (*self as u64).serialize_le(dest);
        }
    }

    fn deserialize_le(bytes: &[u8], ix: &mut usize) -> Result<Self, BlockParseError> where Self: Sized {
        match u8::deserialize_le(bytes, ix)? {
            val @ 0..=0xfc => Ok(val as u64),
            0xfd => u16::deserialize_le(bytes, ix).map(|x| x as u64),
            0xfe => u32::deserialize_le(bytes, ix).map(|x| x as u64),
            0xff => u64::deserialize_le(bytes, ix),
        }?.usize()
    }
}

impl LittleEndianSerialization for Hash {
    fn serialize_le(&self, dest: &mut Vec<u8>) {
        dest.extend(self.0.iter().rev());
    }

    fn deserialize_le(bytes: &[u8], ix: &mut usize) -> Result<Self, BlockParseError> where Self: Sized {
        if bytes.len() < *ix + 32 {
            return Err(BlockParseError::new(format!("Unexpected end of input reading 32 bytes at index {}", *ix)));
        }
        let mut hash = [0; 32];
        for i in 0..32 {
            hash[i] = bytes[*ix + 31 - i];
        }
        *ix += 32;
        Ok(Hash(hash))
    }
}

impl LittleEndianSerialization for TransactionFlags {
    fn serialize_le(&self, dest: &mut Vec<u8>) {
        dest.push(self.bits());
    }

    fn deserialize_le(bytes: &[u8], ix: &mut usize) -> Result<Self, BlockParseError> where Self: Sized {
        let b = u8::deserialize_le(bytes, ix)?;
        TransactionFlags::from_bits(b).ok_or_else(|| BlockParseError::new(format!("Unrecognized transaction flags at index {}", *ix - 1)))
    }
}

impl LittleEndianSerialization for Transaction {
    fn serialize_le(&self, dest: &mut Vec<u8>) {
        self.version.serialize_le(dest);
        if !self.flags.is_empty() {
            dest.push(0);
            self.flags.serialize_le(dest);
        }
        self.inputs.len().serialize_le(dest);
        for input in &self.inputs {
            input.txid.serialize_le(dest);
            input.vout.serialize_le(dest);
            input.unlock_script.len().serialize_le(dest);
            dest.extend(&input.unlock_script);
            input.sequence.serialize_le(dest);
        }
        self.outputs.len().serialize_le(dest);
        for output in &self.outputs {
            output.value.serialize_le(dest);
            output.lock_script.len().serialize_le(dest);
            dest.extend(&output.lock_script);
        }
        if self.flags.contains(TransactionFlags::WITNESS) {
            for input in &self.inputs {
                input.witness_stuff.len().serialize_le(dest);
                for witness in &input.witness_stuff {
                    witness.len().serialize_le(dest);
                    dest.extend(witness);
                }
            }
        }
        self.locktime.serialize_le(dest);
    }

    fn deserialize_le(bytes: &[u8], ix: &mut usize) -> Result<Self, BlockParseError> where Self: Sized {
        let version = u32::deserialize_le(bytes, ix)?;
        let count = usize::deserialize_le(bytes, ix)?;
        let (flags, input_count) = if count == 0 /* && allow_witness*/ {
            (TransactionFlags::deserialize_le(bytes, ix)?, usize::deserialize_le(bytes, ix)?)
        } else {
            (TransactionFlags::empty(), count)
        };
        let mut inputs = Vec::with_capacity(input_count);
        for _ in 0..input_count {
            let txid = Hash::deserialize_le(bytes, ix)?;
            let vout = u32::deserialize_le(bytes, ix)?;
            let unlock_script = read_bytearray(bytes, ix)?;
            let sequence = u32::deserialize_le(bytes, ix)?;

            inputs.push(TransactionInput {
                txid,
                vout,
                unlock_script,
                sequence,
                witness_stuff: vec![],
            })
        }
        let output_count = usize::deserialize_le(bytes, ix)?;
        let mut outputs = Vec::with_capacity(output_count);
        for _ in 0..output_count {
            let value = u64::deserialize_le(bytes, ix)?;
            let lock_script = read_bytearray(bytes, ix)?;

            outputs.push(TransactionOutput {
                value,
                lock_script,
            })
        }
        if flags.contains(TransactionFlags::WITNESS) {
            for input in inputs.iter_mut() {
                let outer_count = usize::deserialize_le(bytes, ix)?;
                let mut witness_stuff = Vec::with_capacity(outer_count);
                for _ in 0..outer_count {
                    witness_stuff.push(read_bytearray(bytes, ix)?);
                }
                input.witness_stuff = witness_stuff;
            }
        }
        let locktime = u32::deserialize_le(bytes, ix)?;

        Ok(Transaction {
            version,
            flags,
            inputs,
            outputs,
            locktime,
        })
    }
}

impl LittleEndianSerialization for BlockHeader {
    fn serialize_le(&self, dest: &mut Vec<u8>) {
        self.version.serialize_le(dest);
        self.prev_block_hash.serialize_le(dest);
        self.merkle_root.serialize_le(dest);
        self.time.serialize_le(dest);
        self.bits.serialize_le(dest);
        self.nonce.serialize_le(dest);
    }

    fn deserialize_le(bytes: &[u8], ix: &mut usize) -> Result<Self, BlockParseError> where Self: Sized {
        let version = u32::deserialize_le(bytes, ix)?;
        let prev_block_hash = Hash::deserialize_le(bytes, ix)?;
        let merkle_root = Hash::deserialize_le(bytes, ix)?;
        let time = u32::deserialize_le(bytes, ix)?;
        let bits = u32::deserialize_le(bytes, ix)?;
        let nonce = u32::deserialize_le(bytes, ix)?;

        Ok(BlockHeader {
            version,
            prev_block_hash,
            merkle_root,
            time,
            bits,
            nonce,
        })
    }
}

impl LittleEndianSerialization for Block {
    fn serialize_le(&self, dest: &mut Vec<u8>) {
        self.network.serialize_le(dest);
        let mut size_ix = dest.len();
        0u32.serialize_le(dest);
        self.header.serialize_le(dest);
        self.transactions.len().serialize_le(dest);
        for transaction in &self.transactions {
            transaction.serialize_le(dest);
        }
        let end_ix = dest.len();
        let size = (end_ix - (size_ix + 4)) as u32;

        let mut serialized_size = Vec::with_capacity(4);
        size.serialize_le(&mut serialized_size);
        for b in serialized_size {
            dest[size_ix] = b;
            size_ix += 1;
        }
    }

    fn deserialize_le(bytes: &[u8], ix: &mut usize) -> Result<Self, BlockParseError> where Self: Sized {
        let network = Network::deserialize_le(bytes, ix)?;
        let size = u32::deserialize_le(bytes, ix)?.usize()?;
        let end = *ix + size;

        let header = BlockHeader::deserialize_le(bytes, ix)?;
        let transaction_count = usize::deserialize_le(bytes, ix)?;
        let mut transactions = Vec::with_capacity(transaction_count);
        for _ in 0..transaction_count {
            transactions.push(Transaction::deserialize_le(bytes, ix)?);
        }

        if *ix != end {
            return Err(BlockParseError::new(format!("Unexpected read index after block; expected {} but got {}", end, *ix)));
        }

        Ok(Block {
            network,
            header,
            transactions,
        })
    }
}

pub(crate) fn read_bytes(bytes: &[u8], ix: &mut usize, count: usize) -> Result<Vec<u8>, BlockParseError> {
    let end = *ix + count;
    if bytes.len() < end {
        return Err(BlockParseError::new(format!("Unexpected end of input reading {} bytes at index {}", count, *ix)));
    }

    let mut result = Vec::with_capacity(count);
    result.extend_from_slice(&bytes[*ix..end]);
    *ix = end;
    Ok(result)
}

pub(crate) fn read_bytearray(bytes: &[u8], ix: &mut usize) -> Result<Vec<u8>, BlockParseError> {
    let count = usize::deserialize_le(bytes, ix)?;
    read_bytes(bytes, ix, count)
}

pub(crate) trait IntoUsize {
    fn usize(self) -> Result<usize, BlockParseError>;
}

impl IntoUsize for u64 {
    fn usize(self) -> Result<usize, BlockParseError> {
        // If the count doesn't fit into a usize then maybe this is running on a 32-bit machine or something with a
        // small usize. Maybe we should handle that case? Punting on it for now.
        usize::try_from(self).map_err(|_| BlockParseError::new(format!("Unable to fit value {} into usize", self)))
    }
}

impl IntoUsize for u32 {
    fn usize(self) -> Result<usize, BlockParseError> {
        // If the count doesn't fit into a usize then maybe this is running on something with a
        // small usize. Maybe we should handle that case? Punting on it for now.
        usize::try_from(self).map_err(|_| BlockParseError::new(format!("Unable to fit value {} into usize", self)))
    }
}

impl IntoUsize for u16 {
    fn usize(self) -> Result<usize, BlockParseError> {
        // If the count doesn't fit into a usize then maybe this is running on something with a
        // small usize. Maybe we should handle that case? Punting on it for now.
        usize::try_from(self).map_err(|_| BlockParseError::new(format!("Unable to fit value {} into usize", self)))
    }
}

impl IntoUsize for u8 {
    fn usize(self) -> Result<usize, BlockParseError> {
        // If the count doesn't fit into a usize then maybe this is running on something with a
        // small usize. Maybe we should handle that case? Punting on it for now.
        usize::try_from(self).map_err(|_| BlockParseError::new(format!("Unable to fit value {} into usize", self)))
    }
}

/// Parse raw byte data into a list of blocks. The bytes provided should be one or more
/// blocks in the standard protocol format (starting with the network magic header).
/// If multiple blocks are present they are assumed to be concatenated in the byte array
/// and are parsed as such.
/// If the data is fully parsed into blocks, the list of blocks is returned. Otherwise,
/// a error is returned indicating why parsing failed.
pub fn parse_blockfile(bytes: &[u8]) -> Result<Vec<Block>, BlockParseError> {
    let mut ix = 0;
    let mut blocks = Vec::new();
    while ix < bytes.len() {
        blocks.push(Block::deserialize_le(bytes, &mut ix)?);
    }
    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use super::*;

    fn read_testdata(file: &str) -> Vec<u8> {
        let mut file = File::open(&format!("testdata/{}", file)).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        bytes
    }

    #[test]
    fn test_block_0() {
        let data = read_testdata("block_0.dat");
        let block_0 = parse_blockfile(&data).unwrap().pop().unwrap();
        assert_eq!(block_0.header.merkle_root.to_string(), "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b");
        assert_eq!(block_0.header.merkle_root, block_0.computed_merkle_root());
        assert_eq!(block_0.transactions.len(), 1);
        let mut serialized = Vec::new();
        block_0.serialize_le(&mut serialized);
        assert_eq!(data, serialized);
    }

    #[test]
    fn test_block_481829() {
        let data = read_testdata("block_481829.dat");
        let block_481829 = parse_blockfile(&data).unwrap().pop().unwrap();
        assert_eq!(block_481829.header.merkle_root.to_string(), "f06f697be2cac7af7ed8cd0b0b81eaa1a39e444c6ebd3697e35ab34461b6c58d");
        assert_eq!(block_481829.header.merkle_root, block_481829.computed_merkle_root());
        assert_eq!(block_481829.transactions.len(), 2020);
        let mut serialized = Vec::new();
        block_481829.serialize_le(&mut serialized);
        assert_eq!(data, serialized);
    }

    #[test]
    fn test_block_265458() {
        let data = read_testdata("block_265458.dat");
        let block_265458 = parse_blockfile(&data).unwrap().pop().unwrap();
        assert_eq!(block_265458.header.merkle_root.to_string(), "501174c68520c1d23bea38774b2dac1d26d4a6c34daef6638762731e78ab1c06");
        assert_eq!(block_265458.header.merkle_root, block_265458.computed_merkle_root());
        assert_eq!(block_265458.transactions.len(), 320);
        let mut serialized = Vec::new();
        block_265458.serialize_le(&mut serialized);
        assert_eq!(data, serialized);
    }
}

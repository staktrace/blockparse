use std::fmt;
use crate::{Block, Hash, Network, Transaction, TransactionFlags, TransactionInput, TransactionOutput};

#[derive(Debug, PartialEq)]
pub struct BlockParseError {
    msg: String,
}

impl BlockParseError {
    pub(crate) fn new(msg: String) -> Self {
        BlockParseError {
            msg,
        }
    }
}

impl fmt::Display for BlockParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for BlockParseError {
}

fn read_2le(bytes: &[u8], ix: &mut usize) -> Result<u16, BlockParseError> {
    if bytes.len() < *ix + 2 {
        return Err(BlockParseError::new(format!("Unexpected end of input reading 2 bytes at index {}", *ix)));
    }
    let result = ((bytes[*ix + 1] as u16) << 8)
        | (bytes[*ix] as u16);
    *ix += 2;
    Ok(result)
}

fn read_4le(bytes: &[u8], ix: &mut usize) -> Result<u32, BlockParseError> {
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

fn read_8le(bytes: &[u8], ix: &mut usize) -> Result<u64, BlockParseError> {
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

fn read_hash_le(bytes: &[u8], ix: &mut usize) -> Result<Hash, BlockParseError> {
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

fn read_compact_size(bytes: &[u8], ix: &mut usize) -> Result<u64, BlockParseError> {
    if bytes.len() < *ix + 1 {
        return Err(BlockParseError::new(format!("Unexpected end of input reading 1 byte at index {}", *ix)));
    }
    *ix += 1;
    match bytes[*ix - 1] {
        val @ 0..=0xfc => Ok(val as u64),
        0xfd => read_2le(bytes, ix).map(|x| x as u64),
        0xfe => read_4le(bytes, ix).map(|x| x as u64),
        0xff => read_8le(bytes, ix),
    }
}

fn read_txflags(bytes: &[u8], ix: &mut usize) -> Result<TransactionFlags, BlockParseError> {
    if bytes.len() < *ix + 1 {
        return Err(BlockParseError::new(format!("Unexpected end of input reading 1 byte at index {}", *ix)));
    }
    *ix += 1;
    TransactionFlags::from_bits(bytes[*ix - 1]).ok_or_else(|| BlockParseError::new(format!("Unrecognized transaction flags at index {}", *ix - 1)))
}

trait IntoUsize {
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
        // If the count doesn't fit into a usize then maybe this is running on a 32-bit machine or something with a
        // small usize. Maybe we should handle that case? Punting on it for now.
        usize::try_from(self).map_err(|_| BlockParseError::new(format!("Unable to fit value {} into usize", self)))
    }
}

fn read_bytearray(bytes: &[u8], ix: &mut usize) -> Result<Vec<u8>, BlockParseError> {
    let count = read_compact_size(bytes, ix)?.usize()?;
    let end = *ix + count;

    if bytes.len() < end {
        return Err(BlockParseError::new(format!("Unexpected end of input reading {} bytes at index {}", count, *ix)));
    }
    let mut result = Vec::with_capacity(count);
    result.extend_from_slice(&bytes[*ix..end]);
    *ix = end;
    Ok(result)
}

pub fn parse_blockfile(raw_data: &[u8]) -> Result<Vec<Block>, BlockParseError> {
    let mut ix = 0;
    let mut blocks = Vec::new();
    while ix < raw_data.len() {
        blocks.push(parse_block(raw_data, &mut ix)?);
    }
    Ok(blocks)
}

pub fn parse_block(raw_data: &[u8], ix: &mut usize) -> Result<Block, BlockParseError> {
    let magic = read_4le(raw_data, ix)?;
    let network = Network::from(magic).ok_or_else(|| BlockParseError::new(format!("Unrecognized network magic value {:#x} at index {}", magic, *ix - 4)))?;
    let size = read_4le(raw_data, ix)?.usize()?;
    let end = *ix + size;
    let version = read_4le(raw_data, ix)?;
    let prev_block_hash = read_hash_le(raw_data, ix)?;
    let merkle_root = read_hash_le(raw_data, ix)?;
    let time = read_4le(raw_data, ix)?;
    let bits = read_4le(raw_data, ix)?;
    let nonce = read_4le(raw_data, ix)?;

    let transaction_count = read_compact_size(raw_data, ix)?.usize()?;
    let mut transactions = Vec::with_capacity(transaction_count);
    for _ in 0..transaction_count {
        transactions.push(parse_transaction(raw_data, ix)?);
    }

    if *ix != end {
        return Err(BlockParseError::new(format!("Unexpected read index after block; expected {} but got {}", end, *ix)));
    }

    Ok(Block {
        network,
        version,
        prev_block_hash,
        merkle_root,
        time,
        bits,
        nonce,
        transactions,
    })
}

pub fn parse_transaction(raw_data: &[u8], ix: &mut usize) -> Result<Transaction, BlockParseError> {
    let version = read_4le(raw_data, ix)?;
    let count = read_compact_size(raw_data, ix)?.usize()?;
    let (flags, input_count) = if count == 0 /* && allow_witness*/ {
        (read_txflags(raw_data, ix)?, read_compact_size(raw_data, ix)?.usize()?)
    } else {
        (TransactionFlags::empty(), count)
    };
    let mut inputs = Vec::with_capacity(input_count);
    for _ in 0..input_count {
        inputs.push(parse_transaction_input(raw_data, ix)?);
    }
    let output_count = read_compact_size(raw_data, ix)?.usize()?;
    let mut outputs = Vec::with_capacity(output_count);
    for _ in 0..output_count {
        outputs.push(parse_transaction_output(raw_data, ix)?);
    }
    if flags.contains(TransactionFlags::WITNESS) {
        for input in inputs.iter_mut() {
            let outer_count = read_compact_size(raw_data, ix)?.usize()?;
            let mut witness_stuff = Vec::with_capacity(outer_count);
            for _ in 0..outer_count {
                witness_stuff.push(read_bytearray(raw_data, ix)?);
            }
            input.witness_stuff = witness_stuff;
        }
    }
    let locktime = read_4le(raw_data, ix)?;

    Ok(Transaction {
        version,
        flags,
        inputs,
        outputs,
        locktime,
    })
}

pub fn parse_transaction_input(raw_data: &[u8], ix: &mut usize) -> Result<TransactionInput, BlockParseError> {
    let txid = read_hash_le(raw_data, ix)?;
    let vout = read_4le(raw_data, ix)?;
    let scriptsig = read_bytearray(raw_data, ix)?;
    let sequence = read_4le(raw_data, ix)?;

    Ok(TransactionInput {
        txid,
        vout,
        scriptsig,
        sequence,
        witness_stuff: vec![],
    })
}

pub fn parse_transaction_output(raw_data: &[u8], ix: &mut usize) -> Result<TransactionOutput, BlockParseError> {
    let value = read_8le(raw_data, ix)?;
    let scriptpubkey = read_bytearray(raw_data, ix)?;

    Ok(TransactionOutput {
        value,
        scriptpubkey,
    })
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
    fn test_parsing() {
        let block_0 = parse_blockfile(&read_testdata("block_0.dat")).unwrap().pop().unwrap();
        assert_eq!(block_0.merkle_root.to_string(), "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b");
        assert_eq!(block_0.transactions.len(), 1);

        let block_481829 = parse_blockfile(&read_testdata("block_481829.dat")).unwrap().pop().unwrap();
        assert_eq!(block_481829.merkle_root.to_string(), "f06f697be2cac7af7ed8cd0b0b81eaa1a39e444c6ebd3697e35ab34461b6c58d");
        assert_eq!(block_481829.transactions.len(), 2020);
    }
}

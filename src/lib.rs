use std::fmt;

pub mod error;

pub use crate::error::BlockParseError;

pub enum Network {
    MAINNET,
    TESTNET3,
    REGTEST,
}

impl Network {
    fn magic(&self) -> u32 {
        match self {
            Network::MAINNET => 0xd9b4bef9,
            Network::TESTNET3 => 0x0709110b,
            Network::REGTEST => 0xdab5bffa,
        }
    }
}

pub type Hash = [u8; 32];

#[derive(Debug)]
pub struct TransactionInput {
    pub txid: [u8; 32],
    pub vout: u32,
    pub scriptsig: Vec<u8>,
    pub sequence: u32,
}

#[derive(Debug)]
pub struct TransactionOutput {
    pub value: u64,
    pub scriptpubkey: Vec<u8>,
}

#[derive(Debug)]
pub struct Transaction {
    pub version: u32,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub locktime: u32,
}

#[derive(Debug)]
pub struct Block {
    pub version: u32,
    pub prev_block_hash: Hash,
    pub merkle_root: Hash,
    pub time: u32,
    pub bits: u32,
    pub nonce: u32,
    pub transactions: Vec<Transaction>,
}

fn write_hash(f: &mut fmt::Formatter<'_>, hash: &Hash) -> fmt::Result {
    for v in hash {
        write!(f, "{:02x}", v)?;
    }
    Ok(())
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "time:{} prev:", self.time)?;
        write_hash(f, &self.prev_block_hash)?;
        write!(f, " merkle:")?;
        write_hash(f, &self.merkle_root)?;
        write!(f, " bits:{} nonce:{}", self.bits, self.nonce)
    }
}

pub(crate) fn read_2le(bytes: &[u8], ix: &mut usize) -> Result<u16, BlockParseError> {
    if bytes.len() < *ix + 2 {
        return Err(BlockParseError::new(format!("Unexpected end of input reading 2 bytes at index {}", *ix)));
    }
    let result = ((bytes[*ix + 1] as u16) << 8)
        | (bytes[*ix + 0] as u16);
    *ix += 2;
    Ok(result)
}

pub(crate) fn read_4le(bytes: &[u8], ix: &mut usize) -> Result<u32, BlockParseError> {
    if bytes.len() < *ix + 4 {
        return Err(BlockParseError::new(format!("Unexpected end of input reading 4 bytes at index {}", *ix)));
    }
    let result = ((bytes[*ix + 3] as u32) << 24)
        | ((bytes[*ix + 2] as u32) << 16)
        | ((bytes[*ix + 1] as u32) << 8)
        | (bytes[*ix + 0] as u32);
    *ix += 4;
    Ok(result)
}

pub(crate) fn read_8le(bytes: &[u8], ix: &mut usize) -> Result<u64, BlockParseError> {
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
        | (bytes[*ix + 0] as u64);
    *ix += 8;
    Ok(result)
}

pub(crate) fn read_hash_le(bytes: &[u8], ix: &mut usize) -> Result<Hash, BlockParseError> {
    if bytes.len() < *ix + 32 {
        return Err(BlockParseError::new(format!("Unexpected end of input reading 32 bytes at index {}", *ix)));
    }
    let mut hash = [0; 32];
    for i in 0..32 {
        hash[i] = bytes[*ix + 31 - i];
    }
    *ix += 32;
    Ok(hash)
}

pub(crate) fn read_varint(bytes: &[u8], ix: &mut usize) -> Result<u64, BlockParseError> {
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

pub(crate) fn read_var(bytes: &[u8], ix: &mut usize, count_u64: u64) -> Result<Vec<u8>, BlockParseError> {
    let count = count_u64.usize()?;
    let end = *ix + count;

    if bytes.len() < end {
        return Err(BlockParseError::new(format!("Unexpected end of input reading {} bytes at index {}", count, *ix)));
    }
    let mut result = Vec::with_capacity(count);
    result.extend_from_slice(&bytes[*ix..end]);
    *ix = end;
    Ok(result)
}

pub fn parse_blockfile(raw_data: &[u8], expected_network: Option<Network>) -> Result<Vec<Block>, BlockParseError> {
    let mut ix = 0;
    let mut blocks = Vec::new();
    while ix < raw_data.len() {
        let magic = read_4le(raw_data, &mut ix)?;
        if let Some(ref network) = expected_network {
            if magic != network.magic() {
                return Err(BlockParseError::new(format!("Incorrect magic header; expected {:#x} but got {:#x}", network.magic(), magic)))
            }
        }

        let size = read_4le(raw_data, &mut ix)?.usize()?;
        let end = ix + size;
        blocks.push(parse_block(raw_data, &mut ix)?);
        if ix != end {
            return Err(BlockParseError::new(format!("Unexpected read index after block {}; expected {} but got {}", blocks.len(), end, ix)));
        }
    }
    Ok(blocks)
}

pub fn parse_block(raw_data: &[u8], ix: &mut usize) -> Result<Block, BlockParseError> {
    let version = read_4le(raw_data, ix)?;
    let prev_block_hash = read_hash_le(raw_data, ix)?;
    let merkle_root = read_hash_le(raw_data, ix)?;
    let time = read_4le(raw_data, ix)?;
    let bits = read_4le(raw_data, ix)?;
    let nonce = read_4le(raw_data, ix)?;

    let transaction_count = read_varint(raw_data, ix)?.usize()?;
    let mut transactions = Vec::with_capacity(transaction_count);
    for _ in 0..transaction_count {
        transactions.push(parse_transaction(raw_data, ix)?);
    }

    Ok(Block {
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
    let input_count = read_varint(raw_data, ix)?.usize()?;
    let mut inputs = Vec::with_capacity(input_count);
    for _ in 0..input_count {
        inputs.push(parse_transaction_input(raw_data, ix)?);
    }
    let output_count = read_varint(raw_data, ix)?.usize()?;
    let mut outputs = Vec::with_capacity(output_count);
    for _ in 0..output_count {
        outputs.push(parse_transaction_output(raw_data, ix)?);
    }
    let locktime = read_4le(raw_data, ix)?;

    Ok(Transaction {
        version,
        inputs,
        outputs,
        locktime,
    })
}

pub fn parse_transaction_input(raw_data: &[u8], ix: &mut usize) -> Result<TransactionInput, BlockParseError> {
    let txid = read_hash_le(raw_data, ix)?;
    let vout = read_4le(raw_data, ix)?;
    let scriptsig_size = read_varint(raw_data, ix)?;
    let scriptsig = read_var(raw_data, ix, scriptsig_size)?;
    let sequence = read_4le(raw_data, ix)?;

    Ok(TransactionInput {
        txid,
        vout,
        scriptsig,
        sequence,
    })
}

pub fn parse_transaction_output(raw_data: &[u8], ix: &mut usize) -> Result<TransactionOutput, BlockParseError> {
    let value = read_8le(raw_data, ix)?;
    let scriptpubkey_size = read_varint(raw_data, ix)?;
    let scriptpubkey = read_var(raw_data, ix, scriptpubkey_size)?;

    Ok(TransactionOutput {
        value,
        scriptpubkey,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

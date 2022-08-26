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

#[derive(Debug)]
pub struct Block {
}

pub(crate) fn read4le(bytes: &[u8], ix: &mut usize) -> Result<u32, BlockParseError> {
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

pub fn parse_blockfile(raw_data: &[u8], expected_network: Option<Network>) -> Result<Vec<Block>, BlockParseError> {
    let mut ix = 0;
    let magic = read4le(raw_data, &mut ix)?;
    if let Some(network) = expected_network {
        if magic != network.magic() {
            return Err(BlockParseError::new(format!("Incorrect magic header; expected {:#x} but got {:#x}", network.magic(), magic)))
        }
    }
    let mut blocks = Vec::new();
    while ix < raw_data.len() {
        blocks.push(parse_block(raw_data, &mut ix)?);
    }
    Ok(blocks)
}

pub fn parse_block(raw_data: &[u8], ix: &mut usize) -> Result<Block, BlockParseError> {
    *ix = raw_data.len();
    Ok(Block {})
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

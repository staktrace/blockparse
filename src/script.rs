use crate::{BlockParseError, LittleEndianSerialization, Opcode, Script};
use crate::parse::{read_bytes, IntoUsize};

impl LittleEndianSerialization for Opcode {
    fn serialize_le(&self, _dest: &mut Vec<u8>) {
        unimplemented!("Will implement this once I have script validation done to lock down the Opcode enum");
    }

    fn deserialize_le(bytes: &[u8], ix: &mut usize) -> Result<Self, BlockParseError> where Self: Sized {
        match u8::deserialize_le(bytes, ix)? {
            v @ 0x00..=0x4b => Ok(Opcode::PushArray(read_bytes(bytes, ix, v.usize()?)?)),
            0x4c => {
                let count = u8::deserialize_le(bytes, ix)?.usize()?;
                Ok(Opcode::PushArray(read_bytes(bytes, ix, count)?))
            }
            0x4d => {
                let count = u16::deserialize_le(bytes, ix)?.usize()?;
                Ok(Opcode::PushArray(read_bytes(bytes, ix, count)?))
            }
            0x4e => {
                let count = u32::deserialize_le(bytes, ix)?.usize()?;
                Ok(Opcode::PushArray(read_bytes(bytes, ix, count)?))
            }
            v @ 0x4f => Ok(Opcode::PushNumber(v as i8 - 0x50)),
            v @ 0x50 => Ok(Opcode::Reserved(v)),
            v @ 0x51..=0x60 => Ok(Opcode::PushNumber(v as i8 - 0x50)),
            v @ 0x61 => Ok(Opcode::Nop(v)),
            0x62 => Ok(Opcode::Ver),
            0x63 => Ok(Opcode::If),
            0x64 => Ok(Opcode::NotIf),
            0x65 => Ok(Opcode::VerIf),
            0x66 => Ok(Opcode::VerNotIf),
            0x67 => Ok(Opcode::Else),
            0x68 => Ok(Opcode::EndIf),
            0x69 => Ok(Opcode::Verify),
            0x6a => Ok(Opcode::Return),
            0x6b => Ok(Opcode::ToAltStack),
            0x6c => Ok(Opcode::FromAltStack),
            0x6d => Ok(Opcode::Drop2),
            0x6e => Ok(Opcode::Dup2),
            0x6f => Ok(Opcode::Dup3),
            0x70 => Ok(Opcode::Over2),
            0x71 => Ok(Opcode::Rot2),
            0x72 => Ok(Opcode::Swap2),
            0x73 => Ok(Opcode::IfDup),
            0x74 => Ok(Opcode::Depth),
            0x75 => Ok(Opcode::Drop),
            0x76 => Ok(Opcode::Dup),
            0x77 => Ok(Opcode::Nip),
            0x78 => Ok(Opcode::Over),
            0x79 => Ok(Opcode::Pick),
            0x7a => Ok(Opcode::Roll),
            0x7b => Ok(Opcode::Rot),
            0x7c => Ok(Opcode::Swap),
            0x7d => Ok(Opcode::Tuck),
            v @ 0x7e..=0x81 => Ok(Opcode::Disabled(v)),
            0x82 => Ok(Opcode::Size),
            v @ 0x83..=0x86 => Ok(Opcode::Disabled(v)),
            0x87 => Ok(Opcode::Equal),
            0x88 => Ok(Opcode::EqualVerify),
            v @ 0x89..=0x8a => Ok(Opcode::Reserved(v)),
            0x8b => Ok(Opcode::Add1),
            0x8c => Ok(Opcode::Sub1),
            v @ 0x8d..=0x8e => Ok(Opcode::Disabled(v)),
            0x8f => Ok(Opcode::Negate),
            0x90 => Ok(Opcode::Abs),
            0x91 => Ok(Opcode::Not),
            0x92 => Ok(Opcode::NotEqual0),
            0x93 => Ok(Opcode::Add),
            0x94 => Ok(Opcode::Sub),
            v @ 0x95..=0x99 => Ok(Opcode::Disabled(v)),
            0x9a => Ok(Opcode::BoolAnd),
            0x9b => Ok(Opcode::BoolOr),
            0x9c => Ok(Opcode::NumEqual),
            0x9d => Ok(Opcode::NumEqualVerify),
            0x9e => Ok(Opcode::NumNotEqual),
            0x9f => Ok(Opcode::LessThan),
            0xa0 => Ok(Opcode::GreaterThan),
            0xa1 => Ok(Opcode::LessThanOrEqual),
            0xa2 => Ok(Opcode::GreaterThanOrEqual),
            0xa3 => Ok(Opcode::Min),
            0xa4 => Ok(Opcode::Max),
            0xa5 => Ok(Opcode::Within),
            0xa6 => Ok(Opcode::RIPEMD160),
            0xa7 => Ok(Opcode::SHA1),
            0xa8 => Ok(Opcode::SHA256),
            0xa9 => Ok(Opcode::Hash160),
            0xaa => Ok(Opcode::Hash256),
            0xab => Ok(Opcode::CodeSeparator),
            0xac => Ok(Opcode::CheckSig),
            0xad => Ok(Opcode::CheckSigVerify),
            0xae => Ok(Opcode::CheckMultisig),
            0xaf => Ok(Opcode::CheckMultisigVerify),
            v @ 0xb0 => Ok(Opcode::Nop(v)),
            0xb1 => Ok(Opcode::CheckLockTimeVerify),
            0xb2 => Ok(Opcode::CheckSequenceVerify),
            v @ 0xb3..=0xb9 => Ok(Opcode::Nop(v)),
            v @ 0xba..=0xff => Ok(Opcode::Invalid(v)),
        }
    }
}

#[allow(unused)]
pub fn parse_script(bytes: &[u8]) -> Result<Script, BlockParseError> {
    let mut opcodes = Vec::new();

    let mut ix = 0;
    while ix < bytes.len() {
        opcodes.push(read_opcode(bytes, &mut ix)?);
    }
    assert!(ix == bytes.len(), "The last call to read_opcode should have returned an error");
    Ok(Script {
        opcodes,
    })
}

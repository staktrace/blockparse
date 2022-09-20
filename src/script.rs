//! A module that exposes a script parsing and verification API.

use crate::{BlockParseError, BlockValidationError, LittleEndianSerialization, Opcode, Script, ScriptError};
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
            v @ 0x62 => Ok(Opcode::Reserved(v)),
            0x63 => Ok(Opcode::If),
            0x64 => Ok(Opcode::NotIf),
            v @ 0x65..=0x66 => Ok(Opcode::Disabled(v)),
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

/// Parses the given script from raw bytes into a list of opcodes encapsulated
/// in the Script structure. Note that this only does structural/syntax checking,
/// and allows invalid opcodes to be in the returned Script.
pub fn parse_script(bytes: &[u8]) -> Result<Script, BlockParseError> {
    let mut opcodes = Vec::new();

    let mut ix = 0;
    while ix < bytes.len() {
        opcodes.push(Opcode::deserialize_le(bytes, &mut ix)?);
    }
    assert!(ix == bytes.len(), "The last call to read_opcode should have returned an error");
    Ok(Script {
        opcodes,
    })
}

impl Script {
    fn validate(self) -> Result<Self, BlockValidationError> {
        for opcode in &self.opcodes {
            if let Opcode::Invalid(op) = opcode {
                return Err(BlockValidationError::new(format!("Invalid opcode {} found in script", op)));
            }
        }
        Ok(self)
    }
}

#[derive(Clone)]
enum StackEntry {
    Bytes(Vec<u8>),
    Number(i64),
}

impl StackEntry {
    fn as_bool(&self) -> bool {
        match self {
            StackEntry::Bytes(v) => !v.is_empty(),  // TODO also check for zero/negative zero bytes?
            StackEntry::Number(v) => *v != 0,
        }
    }
}

struct Executor {
    stack: Vec<StackEntry>,
    alt_stack: Vec<StackEntry>,
}

fn empty_err() -> BlockValidationError {
    BlockValidationError::new(String::from("Stack is empty when attempting to read item"))
}

impl Executor {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            alt_stack: Vec::new(),
        }
    }

    fn top_bool(&mut self) -> Result<bool, BlockValidationError> {
        let as_bool = match self.stack.pop() {
            None => return Err(empty_err()),
            Some(e) => e.as_bool(),
        };
        Ok(as_bool)
    }

    fn execute(&mut self, script: Script) -> Result<(), BlockValidationError> {
        for opcode in script.opcodes {
            match opcode {
                Opcode::PushArray(v) => self.stack.push(StackEntry::Bytes(v)),
                Opcode::PushNumber(v) => self.stack.push(StackEntry::Number(v.into())),

                Opcode::Reserved(op) => return Err(BlockValidationError::new(format!("Unexpected reserved opcode {}", op))),
                Opcode::Disabled(op) => return Err(BlockValidationError::new(format!("Unexpected disabled opcode {}", op))),
                Opcode::Invalid(_) => panic!("Invalid opcodes should have already gotten filtered out"),
                Opcode::Nop(_) => (),
/*
    TODO
    Opcode::If, // 0x63
    Opcode::NotIf, // 0x64
    Opcode::Else, // 0x67
    Opcode::EndIf, // 0x68
*/

                Opcode::Verify => {
                    if !self.top_bool()? {
                        return Err(BlockValidationError::new(String::from("Top stack entry evaluted to false for VERIFY opcode")));
                    }
                }
                Opcode::Return => return Err(BlockValidationError::new(String::from("Encountered RETURN opcode"))),

                Opcode::ToAltStack => self.alt_stack.push(self.stack.pop().ok_or_else(empty_err)?),
                Opcode::FromAltStack => self.stack.push(self.alt_stack.pop().ok_or_else(empty_err)?),
                Opcode::Drop2 => {
                    if self.stack.len() < 2 {
                        return Err(empty_err());
                    }
                    self.stack.pop();
                    self.stack.pop();
                }
                Opcode::Dup2 => {
                    if self.stack.len() < 2 {
                        return Err(empty_err());
                    }
                    self.stack.push(self.stack[self.stack.len() - 2].clone());
                    self.stack.push(self.stack[self.stack.len() - 2].clone());
                }
                Opcode::Dup3 => {
                    if self.stack.len() < 3 {
                        return Err(empty_err());
                    }
                    self.stack.push(self.stack[self.stack.len() - 3].clone());
                    self.stack.push(self.stack[self.stack.len() - 3].clone());
                    self.stack.push(self.stack[self.stack.len() - 3].clone());
                }
                Opcode::Over2 => {
                    if self.stack.len() < 4 {
                        return Err(empty_err());
                    }
                    self.stack.push(self.stack[self.stack.len() - 4].clone());
                    self.stack.push(self.stack[self.stack.len() - 4].clone());
                }
                Opcode::Rot2 => {
                    if self.stack.len() < 6 {
                        return Err(empty_err());
                    }
                    let removed = self.stack.remove(self.stack.len() - 6);
                    self.stack.push(removed);
                    let removed = self.stack.remove(self.stack.len() - 6);
                    self.stack.push(removed);
                }
                Opcode::Swap2 => {
                    if self.stack.len() < 4 {
                        return Err(empty_err());
                    }
                    let removed = self.stack.remove(self.stack.len() - 4);
                    self.stack.push(removed);
                    let removed = self.stack.remove(self.stack.len() - 4);
                    self.stack.push(removed);
                }
                Opcode::IfDup => {
                    if self.stack.is_empty() {
                        return Err(empty_err());
                    }
                    if self.stack[self.stack.len() - 1].as_bool() {
                        self.stack.push(self.stack[self.stack.len() - 1].clone());
                    }
                }
                Opcode::Depth => {
                    let size = i64::try_from(self.stack.len()).map_err(|_| BlockValidationError::new(format!("Stack size {} is too large for i64", self.stack.len())))?;
                    self.stack.push(StackEntry::Number(size));
                }
/*
    TODO
    Opcode::Drop, // 0x75
    Opcode::Dup, // 0x76
    Opcode::Nip, // 0x77
    Opcode::Over, // 0x78
    Opcode::Pick, // 0x79
    Opcode::Roll, // 0x7a
    Opcode::Rot, // 0x7b
    Opcode::Swap, // 0x7c
    Opcode::Tuck, // 0x7d

    Opcode::Size, // 0x82

    Opcode::Equal, // 0x87
    Opcode::EqualVerify, // 0x88

    Opcode::Add1, // 0x8b
    Opcode::Sub1, // 0x8c
    Opcode::Negate, // 0x8f
    Opcode::Abs, // 0x90
    Opcode::Not, // 0x91
    Opcode::NotEqual0, // 0x92
    Opcode::Add, // 0x93
    Opcode::Sub, // 0x94

    Opcode::BoolAnd, // 0x9a
    Opcode::BoolOr, // 0x9b
    Opcode::NumEqual, // 0x9c
    Opcode::NumEqualVerify, // 0x9d
    Opcode::NumNotEqual, // 0x9e
    Opcode::LessThan, // 0x9f
    Opcode::GreaterThan, // 0xa0
    Opcode::LessThanOrEqual, // 0xa1
    Opcode::GreaterThanOrEqual, // 0xa2
    Opcode::Min, // 0xa3
    Opcode::Max, // 0xa4
    Opcode::Within, // 0xa5

    Opcode::RIPEMD160, // 0xa6
    Opcode::SHA1, // 0xa7
    Opcode::SHA256, // 0xa8
    Opcode::Hash160, // 0xa9
    Opcode::Hash256, // 0xaa
    Opcode::CodeSeparator, // 0xab
    Opcode::CheckSig, // 0xac
    Opcode::CheckSigVerify, // 0xad
    Opcode::CheckMultisig, // 0xae
    Opcode::CheckMultisigVerify, // 0xaf

    Opcode::CheckLockTimeVerify, // 0xb1
    Opcode::CheckSequenceVerify, // 0xb2
*/
                _ => (),
            }
        }
        Ok(())
    }
}

/// Verifies the given lock and unlock scripts. This does the three steps of
/// script parsing (fails if syntax is incorrect), script validation (fails
/// if invalid opcodes are used), and script verification (runs the scripts
/// and ensures that the unlock script correctly unlocks the output from the
/// lock script).
pub fn verify(lock: &[u8], unlock: &[u8]) -> Result<bool, ScriptError> {
    let lock = parse_script(lock).map_err(ScriptError::Parse)?.validate().map_err(ScriptError::Validation)?;
    let unlock = parse_script(unlock).map_err(ScriptError::Parse)?.validate().map_err(ScriptError::Validation)?;

    let mut executor = Executor::new();
    executor.execute(unlock).map_err(ScriptError::Validation)?;
    executor.execute(lock).map_err(ScriptError::Validation)?;
    Ok(true)
}

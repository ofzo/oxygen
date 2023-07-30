use std::fmt::Display;

use anyhow::anyhow;

#[derive(Debug, Clone)]
pub enum ValueType {
    ExternRef, //0x6f
    FuncRef,   //0x70
    I32,       //0x7f
    I64,       //0x7e
    F32,       //0x7d
    F64,       //0x7c
    V128,      //0x7b
}

impl ValueType {
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0x6f => Ok(ValueType::ExternRef),
            0x70 => Ok(ValueType::FuncRef),
            0x7f => Ok(ValueType::I32),
            0x7e => Ok(ValueType::I64),
            0x7d => Ok(ValueType::F32),
            0x7c => Ok(ValueType::F64),
            0x7b => Ok(ValueType::V128),
            _ => Err(anyhow!("error value type tag")),
        }
    }
}
impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ValueType::ExternRef => "ExternRef",
                ValueType::FuncRef => "FuncRef",
                ValueType::I32 => "I32",
                ValueType::I64 => "I64",
                ValueType::F32 => "F32",
                ValueType::F64 => "F64",
                ValueType::V128 => "V128",
            }
        )
    }
}

#[derive(Debug, Default)]
pub struct Limit {
    // 0x00 u32 | 0x01 u32 u32
    pub flag: u32,
    pub minimum: u32,
    pub maximum: u32,
}
impl Display for Limit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Limit({:x?}, [{:x?} ~ {:x?}])",
            self.flag, self.minimum, self.maximum
        )
    }
}

#[derive(Debug)]
pub enum RefKind {
    FuncRef,   // 0x70
    ExternRef, //0x6f
}

impl Display for RefKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::FuncRef => "FuncRef",
                Self::ExternRef => "ExternRef",
            }
        )
    }
}

impl RefKind {
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0x6f => Ok(Self::ExternRef),
            0x70 => Ok(Self::FuncRef),
            _ => Err(anyhow!("Error ref tag")),
        }
    }
}

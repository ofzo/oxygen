use std::{fmt::Display, rc::Rc};

use super::{bytecode::ByteCode, opcode::Opcode, typings::Limit, ByteParse, ByteRead, Decode};
use decode_derive::ByteParser;

#[derive(Debug, Default, ByteParser)]
pub struct MemorySection {
    pub raw: Rc<Box<Vec<u8>>>,
    pub offset: usize,
    pub byte_count: u32,
    pub entries: Vec<Mem>,
}

pub fn default(raw: Rc<Box<Vec<u8>>>) -> MemorySection {
    MemorySection {
        raw,
        offset: 0,
        byte_count: 0,
        entries: vec![],
    }
}

#[derive(Debug)]

pub struct Mem {
    pub limits: Limit,
    pub raw: Vec<u8>,
}

impl Decode for MemorySection {
    // 内存段：
    // mem_sec: 0x05|byte_count|vec<mem_type> # vec 目前长度只能是 1
    // 内存类型编码
    // mem_type: limits
    // limits: flags|min|(max)?
    fn decode(&mut self, _ops: &mut Vec<Opcode>) -> anyhow::Result<()> {
        let count = self.read_leb_u32()?;
        for _ in 0..count {
            let start = self.offset;
            let flag = self.read_leb_u32()?;
            let limit = Mem {
                limits: Limit {
                    flag,
                    minimum: self.read_leb_u32()?,
                    maximum: if flag & 0x01 > 0 {
                        self.read_leb_u32()?
                    } else {
                        0x8000 // default 2GB
                    },
                },
                raw: self.raw[start..self.offset].to_vec(),
            };
            self.entries.push(limit);
        }

        Ok(())
    }
}

impl Display for MemorySection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SectionMemory(offset = 0x{:0>8x?}, size= {}, count = {})",
            self.offset,
            self.byte_count,
            self.entries.len()
        )?;
        for (index, item) in self.entries.iter().enumerate() {
            writeln!(f, "    ({index})Memory: {item}")?;
        }
        Ok(())
    }
}

impl Display for Mem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.limits)
    }
}

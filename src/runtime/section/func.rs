use std::{fmt::Display, rc::Rc};

use super::{bytecode::ByteCode, opcode::Opcode, ByteParse, ByteRead, Decode};
use decode_derive::ByteParser;

#[derive(Debug, Default, ByteParser)]
pub struct FuncSection {
    pub offset: usize,
    pub raw: Rc<Box<Vec<u8>>>,
    pub byte_count: u32,
    pub func_count: u32,
    pub entries: Vec<usize>, // index of singtures
}

pub fn default(raw: Rc<Box<Vec<u8>>>) -> FuncSection {
    FuncSection {
        offset: 0,
        raw,
        byte_count: 0,
        func_count: 0,
        entries: vec![],
    }
}

impl Decode for FuncSection
where
    Self: ByteParse + ByteRead,
{
    // 函数段编码格式如下：
    // func_sec: 0x03|byte_count|vec<type_idx>
    fn decode(&mut self, _ops: &mut Vec<Opcode>) -> anyhow::Result<()> {
        self.func_count = self.read_leb_u32()?;

        for _ in 0..self.func_count {
            let type_idx = self.read_leb_u32()?;
            self.entries.push(type_idx as usize)
        }

        Ok(())
    }
}

impl Display for FuncSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SectionFunction(offset = 0x{:0>8x?}, size= {}, count = {})",
            self.offset,
            self.byte_count,
            self.entries.len()
        )?;
        for (index, item) in self.entries.iter().enumerate() {
            writeln!(f, "    ({index})Func: type = {item}")?;
        }
        Ok(())
    }
}

use std::{fmt::Display, rc::Rc};

use decode_derive::ByteParser;

use super::{bytecode::ByteCode, opcode::Opcode, ByteParse, ByteRead, Decode};

#[derive(Debug, Default, ByteParser)]
pub struct CustomSection {
    pub offset: usize,
    pub raw: Rc<Box<Vec<u8>>>,
    pub byte_count: u32,
}

pub fn default(raw: Rc<Box<Vec<u8>>>) -> CustomSection {
    CustomSection {
        offset: 0,
        raw,
        byte_count: 0,
    }
}

impl Decode for CustomSection {
    fn decode(&mut self, _ops: &mut Vec<Opcode>) -> anyhow::Result<()> {
        Ok(())
    }
}
impl Display for CustomSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SectionCustom(offset = 0x{:0>8x?}, size ={})",
            self.offset, self.byte_count
        )
    }
}

use std::{fmt::Display, rc::Rc};

use decode_derive::ByteParser;

use super::{bytecode::ByteCode, opcode::Opcode, ByteParse, ByteRead, Decode};

#[derive(Debug, Default, ByteParser)]
pub struct StartSection {
    pub offset: usize,
    pub raw: Rc<Box<Vec<u8>>>,
    pub byte_count: u32,
    pub start_func: usize,
    pub has_start: bool,
}

pub fn default(raw: Rc<Box<Vec<u8>>>) -> StartSection {
    StartSection {
        offset: 0,
        raw,
        byte_count: 0,
        start_func: 0,
        has_start: false,
    }
}

impl Decode for StartSection
where
    Self: ByteRead,
{
    // 起始段的编码格式如下：
    // start_sec: 0x08|byte_count|func_idx
    fn decode(&mut self, _ops: &mut Vec<Opcode>) -> anyhow::Result<()> {
        self.start_func = self.read_leb_u32()? as usize;
        self.has_start = true;
        Ok(())
    }
}

impl Display for StartSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SectionStart(offset = 0x{:0>8x?}, size = {})",
            self.offset, self.byte_count
        )?;
        writeln!(
            f,
            "    Start: {}",
            if self.has_start {
                self.start_func.to_string()
            } else {
                "NOP".to_string()
            }
        )
    }
}

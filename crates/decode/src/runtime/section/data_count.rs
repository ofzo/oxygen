use std::rc::Rc;

use super::{bytecode::ByteCode, opcode::Opcode, ByteParse, ByteRead, Decode};
use decode_derive::ByteParser;

#[derive(Debug, Default, ByteParser)]
pub struct DataCountSection {
    pub offset: usize,
    pub raw: Rc<Box<Vec<u8>>>,
    pub byte_count: u32,
    pub u32: u32,
}

pub fn default(raw: Rc<Box<Vec<u8>>>) -> DataCountSection {
    DataCountSection {
        offset: 0,
        raw,
        byte_count: 0,
        u32: 0,
    }
}

impl Decode for DataCountSection
where
    Self: ByteParse + ByteCode,
{
    // 函数段编码格式如下：
    // func_sec: 0x0c|byte_count|u32
    fn decode(&mut self, _ops: &mut Vec<Opcode>) -> anyhow::Result<()> {
        self.u32 = self.read_leb_u32()?;
        Ok(())
    }
}

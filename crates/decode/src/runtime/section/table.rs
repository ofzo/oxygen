use std::{fmt::Display, rc::Rc};

use super::{
    bytecode::ByteCode,
    opcode::Opcode,
    typings::{Limit, RefKind},
    ByteParse, ByteRead, Decode,
};
use decode_derive::ByteParser;

#[derive(Debug, Default, ByteParser)]
pub struct TableSection {
    pub offset: usize,
    pub byte_count: u32,
    pub raw: Rc<Box<Vec<u8>>>,
    pub table_count: u32,
    pub entries: Vec<Table>,
}
pub fn default(raw: Rc<Box<Vec<u8>>>) -> TableSection {
    TableSection {
        offset: 0,
        byte_count: 0,
        raw,
        table_count: 0,
        entries: vec![],
    }
}

#[derive(Debug)]
pub struct Table {
    pub kind: RefKind,
    pub raw: Vec<u8>,
    pub limits: Limit,
}

impl Decode for TableSection
where
    Self: ByteParse,
{
    // 表段和表项编码格式如下：
    // table_sec: 0x04|byte_count|vec<table_type> # vec 目前长度只能是 1
    // table_type: 0x70|limits
    // limits: flags|min|(max)?
    fn decode(&mut self, _ops: &mut Vec<Opcode>) -> anyhow::Result<()> {
        let table_count = self.read_leb_u32()?;
        self.table_count = table_count;
        for _ in 0..table_count {
            let start = self.offset;
            let kind = self.read_byte()?;
            let flags = self.read_leb_u32()?;
            let minimum = self.read_leb_u32()?;
            let maximum = if flags & 0x01 > 0 {
                self.read_leb_u32()?.min(0x100000)
            } else {
                0x100000
            };
            self.entries.push(Table {
                kind: RefKind::from_u8(kind)?,
                limits: Limit {
                    flag: flags,
                    minimum,
                    maximum,
                },
                raw: self.raw[start..self.offset].to_vec(),
            })
        }

        Ok(())
    }
}

impl Display for TableSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SectionTable(offset = 0x{:0>8x?}, size= {}, count = {})",
            self.offset,
            self.byte_count,
            self.entries.len()
        )?;
        for (index, item) in self.entries.iter().enumerate() {
            writeln!(f, "    ({index})Table: {item}")?;
        }
        Ok(())
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.kind, self.limits)
    }
}

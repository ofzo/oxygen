use std::{fmt::Display, rc::Rc};

use super::{bytecode::ByteCode, opcode::Opcode, ByteParse, ByteRead, Decode};
use anyhow::anyhow;
use decode_derive::ByteParser;

#[derive(Debug, Default, ByteParser)]
pub struct ExportSection {
    pub offset: usize,
    pub byte_count: u32,
    pub export_count: u32,
    pub raw: Rc<Box<Vec<u8>>>,
    pub entries: Vec<Export>,
}

pub fn default(raw: Rc<Box<Vec<u8>>>) -> ExportSection {
    ExportSection {
        offset: 0,
        byte_count: 0,
        export_count: 0,
        raw,
        entries: vec![],
    }
}

#[derive(Debug)]
pub struct Export {
    pub raw: Vec<u8>,
    pub name: String,
    pub kind: ExportKind,
}

#[derive(Debug, Clone)]
pub enum ExportKind {
    Func(usize),   //= 0x00,
    Table(usize),  // = 0x01,
    Memory(usize), // = 0x02,
    GLobal(usize), // = 0x03,
}

impl ExportKind {
    pub fn from_u8(value: u8, index: usize) -> anyhow::Result<Self> {
        match value {
            0x00 => Ok(ExportKind::Func(index)),
            0x01 => Ok(ExportKind::Table(index)),
            0x02 => Ok(ExportKind::Memory(index)),
            0x03 => Ok(ExportKind::GLobal(index)),
            _ => Err(anyhow!("Error ref tag")),
        }
    }
}

impl Decode for ExportSection
where
    Self: ByteParse,
{
    // 导出段编码格式如下：
    // export_sec: 0x07|byte_count|vec<export>
    // export: name|export_desc
    // export_desc: tag|[func_idx, table_idx, mem_idx, global_idx]
    fn decode(&mut self, _ops: &mut Vec<Opcode>) -> anyhow::Result<()> {
        self.export_count = self.read_leb_u32()?;

        for _ in 0..self.export_count {
            let start = self.offset;
            let name_len = self.read_leb_u32()?;
            let name = self.peek_bytes(name_len)?;
            self.skip(name_len);
            let kind = self.read_byte()?;
            let index = self.read_leb_u32()? as usize;

            self.entries.push(Export {
                name: String::from_utf8(name)?,
                kind: ExportKind::from_u8(kind, index)?,
                raw: self.raw[start..self.offset].to_vec(),
            })
        }
        Ok(())
    }
}

impl Display for ExportSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SectionExport(offset = 0x{:0>8x?}, size = {}, count = {})",
            self.offset,
            self.byte_count,
            self.entries.len()
        )?;
        for (index, item) in self.entries.iter().enumerate() {
            writeln!(f, "    ({index})Export: {item}")?;
        }

        Ok(())
    }
}

impl Display for Export {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.kind)
    }
}

impl Display for ExportKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Func(index) => format!("Func({index})"),
                Self::Table(index) => format!("Table({index})"),
                Self::Memory(index) => format!("Memory({index})"),
                Self::GLobal(index) => format!("GLobal({index})"),
            }
        )
    }
}

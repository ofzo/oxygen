use std::{fmt::Display, rc::Rc};

use anyhow::anyhow;
use decode_derive::ByteParser;

use super::{bytecode::ByteCode, opcode::Opcode, ByteParse, ByteRead, Decode};

#[derive(Debug, Default, ByteParser)]
pub struct DataSection {
    pub offset: usize,
    pub raw: Rc<Box<Vec<u8>>>,
    pub byte_count: u32,
    pub data_count: u32,
    pub entries: Vec<Data>,
}

pub fn default(raw: Rc<Box<Vec<u8>>>) -> DataSection {
    DataSection {
        offset: 0,
        raw,
        byte_count: 0,
        data_count: 0,
        entries: vec![],
    }
}

#[derive(Debug)]
pub struct Data {
    // pub raw: Vec<u8>,
    pub flag: u32,
    pub offset: usize,
    pub kind: DataKind,
}

#[derive(Debug)]
pub enum DataKind {
    Expr((usize, usize, usize), Vec<u8>),
    Vec(Vec<u8>),
    MemIdx(usize, (usize, usize, usize), Vec<u8>),
}

impl Decode for DataSection
where
    Self: ByteCode,
{
    // 数据段编码格式如下：
    // data_sec: 0x0b|byte_count|vec<data>
    // data: mem_idx|offset_expr|vec<byte>
    fn decode(&mut self, ops: &mut Vec<Opcode>) -> anyhow::Result<()> {
        let data_count = self.read_leb_u32()?;
        self.data_count = data_count;

        for _ in 0..data_count {
            let start = self.offset;
            let flag = self.read_leb_u32()?;

            let kind = match flag {
                00 => {
                    let code = self.parse_code(ops, &mut vec![])?;
                    let num = self.read_leb_u32()?;
                    DataKind::Expr(code, self.read_bytes(num)?)
                }
                01 => {
                    let num = self.read_leb_u32()?;
                    DataKind::Vec(self.read_bytes(num)?)
                }
                02 => {
                    let memidx = self.read_leb_u32()? as usize;
                    let expr = self.parse_code(ops, &mut vec![])?;
                    let num = self.read_leb_u32()?;
                    DataKind::MemIdx(memidx, expr, self.read_bytes(num)?)
                }
                _ => return Err(anyhow!("unkonwn data kind {flag}")),
            };
            self.entries.push(Data {
                flag,
                offset: start,
                // raw: self.raw[start..self.offset].to_vec(),
                kind,
            })
        }
        Ok(())
    }
}

impl Display for DataSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SectionData(offset = 0x{:0>8x?}, size = {}, count = {})",
            self.offset,
            self.byte_count,
            self.entries.len()
        )?;
        for (index, item) in self.entries.iter().enumerate() {
            writeln!(f, "    ({index})Data: {item}")?;
        }
        Ok(())
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            DataKind::Expr(e, v) => write!(
                f,
                "Expr(offset = 0x{:0>8x?}, expr = Opcode[{:?}], data = byte[{:?}])",
                self.offset,
                e,
                v.len()
            ),
            DataKind::Vec(v) => write!(f, "offset = 0x{:0>8x?}, data = byte[{:?}]",self.offset,  v.len()),
            DataKind::MemIdx(m, e, v) => write!(
                f,
                "MemIdx(offset = 0x{:0>8x?}, mem_index = {m:x?}, expr = Opcode[{:?}], data = byte[{:?}])",self.offset,
                e,
                v.len()
            ),
        }
    }
}

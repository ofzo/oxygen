use std::{fmt::Display, rc::Rc};

use decode_derive::ByteParser;

use super::{bytecode::ByteCode, opcode::Opcode, typings::ValueType, ByteParse, ByteRead, Decode};

#[derive(Debug, Default, ByteParser)]
pub struct CodeSection {
    pub offset: usize,
    pub byte_count: u32,
    pub body_count: u32,
    pub raw: Rc<Box<Vec<u8>>>,
    pub entries: Vec<FuncBody>,
}

#[derive(Debug, Default, Clone)]
pub struct FuncBody {
    pub size: usize,
    pub local_count: u32,
    pub locales: Vec<(u32, ValueType)>,
    pub code: (usize, usize, usize),
    pub offset: usize,
    // pub raw: [u8],
}
pub fn default(raw: Rc<Box<Vec<u8>>>) -> CodeSection {
    CodeSection {
        offset: 0,
        byte_count: 0,
        body_count: 0,
        raw,
        entries: vec![],
    }
}

impl Decode for CodeSection
where
    Self: ByteCode,
{
    // 代码段编码格式如下：
    // code_sec: 0xoA|byte_count|vec<code>
    // code: byte_count|vec<locals>|expr
    // locals: local_count|val_type
    fn decode(&mut self, ops: &mut Vec<Opcode>) -> anyhow::Result<()> {
        self.body_count = self.read_leb_u32()?;
        for _ in 0..self.body_count {
            let start = self.offset;
            let body_size = self.read_leb_u32()?;
            let local_count = self.read_leb_u32()?;
            let mut locales = vec![];
            for _ in 0..local_count {
                let count = self.read_leb_u32()?;
                let val_type = self.read_byte()?;
                locales.push((count, ValueType::from_u8(val_type)?))
            }
            // let code = self.read_util(0x0b)?;
            let code = self.parse_code(ops, &mut vec![])?;
            self.entries.push(FuncBody {
                size: body_size as usize,
                local_count,
                locales,
                code,
                offset: start,
            })
        }
        Ok(())
    }
}

impl Display for CodeSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SectionCode(offset = 0x{:0>8x?}, size = {}, count = {})",
            self.offset,
            self.byte_count,
            self.entries.len()
        )?;
        for (index, item) in self.entries.iter().enumerate() {
            writeln!(f, "    ({index})Code: {item}")?;
        }
        Ok(())
    }
}

impl Display for FuncBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let locales = self
            .locales
            .iter()
            .map(|item| format!("{}[{}]", item.1, item.0))
            .collect::<Vec<_>>()
            .join(", ");

        write!(
            f,
            "offset = 0x{:0>8x?}, local({}), code = Opcode[{:?}]",
            self.offset, locales, self.code
        )?;
        Ok(())
    }
}

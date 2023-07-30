use std::fmt::Display;
use std::rc::Rc;

use super::opcode::Opcode;
use super::typings::ValueType;
use super::{bytecode::ByteCode, ByteParse, ByteRead, Decode};

use anyhow::ensure;
use decode_derive::ByteParser;

#[derive(Debug, Default, ByteParser)]
pub struct TypeSection {
    pub raw: Rc<Box<Vec<u8>>>,
    pub byte_count: u32,
    pub offset: usize,
    pub type_count: u32,
    pub entries: Vec<FunctionType>,
}

pub fn default(raw: Rc<Box<Vec<u8>>>) -> TypeSection {
    TypeSection {
        raw,
        byte_count: 0,
        offset: 0,
        type_count: 0,
        entries: vec![],
    }
}

#[derive(Debug)]
pub struct FunctionType {
    pub raw: Vec<u8>,
    pub param_count: u32,
    pub result_count: u32,
    pub params: Vec<ValueType>,
    pub results: Vec<ValueType>,
}

impl Decode for TypeSection
where
    Self: ByteCode + ByteParse,
{
    /// deocde type section
    ///
    /// type_sec: 0x01| byte_count | vec<func_type>
    /// buf 不包含 0x01 byte_count
    fn decode(&mut self, _ops: &mut Vec<Opcode>) -> anyhow::Result<()> {
        let type_count = self.read_leb_u32()?;
        self.type_count = type_count;

        for _ in 0..type_count {
            let start = self.offset;
            let func_type = self.read_byte()?;
            ensure!(
                func_type == 0x60,
                "Unkonwn type: expectd 0x60, but get {}",
                func_type
            );

            let param_count = self.read_leb_u32()?;
            let mut params = Vec::with_capacity(param_count as usize);
            for _ in 0..param_count {
                let param_type = self.read_byte().unwrap();
                params.push(ValueType::from_u8(param_type)?);
            }

            let result_count = self.read_leb_u32()?;
            let mut results = Vec::with_capacity(result_count as usize);
            for _ in 0..result_count {
                let result_type = self.read_byte()?;
                results.push(ValueType::from_u8(result_type)?);
            }
            self.entries.push(FunctionType {
                raw: self.raw[start..self.offset].to_vec(),
                param_count,
                result_count,
                params,
                results,
            })
        }

        Ok(())
    }
}

impl Display for TypeSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SectionType(offset = 0x{:0>8x?}, size= {}, count = {})",
            self.offset,
            self.byte_count,
            self.entries.len()
        )?;
        for (index, item) in self.entries.iter().enumerate() {
            writeln!(f, "    ({index}){}", item)?;
        }
        Ok(())
    }
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params = self
            .params
            .iter()
            .map(|item| format!("{}", item))
            .collect::<Vec<_>>()
            .join(",");

        let results = self
            .results
            .iter()
            .map(|item| format!("{}", item))
            .collect::<Vec<_>>()
            .join(",");
        write!(
            f,
            "Type: ({}) => {}",
            params,
            if results.is_empty() {
                "NOP"
            } else {
                results.as_str()
            }
        )
    }
}

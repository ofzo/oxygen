use self::{
    code::CodeSection, custom::CustomSection, data::DataSection, data_count::DataCountSection,
    element::ElementSection, export::ExportSection, func::FuncSection, global::GlobalSection,
    import::ImportSection, memory::MemorySection, opcode::Opcode, start::StartSection,
    table::TableSection, types::TypeSection,
};

use super::constants;
use crate::leb;
pub mod bytecode;
pub mod code;
pub mod custom;
pub mod data;
pub mod data_count;
pub mod element;
pub mod export;
pub mod func;
pub mod global;
pub mod import;
pub mod memory;
pub mod opcode;
pub mod start;
pub mod table;
pub mod types;
pub mod typings;

use anyhow::anyhow;

#[derive(Debug, Default)]
pub struct Section {
    pub custom: CustomSection,
    pub types: TypeSection,
    pub import: ImportSection,
    pub func: FuncSection,
    pub table: TableSection,
    pub memory: MemorySection,
    pub global: GlobalSection,
    pub export: ExportSection,
    pub start: StartSection,
    pub element: ElementSection,
    pub code: CodeSection,
    pub data: DataSection,
    pub data_count: DataCountSection,
}

pub trait ByteParse {
    fn offset(&self) -> usize;
    fn length(&self) -> usize;
    fn skip(&mut self, num: u32);
    fn get(&self, offset: usize) -> Option<&u8>;
}
pub trait ByteRead
where
    Self: ByteParse,
{
    fn is_eof(&self) -> bool {
        self.offset() > self.length()
    }
    fn peek_bytes(&mut self, num: u32) -> anyhow::Result<Vec<u8>> {
        let num = num as usize;
        anyhow::ensure!(
            !(self.is_eof() || self.offset() + num > self.length()),
            "Unexpect token <EOF>"
        );
        let mut arr = vec![];
        for i in 0..num {
            arr.push(match self.get(self.offset() + i) {
                Some(v) => v.clone(),
                None => return Err(anyhow!("overflow")),
            })
        }
        return Ok(arr);
    }

    fn read_byte(&mut self) -> anyhow::Result<u8> {
        let bytes = self.peek_bytes(1)?;
        self.skip(1);
        Ok(bytes.get(0).unwrap().clone().into())
    }
    fn read_bytes(&mut self, num: u32) -> anyhow::Result<Vec<u8>> {
        let bytes = self.peek_bytes(num)?;
        self.skip(num);
        Ok(bytes)
    }

    fn read_leb_u32(&mut self) -> anyhow::Result<u32> {
        let remain = (self.length() - self.offset()) as u32;
        let buf = if remain < constants::MAX_NUMBER_OF_BYTE_U32 {
            self.peek_bytes(remain)?
        } else {
            self.peek_bytes(constants::MAX_NUMBER_OF_BYTE_U32)?
        };
        let (val, size) = leb::decode_leb_u32(&buf);
        self.skip(size as u32);
        Ok(val)
    }
    fn read_leb_i32(&mut self) -> anyhow::Result<i32> {
        let remain = (self.length() - self.offset()) as u32;
        let buf = if remain < constants::MAX_NUMBER_OF_BYTE_U32 {
            self.peek_bytes(remain)?
        } else {
            self.peek_bytes(constants::MAX_NUMBER_OF_BYTE_U32)?
        };
        let (val, size) = leb::decode_leb_i32(&buf);
        self.skip(size as u32);
        Ok(val)
    }
    fn read_leb_u64(&mut self) -> anyhow::Result<u64> {
        let remain = (self.length() - self.offset()) as u32;
        let buf = if remain < constants::MAX_NUMBER_OF_BYTE_U64 {
            self.peek_bytes(remain)?
        } else {
            self.peek_bytes(constants::MAX_NUMBER_OF_BYTE_U64)?
        };
        let (val, size) = leb::decode_leb_u64(&buf);
        self.skip(size as u32);
        Ok(val)
    }
    fn read_leb_i64(&mut self) -> anyhow::Result<i64> {
        let remain = (self.length() - self.offset()) as u32;
        let buf = if remain < constants::MAX_NUMBER_OF_BYTE_U64 {
            self.peek_bytes(remain)?
        } else {
            self.peek_bytes(constants::MAX_NUMBER_OF_BYTE_U64)?
        };
        let (val, size) = leb::decode_leb_i64(&buf);
        self.skip(size as u32);
        Ok(val)
    }
}

pub(crate) trait Decode {
    fn decode(&mut self, ops: &mut Vec<Opcode>) -> anyhow::Result<()>;
}

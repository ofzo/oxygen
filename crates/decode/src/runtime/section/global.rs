use std::{fmt::Display, rc::Rc};

// use super::typings::ValueType;
use super::{bytecode::ByteCode, opcode::Opcode, typings::ValueType, ByteParse, ByteRead, Decode};
use decode_derive::ByteParser;

#[derive(Debug, Default, ByteParser)]
pub struct GlobalSection {
    pub offset: usize,
    pub raw: Rc<Box<Vec<u8>>>,
    pub byte_count: u32,
    pub global_count: u32,
    pub entries: Vec<Global>,
}
pub fn default(raw: Rc<Box<Vec<u8>>>) -> GlobalSection {
    GlobalSection {
        offset: 0,
        raw,
        byte_count: 0,
        global_count: 0,
        entries: vec![],
    }
}

#[derive(Debug)]
pub struct Global {
    pub val_ty: ValueType,
    pub mutability: bool,
    pub raw: Vec<u8>,
    pub expr: (usize, usize, usize),
}

impl Decode for GlobalSection
where
    Self: ByteCode + ByteParse,
{
    // 全局段格式：
    // global_sec: 0x60|byte_count|vec<global>
    // 全局项的编码
    // global: global_type|init_expr
    // global_type: val_type|mut
    // init_expr: (byte)+|0x0B
    fn decode(&mut self, ops: &mut Vec<Opcode>) -> anyhow::Result<()> {
        let global_count = self.read_leb_u32()?;
        self.global_count = global_count;
        for _ in 0..global_count {
            let start = self.offset;
            let val_ty = self.read_byte()?;
            let mutability = self.read_byte()? > 0;
            let expr = self.parse_code(ops, &mut vec![])?;

            self.entries.push(Global {
                val_ty: ValueType::from_u8(val_ty).unwrap(),
                mutability,
                expr,
                raw: self.raw[start..self.offset].to_vec(),
            })
        }
        Ok(())
    }
}

impl Display for GlobalSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SectionGlobal(offset = 0x{:0>8x?}, size= {}, count = {})",
            self.offset,
            self.byte_count,
            self.entries.len()
        )?;
        for (index, item) in self.entries.iter().enumerate() {
            writeln!(f, "    ({index})Global: {item}")?
        }
        Ok(())
    }
}

impl Display for Global {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, type = {}, expr = Opcode[{:?}]",
            if self.mutability { "Var" } else { "Const" },
            self.val_ty,
            self.expr
        )
    }
}

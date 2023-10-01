use std::{fmt::Display, rc::Rc};

// use super::typings::ValueType;
use super::{
    bytecode::ByteCode,
    global::Global,
    opcode::Opcode,
    typings::{Limit, ValueType},
    ByteParse, ByteRead, Decode,
};
use anyhow::anyhow;
use decode_derive::ByteParser;

#[derive(Debug, Default, ByteParser)]
pub struct ImportSection {
    pub offset: usize,
    pub byte_count: u32,
    pub import_count: u32,
    pub raw: Rc<Box<Vec<u8>>>,
    pub entries: Vec<Importer>,
}
#[derive(Debug)]
pub struct Importer {
    pub mod_name: String,
    pub field_name: String,
    pub tag: u8,
    pub kind: Kind,
}

#[derive(Debug)]
pub enum Kind {
    Func(usize),      // 0x00
    Table(u8, Limit), // 0x01, (0x70 | 0x6f,  0x00 u32 | 0x01 u32 u32 )
    Memory(Limit),    // 0x02
    Global(Global),   // 0x03,  ( u8, 0x00 | 0x01)
}

pub fn default(raw: Rc<Box<Vec<u8>>>) -> ImportSection {
    ImportSection {
        offset: 0,
        byte_count: 0,
        import_count: 0,
        raw,
        entries: vec![],
    }
}

impl Decode for ImportSection
where
    Self: ByteParse,
{
    // 导入段编码格式如下：
    // import_sec: 0x02|byte_count|vec<import>
    // import: module_name|member_name|import_desc
    // import_desc: tag|[type_idx, table_type, mem_type, global_type]
    fn decode(&mut self, _ops: &mut Vec<Opcode>) -> anyhow::Result<()> {
        let import_count = self.read_leb_u32()?;
        self.import_count = import_count;
        for _ in 0..import_count {
            let start = self.offset;
            let name_len = self.read_leb_u32()?;
            let mod_name = self.peek_bytes(name_len)?;
            self.skip(name_len);

            let name_len = self.read_leb_u32()?;
            let field_name = self.peek_bytes(name_len)?;
            self.skip(name_len);

            let tag = self.read_byte()?;

            let kind = match tag {
                0x00 => Kind::Func(self.read_leb_u32()? as usize),
                0x01 => Kind::Table(
                    self.read_byte()?, // 0x70 <funcref>  |  0x6f <externref>
                    match self.read_byte()? {
                        0x00 => Limit {
                            flag: 0x00,
                            minimum: self.read_leb_u32()?,
                            maximum: 0x10000,
                        },
                        0x01 => Limit {
                            flag: 0x01,
                            minimum: self.read_leb_u32()?,
                            maximum: self.read_leb_u32()?,
                        },
                        _ => return Err(anyhow!("unkonwn table limit flag")),
                    },
                ),
                0x02 => Kind::Memory(match self.read_byte()? {
                    0x00 => Limit {
                        flag: 0x00,
                        minimum: self.read_leb_u32()?,
                        maximum: 0x10000,
                    },
                    0x01 => Limit {
                        flag: 0x01,
                        minimum: self.read_leb_u32()?,
                        maximum: self.read_leb_u32()?,
                    },
                    _ => return Err(anyhow!("unkonwn limit flag")),
                }),
                0x03 => {
                    let val_ty = self.read_byte()?;
                    let mutability = self.read_byte()? > 0;
                    Kind::Global(Global {
                        val_ty: ValueType::from_u8(val_ty).unwrap(),
                        mutability,
                        raw: self.raw[start..self.offset].to_vec(),
                        expr: (0, 0, 0),
                    })
                } // 0x00 | 0x01
                _ => return Err(anyhow!("unkonwn import kind")),
            };
            self.entries.push(Importer {
                mod_name: String::from_utf8(mod_name).unwrap(),
                field_name: String::from_utf8(field_name).unwrap(),
                tag,
                kind,
            })
        }
        Ok(())
    }
}

impl Display for ImportSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SectionImport(offset = 0x{:0>8x?}, size= {}, count = {})",
            self.offset,
            self.byte_count,
            self.entries.len()
        )?;
        for (index, item) in self.entries.iter().enumerate() {
            writeln!(f, "    ({index})Import: {item}")?;
        }

        Ok(())
    }
}

impl Display for Importer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{} {}", self.mod_name, self.field_name, self.kind)
    }
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Kind::Func(v) => format!("Func({v:x?})"),
                Kind::Table(v, limit) => format!("Table({v:x?}, {})", limit),
                Kind::Memory(limit) => format!("Memory({limit})"),
                Kind::Global(gl) => format!("Global({gl})"),
            }
        )
    }
}

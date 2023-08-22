use std::fmt::Display;
use std::rc::Rc;

use super::bytecode::ByteCode;
use super::opcode::Opcode;
use super::typings::RefKind;
use super::{ByteParse, ByteRead, Decode};
use anyhow::{anyhow, ensure};
use decode_derive::ByteParser;

#[derive(Debug, Default, ByteParser)]
pub struct ElementSection {
    pub offset: usize,
    pub ele_count: u32,
    pub byte_count: u32,
    pub raw: Rc<Box<Vec<u8>>>,
    pub entries: Vec<Element>,
}

pub fn default(raw: Rc<Box<Vec<u8>>>) -> ElementSection {
    ElementSection {
        offset: 0,
        ele_count: 0,
        byte_count: 0,
        raw,
        entries: vec![],
    }
}

#[derive(Debug)]
pub enum Element {
    E0x00(ElementKind<((usize, usize, usize), Vec<usize>)>),
    E0x01(ElementKind<(u8, Vec<usize>)>),
    E0x02(ElementKind<(usize, (usize, usize, usize), u8, Vec<usize>)>),
    E0x03(ElementKind<(u8, Vec<usize>)>),
    E0x04(ElementKind<((usize, usize, usize), Vec<(usize, usize, usize)>)>),
    E0x05(ElementKind<(RefKind, Vec<(usize, usize, usize)>)>),
    E0x06(
        ElementKind<(
            usize,
            (usize, usize, usize),
            RefKind,
            Vec<(usize, usize, usize)>,
        )>,
    ),
    E0x07(ElementKind<(RefKind, Vec<(usize, usize, usize)>)>),
}
#[derive(Debug)]
pub struct ElementKind<T> {
    pub raw: Vec<u8>,
    pub offset: usize,
    pub ele: T,
}

impl Decode for ElementSection
where
    Self: ByteCode,
{
    //  元素段编码格式如下：
    //  elem_sec: 0x09|byte_count|vec<elem>
    //  elem: 0x00 offset_expr | vec<func_id>
    //  elem: 0x01 elekind | vec<func_id>
    //  elem: 0x02 table_idx| offset_expr | elekind | vec<func_id>
    //  elem: 0x03 elekind |vec<func_id>
    //  elem: 0x04 expr | vec<expr>
    //  elem: 0x05 reftype | vec<expr>
    //  elem: 0x06 table_idx | expr | reftype | vec<expr>
    //  elem: 0x07 reftype | vec<expr>
    //  elekind = 0x00
    fn decode(&mut self, ops: &mut Vec<Opcode>) -> anyhow::Result<()> {
        let ele_count = self.read_leb_u32()?;
        self.ele_count = ele_count;
        for _ in 0..ele_count {
            let start = self.offset;
            let flag = self.read_leb_u32()?;

            let ele = match flag {
                0x00 => {
                    let code = self.parse_code(ops, &mut vec![])?;
                    let count = self.read_leb_u32()?;
                    let mut func = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        func.push(self.read_leb_u32()? as usize);
                    }
                    Element::E0x00(ElementKind {
                        raw: self.raw[start..self.offset].to_vec(),
                        offset: start,
                        ele: (code, func),
                    })
                }
                0x01 => {
                    let elekind = self.read_byte()?;
                    ensure!(elekind == 0x00, "0x01 elemnetkind  must be  0x00");
                    let count = self.read_leb_u32()?;
                    let mut func = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        func.push(self.read_leb_u32()? as usize);
                    }
                    Element::E0x01(ElementKind {
                        raw: self.raw[start..self.offset].to_vec(),
                        offset: start,
                        ele: (elekind, func),
                    })
                }
                0x02 => {
                    let table_idx = self.read_leb_u32()? as usize;
                    let expr = self.parse_code(ops, &mut vec![])?;
                    let elekind = self.read_byte()?;
                    ensure!(elekind == 0x00, "0x02 elemnet kind must be 0x00");

                    let count = self.read_leb_u32()?;
                    let mut func = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        func.push(self.read_leb_u32()? as usize);
                    }
                    Element::E0x02(ElementKind {
                        raw: self.raw[start..self.offset].to_vec(),
                        offset: start,
                        ele: (table_idx, expr, elekind, func),
                    })
                }
                0x03 => {
                    let elekind = self.read_byte()?;
                    ensure!(elekind == 0x00, "0x03 elemnet kind must be 0x00");
                    let count = self.read_leb_u32()?;
                    let mut func = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        func.push(self.read_leb_u32()? as usize);
                    }
                    Element::E0x03(ElementKind {
                        raw: self.raw[start..self.offset].to_vec(),
                        offset: start,
                        ele: (elekind, func),
                    })
                }
                0x04 => {
                    let expr = self.parse_code(ops, &mut vec![])?;
                    let count = self.read_leb_u32()?;
                    let mut exprs = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        exprs.push(self.parse_code(ops, &mut vec![])?);
                    }
                    Element::E0x04(ElementKind {
                        raw: self.raw[start..self.offset].to_vec(),
                        offset: start,
                        ele: (expr, exprs),
                    })
                }
                0x05 => {
                    let ty = self.read_byte()?;
                    let count = self.read_leb_u32()?;
                    let mut exprs = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        exprs.push(self.parse_code(ops, &mut vec![])?);
                    }
                    let ele = (RefKind::from_u8(ty)?, exprs);
                    Element::E0x05(ElementKind {
                        raw: self.raw[start..self.offset].to_vec(),
                        offset: start,
                        ele,
                    })
                }
                0x06 => {
                    let table_idx = self.read_leb_u32()? as usize;
                    let expr = self.parse_code(ops, &mut vec![])?;
                    let ref_ty = RefKind::from_u8(self.read_byte()?)?;
                    let count = self.read_leb_u32()?;
                    let mut exprs = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        exprs.push(self.parse_code(ops, &mut vec![])?);
                    }
                    Element::E0x06(ElementKind {
                        raw: self.raw[start..self.offset].to_vec(),
                        offset: start,
                        ele: (table_idx, expr, ref_ty, exprs),
                    })
                }
                0x07 => {
                    let ref_ty = RefKind::from_u8(self.read_byte()?)?;
                    let count = self.read_leb_u32()?;
                    let mut exprs = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        exprs.push(self.parse_code(ops, &mut vec![])?);
                    }
                    Element::E0x07(ElementKind {
                        raw: self.raw[start..self.offset].to_vec(),
                        offset: start,
                        ele: (ref_ty, exprs),
                    })
                }
                v => return Err(anyhow!("Unknown element flag {v:x}")),
            };
            self.entries.push(ele);
        }

        Ok(())
    }
}

impl Display for ElementSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SectionElement(offset = 0x{:0>8x?}, size = {}, count = {})",
            self.offset,
            self.byte_count,
            self.entries.len()
        )?;

        for (index, item) in self.entries.iter().enumerate() {
            writeln!(
                f,
                "    ({index})Element: offset = 0x{:0>8x?}, {item}",
                self.offset
            )?;
        }

        Ok(())
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Element::E0x00(v) => write!(
                f,
                "E0x00(expr = Opcode[{:?}], func_index = {:?})",
                v.ele.0, v.ele.1
            ),
            Element::E0x01(v) => write!(
                f,
                "E0x01(elem_kind = {:x?}, func_index = {:?})",
                v.ele.0, v.ele.1
            ),
            Element::E0x02(v) => write!(
                f,
                "E0x02(table_index = {}, expr = Opcode[{:?}], elem_kind = {:x?}, func_index = {:?})",
                v.ele.0, v.ele.1, v.ele.2, v.ele.3
            ),
            Element::E0x03(v) => write!(
                f,
                "E0x03(elem_kind = {:x?}, func_index = {:?})",
                v.ele.0, v.ele.1
            ),
            Element::E0x04(v) => write!(
                f,
                "E0x04(expr = Opcode[{:?}], expr = Opcode[{:?}][])",
                v.ele.0,
                v.ele.1
            ),
            Element::E0x05(v) => write!(
                f,
                "E0x05(ref_type = {}, ecpr = Opcode[{}])",
                v.ele.0,
                v.ele.1.len()
            ),
            Element::E0x06(v) => write!(
                f,
                "E0x06(table_index = {}, expr = Opcode[{:?}], ref_type = {:x?}, expr = Opcode[{}][])",
                v.ele.0,
                v.ele.1,
                v.ele.2,
                v.ele.3.len()
            ),
            Element::E0x07(v) => write!(
                f,
                "E0x07(ref_type = {:x?}, expr = Opcode[{:?}][])",
                v.ele.0,
                v.ele.1
            ),
        }
    }
}

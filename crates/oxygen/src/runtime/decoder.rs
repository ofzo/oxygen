use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Shl, Sub};
use std::rc::Rc;

use anyhow::ensure;

use super::constants::{self, PAGE_SIZE};
use super::section::code::FuncBody;
use super::section::export::ExportKind;
use super::section::opcode::Opcode;
use super::section::{self, import, ByteParse, ByteRead, Decode, Section};

#[derive(Debug)]
pub struct WasmModule {
    pub raw: Rc<Box<Vec<u8>>>,
    pub offset: usize,
    pub length: usize,
    pub magic_number: Vec<u8>,
    pub version: u32,
    pub section: Section,

    /// progma count
    pub pc: usize,
    /// stack poniter
    pub sp: usize,
    /// frame pointer
    pub fp: usize,
    /// callstack pointer
    pub csp: usize,
    // pub callstack: Vec<Frame>,
    // pub blocks: HashMap<usize, Rc<Block>>,
    pub stack: Vec<WasmValue>,
    pub table: Vec<Vec<usize>>,
    pub mem: Vec<Vec<u8>>,
    pub global: Vec<Global>,
    pub exports: HashMap<String, ExportKind>,
    pub func: Vec<FuncKind>,
    pub ops: Vec<Opcode>,
}

#[derive(Debug, Clone)]
pub enum FuncKind {
    Import(
        usize,
        fn(module: &mut WasmModule, arg: &Vec<WasmValue>) -> Vec<WasmValue>,
    ), // ty
    Local((usize, FuncBody)), // (ty, code index)
}

#[derive(Debug)]
pub enum Global {
    Const(WasmValue),
    Var(WasmValue),
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum WasmValue {
    #[default]
    NOP,
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    V128(i128),
}

impl ByteRead for WasmModule {}
impl ByteParse for WasmModule {
    fn offset(&self) -> usize {
        return self.offset;
    }

    fn length(&self) -> usize {
        return self.length;
    }

    fn skip(&mut self, num: u32) {
        self.offset += num as usize;
    }

    fn get(&self, offset: usize) -> Option<&u8> {
        self.raw.get(offset)
    }
}

impl WasmModule
where
    Self: ByteRead,
{
    pub fn decode(&mut self) -> anyhow::Result<()> {
        self.magic_number = self.parse_magic()?;
        self.version = self.parse_version()?;
        // self.ops.push(Opcode::End(0));

        while self.offset < self.length {
            match self.parse_section() {
                Ok(_) => continue,
                Err(err) => {
                    println!("{}", self);
                    return Err(err);
                }
            }
        }

        Ok(())
    }
    fn parse_version(&mut self) -> anyhow::Result<u32> {
        let version = self.peek_bytes(4)?;
        anyhow::ensure!(version == constants::VERSION, "Unknown binary version");
        self.skip(4);
        Ok(u32::from_le_bytes(version.try_into().unwrap()))
    }
    fn parse_magic(&mut self) -> anyhow::Result<Vec<u8>> {
        let header = self.peek_bytes(4)?;
        anyhow::ensure!(
            header == constants::MAGIC_NUMBER,
            "Magic header not detected"
        );
        self.skip(4);
        Ok(header)
    }

    fn parse_section(&mut self) -> anyhow::Result<()> {
        let offset = self.offset;
        let section_id = self.read_leb_u32()?;
        ensure!(section_id <= 12, "unkonwn section id {section_id}");

        let section_byte_count = self.read_leb_u32()?;

        macro_rules! decode_section {
            ( $x:ident ) => {{
                self.section.$x.offset = self.offset;
                self.section.$x.byte_count = self.offset as u32 + section_byte_count;

                self.section.$x.decode(&mut self.ops)?;

                self.section.$x.offset = offset;
                self.section.$x.byte_count = section_byte_count;
                self.skip(section_byte_count);
            }};
        }

        match section_id {
            0 => decode_section!(custom),
            1 => decode_section!(types),
            2 => decode_section!(import),
            3 => decode_section!(func),
            4 => decode_section!(table),
            5 => decode_section!(memory),
            6 => decode_section!(global),
            7 => decode_section!(export),
            8 => decode_section!(start),
            9 => decode_section!(element),
            10 => decode_section!(code),
            11 => decode_section!(data),
            12 => decode_section!(data_count),
            _ => {}
        }
        Ok(())
    }

    pub fn default(raw: Vec<u8>) -> WasmModule {
        let raw = Rc::new(Box::new(raw));
        Self {
            raw: raw.clone(),
            offset: 0,
            length: raw.len(),
            magic_number: vec![],
            version: 0,
            section: Section {
                custom: section::custom::default(raw.clone()),
                types: section::types::default(raw.clone()),
                import: section::import::default(raw.clone()),
                func: section::func::default(raw.clone()),
                table: section::table::default(raw.clone()),
                memory: section::memory::default(raw.clone()),
                global: section::global::default(raw.clone()),
                export: section::export::default(raw.clone()),
                start: section::start::default(raw.clone()),
                element: section::element::default(raw.clone()),
                code: section::code::default(raw.clone()),
                data: section::data::default(raw.clone()),
                data_count: section::data_count::default(raw.clone()),
            },
            pc: 0,
            sp: 0,
            fp: 0,
            csp: 0,
            stack: Default::default(),
            table: Default::default(),
            mem: Default::default(),
            global: Default::default(),
            exports: Default::default(),
            func: Default::default(),
            ops: Default::default(),
        }
    }
}

impl Display for WasmModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Type: \\0asm")?;
        writeln!(f, "Version: {:x?}", self.version)?;
        writeln!(f, "Size: {:?}\n", self.raw.len())?;
        write!(f, "{}", self.section.custom)?;
        write!(f, "{}", self.section.types)?;
        write!(f, "{}", self.section.import)?;
        write!(f, "{}", self.section.func)?;

        write!(f, "{}", self.section.table)?;
        write!(f, "{}", self.section.memory)?;
        write!(f, "{}", self.section.global)?;

        write!(f, "{}", self.section.export)?;

        write!(f, "{}", self.section.start)?;
        write!(f, "{}", self.section.element)?;

        write!(f, "{}", self.section.code)?;

        write!(f, "{}", self.section.data)?;

        writeln!(f, "-----ops------------------")?;
        let mut level = 0isize;
        for item in self.ops.iter().enumerate() {
            writeln!(
                f,
                "{} {}{:?}",
                item.0,
                "    ".repeat(level as usize),
                item.1
            )?;
            match item.1 {
                Opcode::Block(_, _) => level += 1,
                Opcode::Loop(_, _) => level += 1,
                Opcode::If(_, _) => level += 1,
                Opcode::Else(_) => level += 1,
                Opcode::End(_) => level -= 1,
                Opcode::Br(_, _) => level -= 1,
                Opcode::BrIf(_, _) => level -= 1,
                Opcode::BrTable(_, _, _) => level -= 1,
                Opcode::Return => level -= 1,
                _ => {}
            }
            level = level.max(0) as isize;
        }

        Ok(())
    }
}

pub enum ImportKind {
    Func(fn(module: &mut WasmModule, arg: &Vec<WasmValue>) -> Vec<WasmValue>),
    Value(WasmValue),
}
pub type ImportObject = HashMap<String, HashMap<String, ImportKind>>;

impl WasmModule {
    pub fn instance(&mut self, import_object: Option<ImportObject>) {
        self.pc = 0;
        self.sp = 0;
        self.csp = 0;
        self.fp = 0;
        self.stack_check();

        let mut section = std::mem::take(&mut self.section);

        for ipt in section.import.entries.iter() {
            let v = import_object
                .as_ref()
                .unwrap()
                .get(&ipt.mod_name)
                .unwrap()
                .get(&ipt.field_name)
                .unwrap();
            match &ipt.kind {
                import::Kind::Func(tyidx) => match v {
                    ImportKind::Func(f) => {
                        self.func.push(FuncKind::Import(*tyidx, *f));
                    }
                    ImportKind::Value(_) => todo!(),
                },
                import::Kind::Table(_, _) => {
                    // let mut buf = Vec::with_capacity(table.limits.maximum as usize);
                    // buf.resize(table.limits.minimum as usize, 0);
                    // self.table.push(buf);
                }
                import::Kind::Memory(mem) => {
                    let mut buf = Vec::with_capacity(mem.maximum as usize * PAGE_SIZE);
                    buf.resize(mem.minimum as usize, 0);
                    self.mem.push(buf);
                }
                import::Kind::Global(g) => match v {
                    ImportKind::Func(_) => todo!(),
                    ImportKind::Value(v) => {
                        self.global.push(if g.mutability {
                            Global::Var(v.clone())
                        } else {
                            Global::Const(v.clone())
                        });
                    }
                },
            }
        }

        for (index, ty) in section.func.entries.iter().enumerate() {
            let code = std::mem::take(&mut section.code.entries[index]);
            self.func.push(FuncKind::Local((*ty, code)));
        }

        // init global
        for g in section.global.entries.iter() {
            self.run(g.expr.0);
            let r = self.stack[self.sp].clone();
            self.sp -= 1;
            self.global.push(if g.mutability {
                Global::Var(r)
            } else {
                Global::Const(r)
            });
        }

        // init table
        for table in section.table.entries.iter() {
            //
            let mut buf = Vec::with_capacity(table.limits.maximum as usize);
            buf.resize(table.limits.minimum as usize, 0);
            self.table.push(buf);
        }

        for ele in section.element.entries.iter() {
            match ele {
                section::element::Element::E0x00(ele) => {
                    let opcode = &ele.ele.0;
                    self.run(opcode.0);
                    let offset = &self.stack[self.sp];
                    self.sp -= 1;
                    if let WasmValue::U32(v) = offset {
                        for i in 0..ele.ele.1.len() {
                            self.table[0][*v as usize + i] = ele.ele.1[i];
                        }
                    } else if let WasmValue::I32(v) = offset {
                        for i in 0..ele.ele.1.len() {
                            self.table[0][*v as usize + i] = ele.ele.1[i];
                        }
                    }
                }
                // section::element::Element::E0x01() => {},
                // section::element::Element::E0x02(_) => todo!(),
                // section::element::Element::E0x03(_) => todo!(),
                // section::element::Element::E0x04(_) => todo!(),
                // section::element::Element::E0x05(_) => todo!(),
                // section::element::Element::E0x06(_) => todo!(),
                // section::element::Element::E0x07(_) => todo!(),
                _ => {}
            }
        }

        // init memory
        for mem in section.memory.entries.iter() {
            let mut buf = Vec::with_capacity(mem.limits.maximum as usize * PAGE_SIZE);
            buf.resize(mem.limits.minimum as usize * PAGE_SIZE, 0);
            self.mem.push(buf);
        }

        for data in section.data.entries.iter() {
            match &data.kind {
                section::data::DataKind::Expr(code, bytes) => {
                    self.run(code.0);
                    let offset = &self.stack[self.sp];
                    self.sp -= 1;
                    if let WasmValue::I32(offset) = offset {
                        let cap = self.mem[0].capacity();
                        let new_len = (*offset as usize + bytes.len()).min(cap);
                        if self.mem[0].len() < new_len {
                            self.mem[0].resize(new_len, 0);
                        }
                        for i in 0..bytes.len() {
                            self.mem[0][*offset as usize + i] = bytes[i];
                        }
                    }
                }
                section::data::DataKind::Vec(_) => todo!(),
                section::data::DataKind::MemIdx(_, _, _) => todo!(),
            }
        }

        for export in section.export.entries.iter() {
            self.exports
                .insert(export.name.clone(), export.kind.clone());
        }
        self.section = section;
    }
    pub fn stack_check(&mut self) {
        if self.stack.len() <= self.sp {
            self.stack.resize_with(self.sp + 512, Default::default);
        }
    }
    fn jump(&mut self, offset: usize) {
        let op = &self.ops[offset];
        match op {
            Opcode::Block(_, location) | Opcode::If(_, location) | Opcode::Else(location) => {
                self.pc = location.2;
            }
            Opcode::Loop(_, l) => self.pc = l.0,
            _ => {}
        }
    }
    pub fn run(&mut self, offset: usize) {
        self.pc = offset;
        loop {
            let op = &self.ops[self.pc];
            #[cfg(debug_assertions)]
            {
                print!("\x1b[2J");
                print!("\x1b[H");
                println!(
                    "{} =  {:?}",
                    self.pc,
                    self.stack[self.fp..self.sp + 1].to_vec()
                );
                println!("next op : {}  {:?}", self.pc, op);
            }
            match op {
                Opcode::Unreachable => panic!("RuntimeError:Unreachable at {}", self.sp),
                Opcode::Nop => {}
                Opcode::Block(_, _b) => {}
                Opcode::Loop(_, _l) => {}
                Opcode::If(_ty, ifcode) => {
                    let result = self.stack[self.sp];
                    self.sp -= 1;
                    if let WasmValue::I32(v) = result {
                        self.pc = if v > 0 { ifcode.0 } else { ifcode.1 };
                        continue;
                    }
                }
                Opcode::Else(_) => {}
                Opcode::End(end) => {
                    if *end == offset {
                        return;
                    }
                }
                Opcode::Br(_l, end) => {
                    self.jump(*end);
                    continue;
                }
                Opcode::BrIf(_l, end) => {
                    let result = self.stack[self.sp];
                    self.sp -= 1;
                    if let WasmValue::I32(v) = result {
                        if v > 0 {
                            self.jump(*end);
                            continue;
                        }
                    }
                }
                Opcode::BrTable(count, entries, dft) => {
                    let tar = self.stack[self.sp];
                    self.sp -= 1;
                    if let WasmValue::I32(v) = tar {
                        if (v as usize) < *count {
                            let did = entries[v as usize];
                            self.jump(did.1);
                        } else {
                            self.jump(dft.1);
                        }
                        continue;
                    }
                }
                Opcode::Return => break,
                Opcode::Call(idx) => {
                    let res = self.call(*idx as usize);
                    for i in 0..res.len() {
                        // push return value and clear stack
                        self.sp += 1;
                        self.stack[self.sp] = res[i];
                    }
                }
                Opcode::CallIndirect(_tyidx, tableidx) => {
                    let idx = self.stack[self.sp];
                    self.sp -= 1;
                    if let WasmValue::I32(idx) = idx {
                        let idx = self.table[*tableidx as usize][idx as usize];
                        let res = self.call(idx);
                        for i in 0..res.len() {
                            // push return value and clear stack
                            self.sp += 1;
                            self.stack[self.sp] = res[i];
                        }
                    }
                }
                Opcode::RefNull(_) => todo!("Opcode::RefNull"),
                Opcode::RefIsNull => todo!("Opcode::RefIsNull"),
                Opcode::RefFunc(_) => todo!("Opcode::RefFunc"),
                Opcode::Drop => {
                    self.sp -= 1;
                }
                Opcode::Select => {
                    let con = self.stack[self.sp];
                    let mid = self.stack[self.sp - 1];
                    let bot = self.stack[self.sp - 2];
                    self.sp = self.sp - 2;
                    if con > WasmValue::I32(0) {
                        self.stack[self.sp] = bot;
                    } else {
                        self.stack[self.sp] = mid;
                    }
                }
                Opcode::SelectType(_, _) => todo!("Opcode::SelectType"),
                Opcode::LocalGet(idx) => {
                    // 将指定局部变量压入到操作数栈顶
                    self.sp += 1;
                    self.stack[self.sp] = self.stack[self.fp + *idx as usize];
                }
                Opcode::LocalSet(idx) => {
                    // 将操作数栈顶的值弹出并保存到指定局部变量中
                    self.stack[self.fp + *idx as usize] = self.stack[self.sp];
                    self.sp -= 1;
                }
                Opcode::LocalTee(idx) => {
                    // 将操作数栈顶值保存到指定局部变量中，但不弹出栈顶值
                    self.stack[self.fp + *idx as usize] = self.stack[self.sp];
                }
                Opcode::GlobalGet(v) => {
                    // 将指定全局变量压入到操作数栈顶
                    let r = &self.global[*v as usize];
                    let r = match r {
                        Global::Const(v) => v,
                        Global::Var(v) => v,
                    };
                    self.sp += 1;
                    self.stack[self.sp] = r.clone();
                }
                Opcode::GlobalSet(idx) => {
                    // 操作数栈顶的值弹出并保存到指定全局变量中
                    let v = self.stack[self.sp];
                    self.sp -= 1;
                    self.global[*idx as usize] = Global::Var(v);
                }
                Opcode::TableGet(_) => todo!("Opcode::TableGet"),
                Opcode::TableSet(_) => todo!("Opcode::TableSet"),
                Opcode::I32Load(_, offset) => {
                    let addr = self.stack[self.sp];
                    self.stack[self.sp] = match addr {
                        WasmValue::I32(v) => {
                            self.mem_read((offset + v as u32) as usize, WasmValue::I32(0))
                        }
                        WasmValue::U32(v) => {
                            self.mem_read((offset + v) as usize, WasmValue::I32(0))
                        }
                        _ => todo!(),
                    };
                }
                Opcode::I64Load(_, offset) => {
                    let addr = self.stack[self.sp];
                    self.stack[self.sp] = match addr {
                        WasmValue::I32(v) => {
                            self.mem_read((offset + v as u32) as usize, WasmValue::I64(0))
                        }
                        WasmValue::U32(v) => {
                            self.mem_read((offset + v) as usize, WasmValue::I64(0))
                        }
                        _ => todo!(),
                    };
                }
                Opcode::F32Load(_, offset) => {
                    let addr = self.stack[self.sp];
                    self.stack[self.sp] = match addr {
                        WasmValue::I32(v) => {
                            self.mem_read((offset + v as u32) as usize, WasmValue::F32(0.0))
                        }
                        WasmValue::U32(v) => {
                            self.mem_read((offset + v) as usize, WasmValue::F32(0.0))
                        }
                        _ => todo!(),
                    };
                }
                Opcode::F64Load(_, offset) => {
                    let addr = self.stack[self.sp];
                    self.stack[self.sp] = match addr {
                        WasmValue::I32(v) => {
                            self.mem_read((offset + v as u32) as usize, WasmValue::F64(0.0))
                        }
                        WasmValue::U32(v) => {
                            self.mem_read((offset + v) as usize, WasmValue::F64(0.0))
                        }
                        _ => todo!(),
                    };
                }
                Opcode::I32Load8s(_, offset) => {
                    let addr = self.stack[self.sp];
                    self.stack[self.sp] = match addr {
                        WasmValue::I32(v) => {
                            let mut byte = self.mem[0][(offset + v as u32) as usize];
                            if byte & 0b1000_0000 > 0 {
                                byte = byte & 0b0111_1111;
                                let byte = !byte as i32;
                                let byte = !byte;
                                WasmValue::I32(byte)
                            } else {
                                WasmValue::I32(byte as i32)
                            }
                        }
                        WasmValue::U32(v) => {
                            let mut byte = self.mem[0][(offset + v as u32) as usize];
                            if byte & 0b1000_0000 > 0 {
                                byte = byte & 0b0111_1111;
                                let byte = !byte as i32;
                                let byte = !byte;
                                WasmValue::I32(byte)
                            } else {
                                WasmValue::I32(byte as i32)
                            }
                        }
                        _ => todo!(),
                    };
                }
                Opcode::I32Load8u(_, offset) => {
                    let addr = self.stack[self.sp];

                    self.stack[self.sp] = match addr {
                        WasmValue::I32(v) => {
                            let byte = self.mem[0][(offset + v as u32) as usize];
                            WasmValue::I32(byte as i32)
                        }
                        WasmValue::U32(v) => {
                            let byte = self.mem[0][(offset + v) as usize];
                            WasmValue::I32(byte as i32)
                        }
                        _ => todo!(),
                    };
                }
                Opcode::I32Load16s(_, _) => todo!("Opcode::I32Load16s"),
                Opcode::I32Load16u(_, _) => todo!("Opcode::I32Load16u"),
                Opcode::I64Load8s(_, _) => todo!("Opcode::I64Load8s"),
                Opcode::I64Load8u(_, _) => todo!("Opcode::I64Load8u"),
                Opcode::I64Load16s(_, _) => todo!("Opcode::I64Load16s"),
                Opcode::I64Load16u(_, _) => todo!("Opcode::I64Load16u"),
                Opcode::I64Load32s(_, offset) => {
                    let addr = self.stack[self.sp];

                    self.stack[self.sp] = match addr {
                        WasmValue::I32(v) => {
                            let byte = self.mem[0]
                                [(offset + v as u32) as usize..(4 + offset + v as u32) as usize]
                                .to_vec();
                            let val = i32::from_le_bytes(byte.try_into().unwrap());
                            let val = if val < 0 {
                                val as u64 | 0xffffffff_00000000u64
                            } else {
                                val as u64
                            };
                            WasmValue::I64(val as i64)
                        }
                        WasmValue::U32(v) => {
                            let byte = self.mem[0]
                                [(offset + v) as usize..(4 + offset + v) as usize]
                                .to_vec();
                            let val = i32::from_le_bytes(byte.try_into().unwrap());
                            let val = if val < 0 {
                                val as u64 | 0xffffffff_00000000u64
                            } else {
                                val as u64
                            };
                            WasmValue::I64(val as i64)
                        }
                        _ => todo!(),
                    };
                }
                Opcode::I64Load32u(_, _) => todo!("Opcode::I64Load32u"),
                Opcode::I32Store(_align, offset) => {
                    let value = self.stack[self.sp];
                    let addr = self.stack[self.sp - 1];
                    self.sp -= 2;
                    match addr {
                        WasmValue::NOP => todo!("WasmValue::NOP"),
                        WasmValue::I32(v) => {
                            self.mem_write((offset + v as u32) as usize, &value);
                        }
                        WasmValue::U32(v) => {
                            self.mem_write((offset + v) as usize, &value);
                        }
                        WasmValue::I64(_) => todo!("WasmValue::I64"),
                        WasmValue::U64(_) => todo!("WasmValue::U64"),
                        WasmValue::F32(_) => todo!("WasmValue::F32"),
                        WasmValue::F64(_) => todo!("WasmValue::F64"),
                        WasmValue::V128(_) => todo!("WasmValue::V128"),
                    }
                }
                Opcode::I64Store(_align, offset) => {
                    let value = self.stack[self.sp];
                    let addr = self.stack[self.sp - 1];
                    self.sp -= 2;
                    match addr {
                        WasmValue::NOP => todo!("WasmValue::NOP"),
                        WasmValue::I32(v) => {
                            self.mem_write((offset + v as u32) as usize, &value);
                        }
                        WasmValue::U32(v) => {
                            self.mem_write((offset + v) as usize, &value);
                        }
                        WasmValue::I64(_) => todo!("WasmValue::I64"),
                        WasmValue::U64(_) => todo!("WasmValue::U64"),
                        WasmValue::F32(_) => todo!("WasmValue::F32"),
                        WasmValue::F64(_) => todo!("WasmValue::F64"),
                        WasmValue::V128(_) => todo!("WasmValue::V128"),
                    }
                }
                Opcode::F32Store(_, _) => todo!("Opcode::F32Store"),
                Opcode::F64Store(_, _) => todo!("Opcode::F64Store"),
                Opcode::I32Store8(_, offset) => {
                    // store last 8bits
                    let value = self.stack[self.sp];
                    let addr = self.stack[self.sp - 1];
                    self.sp -= 2;
                    let offset = *offset;
                    if let (WasmValue::U32(addr), WasmValue::I32(val)) = (addr, value) {
                        let val = val.to_le_bytes().to_vec()[0];
                        self.mem[0][(addr as u32 + offset) as usize] = val;
                    }
                    if let (WasmValue::I32(addr), WasmValue::I32(val)) = (addr, value) {
                        let val = val.to_le_bytes().to_vec()[0];
                        self.mem[0][(addr as u32 + offset) as usize] = val;
                    }
                }
                Opcode::I32Store16(_, _) => todo!("Opcode::I32Store16"),
                Opcode::I64Store8(_, _) => todo!("Opcode::I64Store8"),
                Opcode::I64Store16(_, _) => todo!("Opcode::I64Store16"),
                Opcode::I64Store32(_, _) => todo!("Opcode::I64Store32"),
                Opcode::MemorySize => todo!("Opcode::MemorySize"),
                Opcode::MemoryGrow => todo!("Opcode::MemoryGrow"),
                Opcode::I32Const(value) => {
                    self.sp += 1;
                    self.stack[self.sp] = WasmValue::I32(*value);
                }
                Opcode::I64Const(val) => {
                    self.sp += 1;
                    self.stack[self.sp] = WasmValue::I64(*val);
                }
                Opcode::F32Const(val) => {
                    self.sp += 1;
                    self.stack[self.sp] = WasmValue::F32(*val);
                }
                Opcode::F64Const(val) => {
                    self.sp += 1;
                    self.stack[self.sp] = WasmValue::F64(*val);
                }
                Opcode::I32Eqz | Opcode::I64Eqz => {
                    // is or else not zero
                    let val = self.stack[self.sp];
                    if let WasmValue::I32(val) = val {
                        self.stack[self.sp] = WasmValue::I32(if val == 0 { 1 } else { 0 });
                    }
                    if let WasmValue::I64(val) = val {
                        self.stack[self.sp] = WasmValue::I32(if val == 0 { 1 } else { 0 });
                    }
                }
                Opcode::I32Eq | Opcode::I64Eq | Opcode::F32Eq | Opcode::F64Eq => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = WasmValue::I32(if v1 == v2 { 1 } else { 0 });
                }
                Opcode::I32Ne | Opcode::I64Ne | Opcode::F32Ne | Opcode::F64Ne => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = WasmValue::I32(if v1 != v2 { 1 } else { 0 });
                }
                Opcode::I32Lts | Opcode::I64Lts => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = WasmValue::I32(if v1 < v2 { 1 } else { 0 });
                }
                Opcode::I32Ltu | Opcode::I64Ltu => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = WasmValue::I32(if v1 < v2 { 1 } else { 0 });
                }
                Opcode::I32Gts | Opcode::I64Gts => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = WasmValue::I32(if v1 > v2 { 1 } else { 0 });
                }
                Opcode::I32Gtu | Opcode::I64Gtu => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = WasmValue::I32(if v1 > v2 { 1 } else { 0 });
                }
                Opcode::I32Les | Opcode::I64Les => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = WasmValue::I32(if v1 <= v2 { 1 } else { 0 });
                }
                Opcode::I32Leu | Opcode::I64Leu => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = WasmValue::I32(if v1 <= v2 { 1 } else { 0 });
                }
                Opcode::I32Ges | Opcode::I64Ges => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = WasmValue::I32(if v1 >= v2 { 1 } else { 0 });
                }
                Opcode::I32Geu | Opcode::I64Geu => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = WasmValue::I32(if v1 >= v2 { 1 } else { 0 });
                }
                Opcode::F32Lt | Opcode::F64Lt => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = WasmValue::I32(if v1 < v2 { 1 } else { 0 });
                }
                Opcode::F32Gt | Opcode::F64Gt => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = WasmValue::I32(if v1 > v2 { 1 } else { 0 });
                }
                Opcode::F32Le | Opcode::F64Le => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = WasmValue::I32(if v1 <= v2 { 1 } else { 0 });
                }
                Opcode::F32Ge | Opcode::F64Ge => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = WasmValue::I32(if v1 >= v2 { 1 } else { 0 });
                }
                Opcode::I32Clz => todo!("Opcode::I32Clz"),
                Opcode::I32Ctz => todo!("Opcode::I32Ctz"),
                Opcode::I32Popcnt => todo!("Opcode::I32Popcnt"),
                Opcode::I32Add | Opcode::I64Add | Opcode::F32Add | Opcode::F64Add => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = v1 + v2;
                }
                Opcode::I32Sub | Opcode::I64Sub | Opcode::F32Sub | Opcode::F64Sub => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = v1 - v2;
                }
                Opcode::I32Mul | Opcode::I64Mul | Opcode::F32Mul | Opcode::F64Mul => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = v1 * v2;
                }
                Opcode::I32DivS | Opcode::I64DivS | Opcode::F32Div | Opcode::F64Div => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = v1 / v2;
                }
                Opcode::I32DivU | Opcode::I64DivU => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = v1 / v2;
                }
                Opcode::I32RemS => todo!("Opcode::I32RemS"),
                Opcode::I32RemU => todo!("Opcode::I32RemU"),
                Opcode::I32And => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = v1 & v2;
                }
                Opcode::I32Or => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = v1 | v2;
                }
                Opcode::I32Xor => {
                    let v1 = self.stack[self.sp - 1];
                    let v2 = self.stack[self.sp];
                    self.sp -= 1;
                    self.stack[self.sp] = v1 ^ v2;
                }
                Opcode::I32Shl => {
                    let val = self.stack[self.sp - 1];
                    let shift = self.stack[self.sp];
                    self.stack[self.sp - 1] = val << shift;
                    self.sp -= 1;
                }
                Opcode::I32ShlS => todo!("Opcode::I32ShlS"),
                Opcode::I32ShlU => todo!("Opcode::I32ShlU"),
                Opcode::I32Rotl => todo!("Opcode::I32Rotl"),
                Opcode::I32Rotr => todo!("Opcode::I32Rotr"),
                Opcode::I64Clz => todo!("Opcode::I64Clz"),
                Opcode::I64Ctz => todo!("Opcode::I64Ctz"),
                Opcode::I64Popcnt => todo!("Opcode::I64Popcnt"),
                Opcode::I64RemS => todo!("Opcode::I64RemS"),
                Opcode::I64RemU => todo!("Opcode::I64RemU"),
                Opcode::I64And => todo!("Opcode::I64And"),
                Opcode::I64Or => todo!("Opcode::I64Or"),
                Opcode::I64Xor => todo!("Opcode::I64Xor"),
                Opcode::I64Shl => todo!("Opcode::I64Shl"),
                Opcode::I64ShlS => todo!("Opcode::I64ShlS"),
                Opcode::I64ShlU => todo!("Opcode::I64ShlU"),
                Opcode::I64Rotl => todo!("Opcode::I64Rotl"),
                Opcode::I64Rotr => todo!("Opcode::I64Rotr"),
                Opcode::F32Abs => todo!("Opcode::F32Abs"),
                Opcode::F32Neg => todo!("Opcode::F32Neg"),
                Opcode::F32Ceil => todo!("Opcode::F32Ceil"),
                Opcode::F32Floor => todo!("Opcode::F32Floor"),
                Opcode::F32Trunc => todo!("Opcode::F32Trunc"),
                Opcode::F32Nearest => todo!("Opcode::F32Nearest"),
                Opcode::F32Sqrt => todo!("Opcode::F32Sqrt"),
                Opcode::F32Min => todo!("Opcode::F32Min"),
                Opcode::F32Max => todo!("Opcode::F32Max"),
                Opcode::F32Copysign => todo!("Opcode::F32Copysign"),
                Opcode::F64Abs => todo!("Opcode::F64Abs"),
                Opcode::F64Neg => todo!("Opcode::F64Neg"),
                Opcode::F64Ceil => todo!("Opcode::F64Ceil"),
                Opcode::F64Floor => todo!("Opcode::F64Floor"),
                Opcode::F64Trunc => todo!("Opcode::F64Trunc"),
                Opcode::F64Nearest => todo!("Opcode::F64Nearest"),
                Opcode::F64Sqrt => todo!("Opcode::F64Sqrt"),
                Opcode::F64Min => todo!("Opcode::F64Min"),
                Opcode::F64Max => todo!("Opcode::F64Max"),
                Opcode::F64Copysign => todo!("Opcode::F64Copysign"),
                Opcode::I32WrapI64 => {
                    let val = self.stack[self.sp];
                    if let WasmValue::I64(val) = val {
                        self.stack[self.sp] = WasmValue::I32((val & 0x00000000_ffffffffi64) as i32);
                    }
                }
                Opcode::I32TruncF32s => todo!("Opcode::I32TruncF32s"),
                Opcode::I32TruncF32u => todo!("Opcode::I32TruncF32u"),
                Opcode::I32TruncF64s => todo!("Opcode::I32TruncF64s"),
                Opcode::I32TruncF64u => todo!("Opcode::I32TruncF64u"),
                Opcode::I64ExtendsI32s => todo!("Opcode::I64ExtendsI32s"),
                Opcode::I64ExtendsI32u => {
                    let val = self.stack[self.sp];
                    if let WasmValue::I32(val) = val {
                        self.stack[self.sp] = WasmValue::I64(val as i64);
                    }
                }
                Opcode::I64TruncF32s => todo!("Opcode::I64TruncF32s"),
                Opcode::I64TruncF32u => todo!("Opcode::I64TruncF32u"),
                Opcode::I64TruncF64s => todo!("Opcode::I64TruncF64s"),
                Opcode::I64TruncF64u => todo!("Opcode::I64TruncF64u"),
                Opcode::F32ConvertI32s => todo!("Opcode::F32ConvertI32s"),
                Opcode::F32ConvertI32u => todo!("Opcode::F32ConvertI32u"),
                Opcode::F32ConvertI64s => todo!("Opcode::F32ConvertI64s"),
                Opcode::F32ConvertI64u => todo!("Opcode::F32ConvertI64u"),
                Opcode::F32DemoteF64 => todo!("Opcode::F32DemoteF64"),
                Opcode::F64ConvertI32s => todo!("Opcode::F64ConvertI32s"),
                Opcode::F64ConvertI32u => todo!("Opcode::F64ConvertI32u"),
                Opcode::F64ConvertI64s => todo!("Opcode::F64ConvertI64s"),
                Opcode::F64ConvertI64u => todo!("Opcode::F64ConvertI64u"),
                Opcode::F64DemoteF32 => todo!("Opcode::F64DemoteF32"),
                Opcode::I32ReinterpretF32 => todo!("Opcode::I32ReinterpretF32"),
                Opcode::I64ReinterpretF64 => todo!("Opcode::I64ReinterpretF64"),
                Opcode::F32ReinterpretI32 => todo!("Opcode::F32ReinterpretI32"),
                Opcode::F64ReinterpretI64 => todo!("Opcode::F64ReinterpretI64"),
                Opcode::I32Extends8s => todo!("Opcode::I32Extends8s"),
                Opcode::I32Extends16s => todo!("Opcode::I32Extends16s"),
                Opcode::I64Extends8s => todo!("Opcode::I64Extends8s"),
                Opcode::I64Extends16s => todo!("Opcode::I64Extends16s"),
                Opcode::I64Extends32s => todo!("Opcode::I64Extends32s"),
                Opcode::FD(_) => todo!("Opcode::FD"),
                Opcode::I32TruncSatF32s => todo!("Opcode::I32TruncSatF32s"),
                Opcode::I32TruncSatF32u => todo!("Opcode::I32TruncSatF32u"),
                Opcode::I32TruncSatF64s => todo!("Opcode::I32TruncSatF64s"),
                Opcode::I32TruncSatF64u => todo!("Opcode::I32TruncSatF64u"),
                Opcode::I64TruncSatF32s => todo!("Opcode::I64TruncSatF32s"),
                Opcode::I64TruncSatF32u => todo!("Opcode::I64TruncSatF32u"),
                Opcode::I64TruncSatF64s => todo!("Opcode::I64TruncSatF64s"),
                Opcode::I64TruncSatF64u => todo!("Opcode::I64TruncSatF64u"),
                Opcode::MemoryInit(_) => todo!("Opcode::MemoryInit"),
                Opcode::DataDrop(_) => todo!("Opcode::DataDrop"),
                Opcode::MemoryCopy => todo!("Opcode::MemoryCopy"),
                Opcode::MemoryFill => todo!("Opcode::MemoryFill"),
                Opcode::TableInit(_, _) => todo!("Opcode::TableInit"),
                Opcode::ElemDrop(_) => todo!("Opcode::ElemDrop"),
                Opcode::TableCopy(_, _) => todo!("Opcode::TableCopy"),
                Opcode::TableGrow(_) => todo!("Opcode::TableGrow"),
                Opcode::TableSize(_) => todo!("Opcode::TableSize"),
                Opcode::TableFill(_) => todo!("Opcode::TableFill"),
                Opcode::Reserved(_) => todo!("Opcode::Reserved"),
            }
            self.pc += 1;
        }
    }
    fn mem_write(&mut self, offset: usize, value: &WasmValue) {
        let bytes = match value {
            WasmValue::NOP => todo!("WasmValue::NOP"),
            WasmValue::I32(v) => v.to_le_bytes().to_vec(),
            WasmValue::U32(v) => v.to_le_bytes().to_vec(),
            WasmValue::I64(v) => v.to_le_bytes().to_vec(),
            WasmValue::U64(v) => v.to_le_bytes().to_vec(),
            WasmValue::F32(v) => v.to_le_bytes().to_vec(),
            WasmValue::F64(v) => v.to_le_bytes().to_vec(),
            WasmValue::V128(v) => v.to_le_bytes().to_vec(),
        };
        for (index, item) in bytes.iter().enumerate() {
            self.mem[0][offset + index] = *item;
        }
    }
    fn mem_read(&mut self, offset: usize, value: WasmValue) -> WasmValue {
        match value {
            WasmValue::NOP => WasmValue::NOP,
            WasmValue::I32(v) => {
                let mut bytes = v.to_le_bytes().to_vec();
                for index in 0..bytes.len() {
                    bytes[index] = self.mem[0][offset + index];
                }
                WasmValue::I32(i32::from_le_bytes(bytes.try_into().unwrap()))
            }
            WasmValue::U32(v) => {
                let mut bytes = v.to_le_bytes().to_vec();
                for index in 0..bytes.len() {
                    bytes[index] = self.mem[0][offset + index];
                }
                WasmValue::U32(u32::from_le_bytes(bytes.try_into().unwrap()))
            }
            WasmValue::I64(v) => {
                let mut bytes = v.to_le_bytes().to_vec();
                for index in 0..bytes.len() {
                    bytes[index] = self.mem[0][offset + index];
                }
                WasmValue::I64(i64::from_le_bytes(bytes.try_into().unwrap()))
            }
            WasmValue::U64(v) => {
                let mut bytes = v.to_le_bytes().to_vec();
                for index in 0..bytes.len() {
                    bytes[index] = self.mem[0][offset + index];
                }
                WasmValue::U64(u64::from_le_bytes(bytes.try_into().unwrap()))
            }
            WasmValue::F32(v) => {
                let mut bytes = v.to_le_bytes().to_vec();
                for index in 0..bytes.len() {
                    bytes[index] = self.mem[0][offset + index];
                }
                WasmValue::F32(f32::from_le_bytes(bytes.try_into().unwrap()))
            }
            WasmValue::F64(v) => {
                let mut bytes = v.to_le_bytes().to_vec();
                for index in 0..bytes.len() {
                    bytes[index] = self.mem[0][offset + index];
                }
                WasmValue::F64(f64::from_le_bytes(bytes.try_into().unwrap()))
            }
            WasmValue::V128(v) => {
                let mut bytes = v.to_le_bytes().to_vec();
                for index in 0..bytes.len() {
                    bytes[index] = self.mem[0][offset + index];
                }
                WasmValue::V128(i128::from_le_bytes(bytes.try_into().unwrap()))
            }
        }
    }
    pub fn call(&mut self, idx: usize) -> Vec<WasmValue> {
        let func = &self.func[idx];
        let pc = self.pc;
        let fp = self.fp;
        let sp = self.sp;
        match func {
            FuncKind::Import(ty, f) => {
                let param_count = self.section.types.entries[*ty].param_count as usize;
                // let result_count = self.section.types.entries[*ty].result_count as usize;
                let mut params = vec![];
                self.fp = self.sp - param_count + 1;

                for i in 0..param_count {
                    params.push(self.stack[self.fp + i].clone());
                }
                let res = f(self, &params);
                self.pc = pc;
                self.fp = fp;
                self.sp = sp - param_count;
                // check result count
                res
            }
            FuncKind::Local((ty, func)) => {
                let param_count = self.section.types.entries[*ty].param_count as usize;
                let result_count = self.section.types.entries[*ty].result_count as usize;
                self.fp = self.sp - param_count + 1;
                let new_len = self.sp + 512;

                if self.stack.len() < new_len {
                    self.stack.resize_with(new_len, Default::default);
                }

                for item in func.locales.iter() {
                    use section::typings::ValueType::*;
                    for _ in 0..item.0 {
                        self.sp += 1;
                        self.stack[self.sp] = match item.1 {
                            ExternRef => todo!("ExternRef"),
                            FuncRef => todo!("FuncRef"),
                            I32 => WasmValue::I32(0),
                            I64 => WasmValue::I64(0),
                            F32 => WasmValue::F32(0.0),
                            F64 => WasmValue::F64(0.0),
                            V128 => WasmValue::V128(0),
                        };
                    }
                }
                #[cfg(test)]
                println!(
                    "call func{idx}({:?}) fp={}, sp={}",
                    self.stack[self.fp..self.fp + param_count].to_vec(),
                    self.fp,
                    self.sp
                );
                self.run(func.code.0);
                self.pc = pc;
                self.fp = fp;
                if result_count == 0 {
                    self.sp = sp - param_count;
                    return vec![];
                }
                let mut res = vec![];
                let mut rsp = self.sp;
                self.sp = sp - param_count;
                for _ in 0..result_count {
                    res.push(self.stack[rsp]);
                    rsp -= 1;
                }
                res
            }
        }
    }
    pub fn start(&mut self) -> anyhow::Result<()> {
        let start = self.exports.get(&"_start".to_string());
        ensure!(
            start.is_some(),
            "must be have `_start` function on run a wasm module"
        );
        let start = start.unwrap();
        ensure!(
            matches!(start, ExportKind::Func(_)),
            "`_start` must be a function"
        );
        // self.stack.();
        self.sp = 0;
        self.fp = 0;
        self.pc = 0;
        self.csp = 0;
        match start {
            ExportKind::Func(idx) => self.call(*idx),
            _ => todo!("not yet impl"),
        };
        Ok(())
    }
}

impl Add for WasmValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        use WasmValue::*;
        match (self, rhs) {
            (I32(v1), I32(v2)) => I32(v1 + v2),
            (U32(v1), U32(v2)) => U32(v1 + v2),
            (I64(v1), I64(v2)) => I64(v1 + v2),
            (U64(v1), U64(v2)) => U64(v1 + v2),
            (F32(v1), F32(v2)) => F32(v1 + v2),
            (F64(v1), F64(v2)) => F64(v1 + v2),
            (V128(v1), V128(v2)) => V128(v1 + v2),
            _ => todo!("{:?} + {:?} not support", self, rhs),
        }
    }
}

impl Sub for WasmValue {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        use WasmValue::*;
        match (self, rhs) {
            (I32(v1), I32(v2)) => I32(v1 - v2),
            (U32(v1), U32(v2)) => U32(v1 - v2),
            (I64(v1), I64(v2)) => I64(v1 - v2),
            (U64(v1), U64(v2)) => U64(v1 - v2),
            (F32(v1), F32(v2)) => F32(v1 - v2),
            (F64(v1), F64(v2)) => F64(v1 - v2),
            (V128(v1), V128(v2)) => V128(v1 - v2),
            _ => todo!("{:?} - {:?} not support", self, rhs),
        }
    }
}
impl Mul for WasmValue {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        use WasmValue::*;
        match (self, rhs) {
            (I32(v1), I32(v2)) => I32(v1 * v2),
            (U32(v1), U32(v2)) => U32(v1 * v2),
            (I64(v1), I64(v2)) => I64(v1 * v2),
            (U64(v1), U64(v2)) => U64(v1 * v2),
            (F32(v1), F32(v2)) => F32(v1 * v2),
            (F64(v1), F64(v2)) => F64(v1 * v2),
            (V128(v1), V128(v2)) => V128(v1 * v2),
            _ => todo!("{:?} * {:?} not support", self, rhs),
        }
    }
}
impl Div for WasmValue {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        use WasmValue::*;
        match (self, rhs) {
            (I32(v1), I32(v2)) => I32(v1 / v2),
            (U32(v1), U32(v2)) => U32(v1 / v2),
            (I64(v1), I64(v2)) => I64(v1 / v2),
            (U64(v1), U64(v2)) => U64(v1 / v2),
            (F32(v1), F32(v2)) => F32(v1 / v2),
            (F64(v1), F64(v2)) => F64(v1 / v2),
            (V128(v1), V128(v2)) => V128(v1 / v2),
            _ => todo!("{:?} / {:?} not support", self, rhs),
        }
    }
}

impl BitAnd for WasmValue {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        use WasmValue::*;
        match (self, rhs) {
            (I32(v1), I32(v2)) => I32(v1 & v2),
            (U32(v1), U32(v2)) => U32(v1 & v2),
            (I64(v1), I64(v2)) => I64(v1 & v2),
            (U64(v1), U64(v2)) => U64(v1 & v2),
            _ => todo!("{:?} & {:?} not support", self, rhs),
        }
    }
}

impl BitOr for WasmValue {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        use WasmValue::*;
        match (self, rhs) {
            (I32(v1), I32(v2)) => I32(v1 | v2),
            (U32(v1), U32(v2)) => U32(v1 | v2),
            (I64(v1), I64(v2)) => I64(v1 | v2),
            (U64(v1), U64(v2)) => U64(v1 | v2),
            _ => todo!("{:?} & {:?} not support", self, rhs),
        }
    }
}

impl BitXor for WasmValue {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        use WasmValue::*;
        match (self, rhs) {
            (I32(v1), I32(v2)) => I32(v1 ^ v2),
            (U32(v1), U32(v2)) => U32(v1 ^ v2),
            (I64(v1), I64(v2)) => I64(v1 ^ v2),
            (U64(v1), U64(v2)) => U64(v1 ^ v2),
            _ => todo!("{:?} & {:?} not support", self, rhs),
        }
    }
}

impl PartialOrd for WasmValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use WasmValue::*;
        match (self, other) {
            (NOP, NOP) => todo!(),
            (I32(v1), I32(v2)) => {
                if v1 == v2 {
                    return Some(Ordering::Equal);
                } else if v1 > v2 {
                    return Some(Ordering::Greater);
                } else {
                    return Some(Ordering::Less);
                }
            }
            (U32(v1), U32(v2)) => {
                if v1 == v2 {
                    return Some(Ordering::Equal);
                } else if v1 > v2 {
                    return Some(Ordering::Greater);
                } else {
                    return Some(Ordering::Less);
                }
            }
            (I64(v1), I64(v2)) => {
                if v1 == v2 {
                    return Some(Ordering::Equal);
                } else if v1 > v2 {
                    return Some(Ordering::Greater);
                } else {
                    return Some(Ordering::Less);
                }
            }
            (U64(v1), U64(v2)) => {
                if v1 == v2 {
                    return Some(Ordering::Equal);
                } else if v1 > v2 {
                    return Some(Ordering::Greater);
                } else {
                    return Some(Ordering::Less);
                }
            }
            (F32(v1), F32(v2)) => {
                if v1 == v2 {
                    return Some(Ordering::Equal);
                } else if v1 > v2 {
                    return Some(Ordering::Greater);
                } else {
                    return Some(Ordering::Less);
                }
            }
            (F64(v1), F64(v2)) => {
                if v1 == v2 {
                    return Some(Ordering::Equal);
                } else if v1 > v2 {
                    return Some(Ordering::Greater);
                } else {
                    return Some(Ordering::Less);
                }
            }
            (V128(v1), V128(v2)) => {
                if v1 == v2 {
                    return Some(Ordering::Equal);
                } else if v1 > v2 {
                    return Some(Ordering::Greater);
                } else {
                    return Some(Ordering::Less);
                }
            }
            (v1, v2) => todo!("{v1:?} compare {v2:?} isn't support"),
        }
    }

    fn lt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Less))
    }

    fn le(&self, other: &Self) -> bool {
        matches!(
            self.partial_cmp(other),
            Some(Ordering::Less | Ordering::Equal)
        )
    }

    fn gt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Greater))
    }

    fn ge(&self, other: &Self) -> bool {
        matches!(
            self.partial_cmp(other),
            Some(Ordering::Greater | Ordering::Equal)
        )
    }
}

impl Shl for WasmValue {
    type Output = WasmValue;

    fn shl(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (WasmValue::I32(a), WasmValue::I32(b)) => WasmValue::I32(a << b),
            (WasmValue::U32(_), WasmValue::U32(_)) => todo!(),
            (WasmValue::I64(_), WasmValue::I64(_)) => todo!(),
            (WasmValue::U64(_), WasmValue::U64(_)) => todo!(),
            (WasmValue::F32(_), WasmValue::F32(_)) => todo!(),
            (WasmValue::F64(_), WasmValue::F64(_)) => todo!(),
            (WasmValue::V128(_), WasmValue::V128(_)) => todo!(),
            _ => todo!("{:?} << {:?}", self, rhs),
        }
    }
}

use anyhow::{anyhow, ensure};

use super::{
    opcode::{BlockType, Location, Opcode, FD},
    ByteParse, ByteRead,
};

pub(crate) trait ByteCode: ByteParse + ByteRead {
    fn parse_code(
        &mut self,
        ops: &mut Vec<Opcode>,
        blocks: &mut Vec<usize>,
    ) -> anyhow::Result<(usize, usize, usize)> {
        // let mut opcode = vec![];
        let mut pos = (ops.len(), 0, 0);
        blocks.push(0.max(pos.0 as isize - 1) as usize);
        while self.offset() < self.length() {
            let code = self.read_byte()?;
            match code {
                0x00 => ops.push(Opcode::Unreachable), /* unreachable */
                0x01 => ops.push(Opcode::Nop),         /* nop */
                0x02 => {
                    /* block <bt:blocktype> in*:instr end */
                    let bt = self.read_leb_u32()?;
                    ops.push(Opcode::Block(BlockType::from_u32(bt), Location(0, 0, 0)));
                    let last = ops.len() - 1;
                    self.parse_code(ops, blocks)?;
                    ops[last] = Opcode::Block(
                        BlockType::from_u32(bt),
                        Location(last + 1, ops.len() - 1, ops.len() - 1),
                    );
                }
                0x03 => {
                    /* loop <bt:blocktype> in*:instr end */
                    let bt = self.read_leb_u32()?;
                    ops.push(Opcode::Loop(BlockType::from_u32(bt), Location(0, 0, 0)));
                    let last = ops.len() - 1;
                    self.parse_code(ops, blocks)?;
                    ops[last] = Opcode::Loop(
                        BlockType::from_u32(bt),
                        Location(last + 1, ops.len() - 1, ops.len() - 1),
                    );
                }
                0x04 => {
                    /* if <bt:blocktype> in*:instr else in*:instr end */
                    let bt = self.read_leb_u32()?;
                    ops.push(Opcode::If(
                        BlockType::from_u32(bt),
                        Location(ops.len(), 0, 0),
                    ));
                    let last = ops.len() - 1;
                    let (_, end, _) = self.parse_code(ops, blocks)?;

                    ops[last] = Opcode::If(
                        BlockType::from_u32(bt),
                        Location(last + 1, end, ops.len() - 1),
                    );
                }
                0x05 => {
                    /* else */
                    ops.push(Opcode::Br(0, *blocks.last().unwrap())); //  if {block  end} {else end} end
                    ops.push(Opcode::Else(Location(0, 0, 0)));
                    let last = ops.len() - 1;
                    self.parse_code(ops, blocks)?;
                    ops[last] = Opcode::Else(Location(last + 1, ops.len() - 1, ops.len() - 1));

                    pos.1 = last;
                    blocks.pop();
                    break;
                }
                0x0b => {
                    /* end */
                    ops.push(Opcode::End(pos.0));
                    blocks.pop();
                    pos.1 = ops.len() - 1;
                    break;
                }
                0x0c => {
                    /* br <l:lableidx> */
                    let label = self.read_leb_u32()? as usize;
                    let len = blocks.len();
                    ops.push(Opcode::Br(label, blocks[len - 1 - label]));
                }
                0x0d => {
                    /* br_if <l:lableidx> */
                    let label = self.read_leb_u32()? as usize;
                    let len = blocks.len();
                    ops.push(Opcode::BrIf(label, blocks[len - label - 1]));
                }
                0x0e => {
                    /* br_table <l*:vec(lableidx)> <lN:lableidx> */
                    let count = self.read_leb_u32()? as usize;
                    // ensure!(count <= MAX_BR_TABLE, "br table overflow {}", count);
                    let mut entries = vec![];
                    let len = blocks.len();
                    for _ in 0..count {
                        let i = self.read_leb_u32()? as usize;
                        entries.push((i, blocks[len - i - 1]))
                    }
                    let default = self.read_leb_u32()? as usize;
                    ops.push(Opcode::BrTable(
                        count,
                        entries,
                        (default, blocks[len - default - 1]),
                    ));
                }
                0x0f => ops.push(Opcode::Return), /* return */
                0x10 => ops.push(Opcode::Call(self.read_leb_u32()?)), /* call <x:funcidx> */
                0x11 => {
                    /* call_indirect <x:typeidx> <y:tableidx> */
                    ops.push(Opcode::CallIndirect(
                        self.read_leb_u32()?,
                        self.read_leb_u32()?,
                    ))
                }
                0xd0 => {
                    /* ref.null t:reftype */
                    let byte = self.read_byte()?;
                    ensure!(byte == 0x70 || byte == 0x6f, "");
                    ops.push(Opcode::RefNull(byte));
                }
                0xd1 => ops.push(Opcode::RefIsNull), /* ref.is_null */
                0xd2 => ops.push(Opcode::RefFunc(self.read_leb_u32()?)), /* ref.func x:funcidx */
                0x1a => ops.push(Opcode::Drop),      /* drop */
                0x1b => ops.push(Opcode::Select),    /* select */
                0x1c => {
                    /* select t*:vec(valtype) */
                    let count = self.read_leb_u32()? as usize;
                    let mut types = vec![];
                    for _ in 0..count {
                        types.push(self.read_byte()? as usize)
                    }
                    ops.push(Opcode::SelectType(count, types));
                }
                0x20 => ops.push(Opcode::LocalGet(self.read_leb_u32()?)), /* local.get <x:localidx> */
                0x21 => ops.push(Opcode::LocalSet(self.read_leb_u32()?)), /* local.set <x:localidx> */
                0x22 => ops.push(Opcode::LocalTee(self.read_leb_u32()?)), /* local.tee <x:localidx> */
                0x23 => ops.push(Opcode::GlobalGet(self.read_leb_u32()?)), /* global.get <x:globalidx> */
                0x24 => ops.push(Opcode::GlobalSet(self.read_leb_u32()?)), /* global.set <x:globalidx> */
                0x25 => ops.push(Opcode::TableGet(self.read_leb_u32()?)), /* table.get x:tableidx */
                0x26 => ops.push(Opcode::TableSet(self.read_leb_u32()?)), /* table.set x:tableidx */
                0x28 => ops.push(Opcode::I32Load(self.read_leb_u32()?, self.read_leb_u32()?)), /* i32.load m:memarg */
                0x29 => ops.push(Opcode::I64Load(self.read_leb_u32()?, self.read_leb_u32()?)), /* i64.load m:memarg */
                0x2a => ops.push(Opcode::F32Load(self.read_leb_u32()?, self.read_leb_u32()?)), /* f32.load m:memarg */
                0x2b => ops.push(Opcode::F64Load(self.read_leb_u32()?, self.read_leb_u32()?)), /* f64.load m:memarg */
                0x2c => ops.push(Opcode::I32Load8s(
                    self.read_leb_u32()?,
                    self.read_leb_u32()?,
                )), /* i32.load8_s m:memarg */
                0x2d => ops.push(Opcode::I32Load8u(
                    self.read_leb_u32()?,
                    self.read_leb_u32()?,
                )), /* i32.load 8_u m:memarg */
                0x2e => ops.push(Opcode::I32Load16s(
                    self.read_leb_u32()?,
                    self.read_leb_u32()?,
                )), /* i32.load16_s m:memarg */
                0x2f => ops.push(Opcode::I32Load16u(
                    self.read_leb_u32()?,
                    self.read_leb_u32()?,
                )), /* i32.load16_u m:memarg */
                0x30 => ops.push(Opcode::I64Load8s(
                    self.read_leb_u32()?,
                    self.read_leb_u32()?,
                )), /* i64.load8_s m:memarg */
                0x31 => ops.push(Opcode::I64Load8u(
                    self.read_leb_u32()?,
                    self.read_leb_u32()?,
                )), /* i64.load8_u m:memarg */
                0x32 => ops.push(Opcode::I64Load16s(
                    self.read_leb_u32()?,
                    self.read_leb_u32()?,
                )), /* i64.load16_s m:memarg */
                0x33 => ops.push(Opcode::I64Load16u(
                    self.read_leb_u32()?,
                    self.read_leb_u32()?,
                )), /* i64.load16_u m:memarg */
                0x34 => ops.push(Opcode::I64Load32s(
                    self.read_leb_u32()?,
                    self.read_leb_u32()?,
                )), /* i64.load32_s m:memarg */
                0x35 => ops.push(Opcode::I64Load32u(
                    self.read_leb_u32()?,
                    self.read_leb_u32()?,
                )), /* i64.load32_u m:memarg */
                0x36 => ops.push(Opcode::I32Store(self.read_leb_u32()?, self.read_leb_u32()?)), /* i32.store m:memarg */
                0x37 => ops.push(Opcode::I64Store(self.read_leb_u32()?, self.read_leb_u32()?)), /* i64.store m:memarg */
                0x38 => ops.push(Opcode::F32Store(self.read_leb_u32()?, self.read_leb_u32()?)), /* f32.store m:memarg */
                0x39 => ops.push(Opcode::F64Store(self.read_leb_u32()?, self.read_leb_u32()?)), /* f64.store m:memarg */
                0x3a => ops.push(Opcode::I32Store8(
                    self.read_leb_u32()?,
                    self.read_leb_u32()?,
                )), /* i32.store8 m:memarg */
                0x3b => ops.push(Opcode::I32Store16(
                    self.read_leb_u32()?,
                    self.read_leb_u32()?,
                )), /* i32.store16 m:memarg */
                0x3c => ops.push(Opcode::I64Store8(
                    self.read_leb_u32()?,
                    self.read_leb_u32()?,
                )), /* i64.store8 m:memarg */
                0x3d => ops.push(Opcode::I64Store16(
                    self.read_leb_u32()?,
                    self.read_leb_u32()?,
                )), /* i64.store16 m:memarg */
                0x3e => ops.push(Opcode::I64Store32(
                    self.read_leb_u32()?,
                    self.read_leb_u32()?,
                )), /* i64.store32 m:memarg */
                0x3f => ops.push(Opcode::MemorySize), /* memory.size */
                0x40 => ops.push(Opcode::MemoryGrow), /* memory.grow */
                0x41 => ops.push(Opcode::I32Const(self.read_leb_i32()?)), /* i32.const x:i32 */
                0x42 => ops.push(Opcode::I64Const(self.read_leb_i64()?)), /* i64.const x:i64 */
                0x43 => {
                    /* f32.const x:f32 */
                    let bytes = self.read_bytes(4)?;
                    ops.push(Opcode::F32Const(f32::from_le_bytes(
                        bytes.try_into().unwrap(),
                    )));
                }
                0x44 => {
                    /* f64.const x.f64 */
                    let bytes = self.read_bytes(8)?;
                    ops.push(Opcode::F64Const(f64::from_le_bytes(
                        bytes.try_into().unwrap(),
                    )));
                }
                0x45 => ops.push(Opcode::I32Eqz),      /* i32.eqz */
                0x46 => ops.push(Opcode::I32Eq),       /* i32.eq */
                0x47 => ops.push(Opcode::I32Ne),       /* i32.ne */
                0x48 => ops.push(Opcode::I32Lts),      /* i32.lt_s */
                0x49 => ops.push(Opcode::I32Ltu),      /* i32.lt_u */
                0x4a => ops.push(Opcode::I32Gts),      /* i32.gt_s */
                0x4b => ops.push(Opcode::I32Gtu),      /* i32.gt_u */
                0x4c => ops.push(Opcode::I32Les),      /* i32.le_s */
                0x4d => ops.push(Opcode::I32Leu),      /* i32.le_u */
                0x4e => ops.push(Opcode::I32Ges),      /* i32.ge_s */
                0x4f => ops.push(Opcode::I32Geu),      /* i32.ge_u */
                0x50 => ops.push(Opcode::I64Eqz),      /* i64.eqz */
                0x51 => ops.push(Opcode::I64Eq),       /* i64.eq */
                0x52 => ops.push(Opcode::I64Ne),       /* i64.ne */
                0x53 => ops.push(Opcode::I64Lts),      /* i64.lt_s */
                0x54 => ops.push(Opcode::I64Ltu),      /* i64.lt_u */
                0x55 => ops.push(Opcode::I64Gts),      /* i64.gt_s */
                0x56 => ops.push(Opcode::I64Gtu),      /* i64.gt_u */
                0x57 => ops.push(Opcode::I64Les),      /* i64.le_s */
                0x58 => ops.push(Opcode::I64Leu),      /* i64.le_u */
                0x59 => ops.push(Opcode::I64Ges),      /* i64.ge_s */
                0x5a => ops.push(Opcode::I64Geu),      /* i64.ge_u */
                0x5b => ops.push(Opcode::F32Eq),       /* f32.eq */
                0x5c => ops.push(Opcode::F32Ne),       /* f32.ne */
                0x5d => ops.push(Opcode::F32Lt),       /* f32.lt */
                0x5e => ops.push(Opcode::F32Gt),       /* f32.gt */
                0x5f => ops.push(Opcode::F32Le),       /* f32.le */
                0x60 => ops.push(Opcode::F32Ge),       /* f32.ge */
                0x61 => ops.push(Opcode::F64Eq),       /* f64.eq */
                0x62 => ops.push(Opcode::F64Ne),       /* f64.ne */
                0x63 => ops.push(Opcode::F64Lt),       /* f64.lt */
                0x64 => ops.push(Opcode::F64Gt),       /* f64.gt */
                0x65 => ops.push(Opcode::F64Le),       /* f64.le */
                0x66 => ops.push(Opcode::F64Ge),       /* f64.ge */
                0x67 => ops.push(Opcode::I32Clz),      /* i32.clz */
                0x68 => ops.push(Opcode::I32Ctz),      /* i32.ctz */
                0x69 => ops.push(Opcode::I32Popcnt),   /* i32.popcnt */
                0x6a => ops.push(Opcode::I32Add),      /* i32.add */
                0x6b => ops.push(Opcode::I32Sub),      /* i32.sub */
                0x6c => ops.push(Opcode::I32Mul),      /* i32.mul */
                0x6d => ops.push(Opcode::I32DivS),     /* i32.div_s */
                0x6e => ops.push(Opcode::I32DivU),     /* i32.div_u */
                0x6f => ops.push(Opcode::I32RemS),     /* i32.rem_s */
                0x70 => ops.push(Opcode::I32RemU),     /* i32.rem_u */
                0x71 => ops.push(Opcode::I32And),      /* i32.and */
                0x72 => ops.push(Opcode::I32Or),       /* i32.or */
                0x73 => ops.push(Opcode::I32Xor),      /* i32.xor */
                0x74 => ops.push(Opcode::I32Shl),      /* i32.shl */
                0x75 => ops.push(Opcode::I32ShlS),     /* i32.shl_s */
                0x76 => ops.push(Opcode::I32ShlU),     /* i32.shl_u */
                0x77 => ops.push(Opcode::I32Rotl),     /* i32.rotl */
                0x78 => ops.push(Opcode::I32Rotr),     /* i32.rotr */
                0x79 => ops.push(Opcode::I64Clz),      /* i64.clz */
                0x7a => ops.push(Opcode::I64Ctz),      /* i64.ctz */
                0x7b => ops.push(Opcode::I64Popcnt),   /* i64.popcnt */
                0x7c => ops.push(Opcode::I64Add),      /* i64.add */
                0x7d => ops.push(Opcode::I64Sub),      /* i64.sub */
                0x7e => ops.push(Opcode::I64Mul),      /* i64.mul */
                0x7f => ops.push(Opcode::I64DivS),     /* i64.div_s */
                0x80 => ops.push(Opcode::I64DivU),     /* i64.div_u */
                0x81 => ops.push(Opcode::I64RemS),     /* i64.rem_s */
                0x82 => ops.push(Opcode::I64RemU),     /* i64.rem_u */
                0x83 => ops.push(Opcode::I64And),      /* i64.and */
                0x84 => ops.push(Opcode::I64Or),       /* i64.or */
                0x85 => ops.push(Opcode::I64Xor),      /* i64.xor */
                0x86 => ops.push(Opcode::I64Shl),      /* i64.shl */
                0x87 => ops.push(Opcode::I64ShlS),     /* i64.shl_s */
                0x88 => ops.push(Opcode::I64ShlU),     /* i64.shl_u */
                0x89 => ops.push(Opcode::I64Rotl),     /* i64.rotl */
                0x8a => ops.push(Opcode::I64Rotr),     /* i64.rotr */
                0x8b => ops.push(Opcode::F32Abs),      /* f32.abs */
                0x8c => ops.push(Opcode::F32Neg),      /* f32.neg */
                0x8d => ops.push(Opcode::F32Ceil),     /* f32.ceil */
                0x8e => ops.push(Opcode::F32Floor),    /* f32.floor */
                0x8f => ops.push(Opcode::F32Trunc),    /* f32.trunc */
                0x90 => ops.push(Opcode::F32Nearest),  /* f32.nearest */
                0x91 => ops.push(Opcode::F32Sqrt),     /* f32.sqrt */
                0x92 => ops.push(Opcode::F32Add),      /* f32.add */
                0x93 => ops.push(Opcode::F32Sub),      /* f32.sub */
                0x94 => ops.push(Opcode::F32Mul),      /* f32.mul */
                0x95 => ops.push(Opcode::F32Div),      /* f32.div */
                0x96 => ops.push(Opcode::F32Min),      /* f32.min */
                0x97 => ops.push(Opcode::F32Max),      /* f32.max */
                0x98 => ops.push(Opcode::F32Copysign), /* f32.copysign */
                0x99 => ops.push(Opcode::F64Abs),      /* f64.abs */
                0x9a => ops.push(Opcode::F64Neg),      /* f64.neg */
                0x9b => ops.push(Opcode::F64Ceil),     /* f64.ceil */
                0x9c => ops.push(Opcode::F64Floor),    /* f64.floor */
                0x9d => ops.push(Opcode::F64Trunc),    /* f64.trunc */
                0x9e => ops.push(Opcode::F64Nearest),  /* f64.nearest */
                0x9f => ops.push(Opcode::F64Sqrt),     /* f64.sqrt */
                0xa0 => ops.push(Opcode::F64Add),      /* f64.add */
                0xa1 => ops.push(Opcode::F64Sub),      /* f64.sub */
                0xa2 => ops.push(Opcode::F64Mul),      /* f64.mul */
                0xa3 => ops.push(Opcode::F64Div),      /* f64.div */
                0xa4 => ops.push(Opcode::F64Min),      /* f64.min */
                0xa5 => ops.push(Opcode::F64Max),      /* f64.max */
                0xa6 => ops.push(Opcode::F64Copysign), /* f64.copysign */
                0xa7 => ops.push(Opcode::I32WrapI64),  /* i32.wrap_i64 */
                0xa8 => ops.push(Opcode::I32TruncF32s), /* i32.trunc_f32_s */
                0xa9 => ops.push(Opcode::I32TruncF32u), /* i32.trunc_f32_u */
                0xaa => ops.push(Opcode::I32TruncF64s), /* i32.trunc_f64_s */
                0xab => ops.push(Opcode::I32TruncF64u), /* i32.trunc_f64_u */
                0xac => ops.push(Opcode::I64ExtendsI32s), /* i64.extends_i32_s */
                0xad => ops.push(Opcode::I64ExtendsI32u), /* i64.extends_i32_u */
                0xae => ops.push(Opcode::I64TruncF32s), /* i64.trunc_f32_s */
                0xaf => ops.push(Opcode::I64TruncF32u), /* i64.trunc_f32_u */
                0xb0 => ops.push(Opcode::I64TruncF64s), /* i64.trunc_f64_s */
                0xb1 => ops.push(Opcode::I64TruncF64u), /* i64.trunc_f64_u */
                0xb2 => ops.push(Opcode::F32ConvertI32s), /* f32.convert_i32_s */
                0xb3 => ops.push(Opcode::F32ConvertI32u), /* f32.convert_i32_u */
                0xb4 => ops.push(Opcode::F32ConvertI64s), /* f32.convert_i64_s */
                0xb5 => ops.push(Opcode::F32ConvertI64u), /* f32.convert_i64_u */
                0xb6 => ops.push(Opcode::F32DemoteF64), /* f32.demote_f64 */
                0xb7 => ops.push(Opcode::F64ConvertI32s), /* f64.convert_i32_s */
                0xb8 => ops.push(Opcode::F64ConvertI32u), /* f64.convert_i32_u */
                0xb9 => ops.push(Opcode::F64ConvertI64s), /* f64.convert_i64_s */
                0xba => ops.push(Opcode::F64ConvertI64u), /* f64.convert_i64_u */
                0xbb => ops.push(Opcode::F64DemoteF32), /* f64.demote_f32 */
                0xbc => ops.push(Opcode::I32ReinterpretF32), /* i32.reinterpret_f32 */
                0xbd => ops.push(Opcode::I64ReinterpretF64), /* i64.reinterpret_f64 */
                0xbe => ops.push(Opcode::F32ReinterpretI32), /* f32.reinterpret_i32 */
                0xbf => ops.push(Opcode::F64ReinterpretI64), /* f64.reinterpret_i64 */
                0xc0 => ops.push(Opcode::I32Extends8s), /* i32.extends8_s */
                0xc1 => ops.push(Opcode::I32Extends16s), /* i32.extends16_s */
                0xc2 => ops.push(Opcode::I64Extends8s), /* i64.extends8_s */
                0xc3 => ops.push(Opcode::I64Extends16s), /* i64.extends16_s */
                0xc4 => ops.push(Opcode::I64Extends32s), /* i64.extends32_s */
                0xfc => {
                    /* op */
                    let op = self.read_leb_u32()?;
                    ensure!(op <= 17, "Unkonwn op {op}");
                    match op {
                        00 => ops.push(Opcode::I32TruncSatF32s), /* i32.trunc_sat_f32_s */
                        01 => ops.push(Opcode::I32TruncSatF32u), /* i32.trunc_sat_f32_u */
                        02 => ops.push(Opcode::I32TruncSatF64s), /* i32.trunc_sat_f64_s */
                        03 => ops.push(Opcode::I32TruncSatF64u), /* i32.trunc_sat_f64_u */
                        04 => ops.push(Opcode::I64TruncSatF32s), /* i64.trunc_sat_f32_s */
                        05 => ops.push(Opcode::I64TruncSatF32u), /* i64.trunc_sat_f32_u */
                        06 => ops.push(Opcode::I64TruncSatF64s), /* i64.trunc_sat_f64_s */
                        07 => ops.push(Opcode::I64TruncSatF64u), /* i64.trunc_sat_f64_u */
                        08 => ops.push(Opcode::MemoryInit(self.read_leb_u32()? as usize)), /* memory.init x */
                        09 => ops.push(Opcode::DataDrop(self.read_leb_u32()? as usize)), /* data.drop x */
                        10 => ops.push(Opcode::MemoryCopy), /* memory.copy */
                        11 => ops.push(Opcode::MemoryFill), /* memory.fill */
                        12 => ops.push(Opcode::TableInit(
                            self.read_leb_u32()? as usize,
                            self.read_leb_u32()? as usize,
                        )), /* table.init x y */
                        13 => ops.push(Opcode::ElemDrop(self.read_leb_u32()? as usize)), /* elem.drop x */
                        14 => ops.push(Opcode::TableCopy(
                            self.read_leb_u32()? as usize,
                            self.read_leb_u32()? as usize,
                        )), /* table.copy x y */
                        15 => ops.push(Opcode::TableGrow(self.read_leb_u32()? as usize)), /* table.grow x */
                        16 => ops.push(Opcode::TableSize(self.read_leb_u32()? as usize)), /* table.size x */
                        17 => ops.push(Opcode::TableFill(self.read_leb_u32()? as usize)), /* table.fill x */
                        _ => {}
                    }
                }
                0xfd => {
                    let code = self.read_leb_u32()?;
                    ops.push(Opcode::FD(self.parse_fd(code)?))
                }
                0x06..=0x0a | 0x12..=0x19 | 0x1d..=0x1f | 0x27 | 0xc5..=0xcf | 0xd3..=0xfb => {
                    ops.push(Opcode::Reserved(code))
                }
                v => {
                    return Err(anyhow!(
                        "Unkonwn code offset = {}, value = {v:x}",
                        self.offset()
                    ))
                }
            }
        }

        Ok((pos.0, pos.1, ops.len() - 1))
    }
    fn parse_fd(&mut self, code: u32) -> anyhow::Result<FD> {
        match code {
            00 => Ok(FD::V128Load(self.read_leb_u32()?, self.read_leb_u32()?)), // v128.load m:memarg
            01 => Ok(FD::V128Load8x8s(self.read_leb_u32()?, self.read_leb_u32()?)), // v128.load8x8_s m:memarg
            02 => Ok(FD::V128Load8x8u(self.read_leb_u32()?, self.read_leb_u32()?)), // v128.load8x8_u m:memarg
            03 => Ok(FD::V128Load16x4s(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
            )), // v128.load16x4_s m:memarg
            04 => Ok(FD::V128Load16x4u(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
            )), // v128.load16x4_u m:memarg
            05 => Ok(FD::V128Load32x2s(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
            )), // v128.load32x2_s m:memarg
            06 => Ok(FD::V128Load32x2u(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
            )), // v128.load32x2_u m:memarg
            07 => Ok(FD::V128Load8splat(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
            )), // v128.load8_splat m:memarg
            08 => Ok(FD::V128Load16splat(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
            )), // v128.load16_splat m:memarg
            09 => Ok(FD::V128Load32splat(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
            )), // v128.load32_splat m:memarg
            10 => Ok(FD::V128Load64splat(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
            )), // v128.load64_splat m:memarg
            92 => Ok(FD::V128Load32zero(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
            )), // v128.load32_zero m:memarg
            93 => Ok(FD::V128Load64zero(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
            )), // v128.load64_zero m:memarg
            11 => Ok(FD::V128Store(self.read_leb_u32()?, self.read_leb_u32()?)), // v128.store m:memarg
            84 => Ok(FD::V128Load8lane(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
                self.read_byte()?,
            )), // v128.load8_lane m:memarg l:laneidx:byte
            85 => Ok(FD::V128Load16lane(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
                self.read_byte()?,
            )), // v128.load16_lane  m:memarg l:laneidx:byte
            86 => Ok(FD::V128Load32lane(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
                self.read_byte()?,
            )), // v128.load32_lane  m:memarg l:laneidx:byte
            87 => Ok(FD::V128Load64lane(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
                self.read_byte()?,
            )), // v128.load64_lane  m:memarg l:laneidx:byte
            88 => Ok(FD::V128Store8lane(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
                self.read_byte()?,
            )), // v128.store8_lane  m:memarg l:laneidx:byte
            89 => Ok(FD::V128Store16lane(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
                self.read_byte()?,
            )), // v128.store16_lane  m:memarg l:laneidx:byte
            90 => Ok(FD::V128Store32lane(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
                self.read_byte()?,
            )), // v128.store32_lane  m:memarg l:laneidx:byte
            91 => Ok(FD::V128Store64lane(
                self.read_leb_u32()?,
                self.read_leb_u32()?,
                self.read_byte()?,
            )), // v128.store64_lane  m:memarg l:laneidx:byte
            12 => {
                let num = self.read_bytes(16)?;
                let val = i128::from_le_bytes(num.try_into().unwrap());
                Ok(FD::V128Const(val))
            } // v128.const b:bytes(16):i128
            13 => Ok(FD::I8x16Shuffle(self.read_bytes(16)?)), // i8x16.shuffle l:laneidx:byte
            21 => Ok(FD::I8x16ExtractLaneS(self.read_byte()?)), // i8x16.extract_lane_s l:laneidx
            22 => Ok(FD::I8x16ExtractLaneU(self.read_byte()?)), // i8x16.extract_lane_u l:laneidx
            23 => Ok(FD::I8x16ReplaceLane(self.read_byte()?)), // i8x16.replace_lane   l:laneidx
            24 => Ok(FD::I16x8ExtractLaneS(self.read_byte()?)), // i16x8.extract_lane_s l:laneidx
            25 => Ok(FD::I16x8ExtractLaneU(self.read_byte()?)), // i16x8.extract_lane_u l:laneidx
            26 => Ok(FD::I16x8ReplaceLane(self.read_byte()?)), // i16x8.replace_lane   l:laneidx
            27 => Ok(FD::I32x4ExtractLane(self.read_byte()?)), // i32x4.extract_lane   l:laneidx
            28 => Ok(FD::I32x4ReplaceLane(self.read_byte()?)), // i32x4.replace_lane   l:laneidx
            29 => Ok(FD::I64x2ExtractLane(self.read_byte()?)), // i64x2.extract_lane   l:laneidx
            30 => Ok(FD::I64x2ReplaceLane(self.read_byte()?)), // i64x2.replace_lane   l:laneidx
            31 => Ok(FD::F32x4ExtractLane(self.read_byte()?)), // f32x4.extract_lane   l:laneidx
            32 => Ok(FD::F32x4ReplaceLane(self.read_byte()?)), // f32x4.replace_lane   l:laneidx
            33 => Ok(FD::F64x2ExtractLane(self.read_byte()?)), // f64x2.extract_lane   l:laneidx
            34 => Ok(FD::F64x2ReplaceLane(self.read_byte()?)), // f64x2.replace_lane   l:laneidx
            14 => Ok(FD::I8x16Swizzle),                       // i8x16.swizzle
            15 => Ok(FD::I8x16Splat),                         // i8x16.splat
            16 => Ok(FD::I16x8Splat),                         // i16x8.splat
            17 => Ok(FD::I32x4Splat),                         // i32x4.splat
            18 => Ok(FD::I64x2Splat),                         // i64x2.splat
            19 => Ok(FD::F32x4Splat),                         // f32x4.splat
            20 => Ok(FD::F64x2Splat),                         // f64x2.splat
            35 => Ok(FD::I8x16Eq),                            // i8x16.eq
            36 => Ok(FD::I8x16Ne),                            // i8x16.ne
            37 => Ok(FD::I8x16Lts),                           // i8x16.lt_s
            38 => Ok(FD::I8x16Ltu),                           // i8x16.lt_u
            39 => Ok(FD::I8x16Gts),                           // i8x16.gt_s
            40 => Ok(FD::I8x16Gtu),                           // i8x16.gt_u
            41 => Ok(FD::I8x16Les),                           // i8x16.le_s
            42 => Ok(FD::I8x16Leu),                           // i8x16.le_u
            43 => Ok(FD::I8x16Ges),                           // i8x16.ge_s
            44 => Ok(FD::I8x16Geu),                           // i8x16.ge_u
            45 => Ok(FD::I16x8Eq),                            // i16x8.eq
            46 => Ok(FD::I16x8Ne),                            // i16x8.ne
            47 => Ok(FD::I16x8Lts),                           // i16x8.lt_s
            48 => Ok(FD::I16x8Ltu),                           // i16x8.lt_u
            49 => Ok(FD::I16x8Gts),                           // i16x8.gt_s
            50 => Ok(FD::I16x8Gtu),                           // i16x8.gt_u
            51 => Ok(FD::I16x8Les),                           // i16x8.le_s
            52 => Ok(FD::I16x8Leu),                           // i16x8.le_u
            53 => Ok(FD::I16x8Ges),                           // i16x8.ge_s
            54 => Ok(FD::I16x8Geu),                           // i16x8.ge_u
            55 => Ok(FD::I32x4Eq),                            // i32x4.eq
            56 => Ok(FD::I32x4Ne),                            // i32x4.ne
            57 => Ok(FD::I32x4Lts),                           // i32x4.lt_s
            58 => Ok(FD::I32x4Ltu),                           // i32x4.lt_u
            59 => Ok(FD::I32x4Gts),                           // i32x4.gt_s
            60 => Ok(FD::I32x4Gtu),                           // i32x4.gt_u
            61 => Ok(FD::I32x4Les),                           // i32x4.le_s
            62 => Ok(FD::I32x4Leu),                           // i32x4.le_u
            63 => Ok(FD::I32x4Ges),                           // i32x4.ge_s
            64 => Ok(FD::I32x4Geu),                           // i32x4.ge_u
            214 => Ok(FD::I64x2Eq),                           // i64x2.eq
            215 => Ok(FD::I64x2Ne),                           // i64x2.ne
            216 => Ok(FD::I64x2Lts),                          // i64x2.lt_s
            217 => Ok(FD::I64x2Gts),                          // i64x2.gt_s
            218 => Ok(FD::I64x2Les),                          // i64x2.le_s
            219 => Ok(FD::I64x2Ges),                          // i64x2.ge_s
            65 => Ok(FD::F32x4Eq),                            // f64x2.eq
            66 => Ok(FD::F32x4Ne),                            // f64x2.ne
            67 => Ok(FD::F32x4Lts),                           // f64x2.lt_s
            68 => Ok(FD::F32x4Gts),                           // f64x2.gt_s
            69 => Ok(FD::F32x4Les),                           // f64x2.le_s
            70 => Ok(FD::F32x4Ges),                           // f64x2.ge_s
            71 => Ok(FD::F64x2Eq),                            // f64x2.eq
            72 => Ok(FD::F64x2Ne),                            // f64x2.ne
            73 => Ok(FD::F64x2Lts),                           // f64x2.lt_s
            74 => Ok(FD::F64x2Gts),                           // f64x2.gt_s
            75 => Ok(FD::F64x2Les),                           // f64x2.le_s
            76 => Ok(FD::F64x2Ges),                           // f64x2.ge_s
            77 => Ok(FD::V128Not),                            // v128.not
            78 => Ok(FD::V128And),                            // v128.and
            79 => Ok(FD::V128AndNot),                         // v128.and_not
            80 => Ok(FD::V128Or),                             // v128.or
            81 => Ok(FD::V128Xor),                            // v128.xor
            82 => Ok(FD::V128BitSelect),                      // v128.bit_select
            83 => Ok(FD::V128AnyTrue),                        // v128.any_true
            96 => Ok(FD::I8x16Abs),                           // i8x16.abs
            97 => Ok(FD::I8x16Neg),                           // i8x16.neg
            98 => Ok(FD::I8x16Popcnt),                        // i8x16.popcnt
            99 => Ok(FD::I8x16AllTrue),                       // i8x16.all_true
            100 => Ok(FD::I8x16BitMask),                      // i8x16.bit_mask
            101 => Ok(FD::I8x16Narrow16x8s),                  // i8x16.narrow_16x8_s
            102 => Ok(FD::I8x16Narrow16x8u),                  // i8x16.narrow_16x8_u
            107 => Ok(FD::I8x16Shl),                          // i8x16.shl
            108 => Ok(FD::I8x16Shrs),                         // i8x16.shr_s
            109 => Ok(FD::I8x16Shru),                         // i8x16.shr_u
            110 => Ok(FD::I8x16Add),                          // i8x16.add
            111 => Ok(FD::I8x16AddSats),                      // i8x16.add_sats
            112 => Ok(FD::I8x16AddSatu),                      // i8x16.add_satu
            113 => Ok(FD::I8x16Sub),                          // i8x16.sub
            114 => Ok(FD::I8x16SubStas),                      // i8x16.sub_stas
            115 => Ok(FD::I8x16SubStau),                      // i8x16.sub_stau
            118 => Ok(FD::I8x16Mins),                         // i8x16.min_s
            119 => Ok(FD::I8x16Minu),                         // i8x16.min_u
            120 => Ok(FD::I8x16Maxs),                         // i8x16.max_s
            121 => Ok(FD::I8x16Maxu),                         // i8x16.max_u
            123 => Ok(FD::I8x16Avgru),                        // i8x16.avgr_u
            124 => Ok(FD::I16x8ExtaddPariwiseI8x16s),         // i16x8.extadd_pariwise.i8x16_s,
            125 => Ok(FD::I16x8ExtaddPariwiseI8x16u),         // i16x8.extadd_pariwise.i8x16_u,
            128 => Ok(FD::I16x8Abs),                          // i16x8.abs,
            129 => Ok(FD::I16x8Neg),                          // i16x8.neg,
            130 => Ok(FD::I16x8Q15MulrSats),                  // i16x8.q15mulr_sat_s,
            131 => Ok(FD::I16x8AllTrue),                      // i16x8.all_true,
            132 => Ok(FD::I16x8BitMask),                      // i16x8.bit_task,
            133 => Ok(FD::I16x8NarrowI32x4s),                 // i16x8.narrow_i32x4_s,
            134 => Ok(FD::I16x8NarrowI32x4u),                 // i16x8.narrow_i32x4_u,
            135 => Ok(FD::I16x8ExtendLowI8x16s),              // i16x8.extend_low_i8x16_s,
            136 => Ok(FD::I16x8ExtendHighI8x16s),             // i16x8.extend_high_i8x16_s,
            137 => Ok(FD::I16x8ExtendLowI8x16u),              // i16x8.extend_low_i8x16_u,
            138 => Ok(FD::I16x8ExtendHighI8x16u),             // i16x8.extend_high_i8x16_u,
            139 => Ok(FD::I16x8Shl),                          // i16x8.shl,
            140 => Ok(FD::I16x8Shrs),                         // i16x8.shr_s,
            141 => Ok(FD::I16x8Shru),                         // i16x8.shr_u,
            142 => Ok(FD::I16x8Add),                          // i16x8.add,
            143 => Ok(FD::I16x8AddSats),                      // i16x8.add_sat_s,
            144 => Ok(FD::I16x8AddSatu),                      // i16x8.add_sat_u,
            145 => Ok(FD::I16x8Sub),                          // i16x8.sub,
            146 => Ok(FD::I16x8SubSats),                      // i16x8.sub_sat_s,
            147 => Ok(FD::I16x8SubSatu),                      // i16x8.sub_sat_u,
            149 => Ok(FD::I16x8Mul),                          // i16x8.mul,
            150 => Ok(FD::I16x8Mins),                         // i16x8.min_s,
            151 => Ok(FD::I16x8Minu),                         // i16x8.min_u,
            152 => Ok(FD::I16x8Maxs),                         // i16x8.max_s,
            153 => Ok(FD::I16x8Maxu),                         // i16x8.max_u,
            155 => Ok(FD::I16x8Avgru),                        // i16x8.avgr_u,
            156 => Ok(FD::I16x8ExtmulLowI8x16s),              // i16x8.extmul_low_i8x16_s,
            157 => Ok(FD::I16x8ExtmulHighI8x16s),             // i16x8.extmul_high_i8x16_s,
            158 => Ok(FD::I16x8ExtmulLowI8x16u),              // i16x8.extmul_low_i8x16_u,
            159 => Ok(FD::I16x8ExtmulHighI8x16u),             // i16x8.extmul_high_i8x16_u,
            126 => Ok(FD::I32x4ExtaddPariwiseI8x16s),         // i32x4.extadd_pariwise_i8x16_s
            127 => Ok(FD::I32x4ExtaddPariwiseI8x16u),         // i32x4.extadd_pariwise_i8x16_u
            160 => Ok(FD::I32x4Abs),                          // i32x4.abs
            161 => Ok(FD::I32x4Neg),                          // i32x4.neg
            163 => Ok(FD::I32x4AllTrue),                      // i32x4.all_true
            164 => Ok(FD::I32x4BitMask),                      // i32x4.bit_mask
            167 => Ok(FD::I32x4ExtendLowI8x16s),              // i32x4.extend_low_i8x16_s
            168 => Ok(FD::I32x4ExtendHighI8x16s),             // i32x4.extend_high_i8x16_s
            169 => Ok(FD::I32x4ExtendLowI8x16u),              // i32x4.extend_low_i8x16_u
            170 => Ok(FD::I32x4ExtendHighI8x16u),             // i32x4.extend_high_i8x16_u
            171 => Ok(FD::I32x4Shl),                          // i32x4.shl
            172 => Ok(FD::I32x4Shrs),                         // i32x4.shr_s
            173 => Ok(FD::I32x4Shru),                         // i32x4.shr_u
            174 => Ok(FD::I32x4Add),                          // i32x4.add
            177 => Ok(FD::I32x4Sub),                          // i32x4.sub
            181 => Ok(FD::I32x4Mul),                          // i32x4.mul
            182 => Ok(FD::I32x4Mins),                         // i32x4.min_s
            183 => Ok(FD::I32x4Minu),                         // i32x4.min_u
            184 => Ok(FD::I32x4Maxs),                         // i32x4.max_s
            185 => Ok(FD::I32x4Maxu),                         // i32x4.max_u
            186 => Ok(FD::I32x4DotI16x8),                     // i32x4.dot_i16x8
            188 => Ok(FD::I32x4ExtmulLowI8x16s),              // i32x4.extmul_low_i8x16_s
            189 => Ok(FD::I32x4ExtmulHighI8x16s),             // i32x4.extmul_high_i8x16_s
            190 => Ok(FD::I32x4ExtmulLowI8x16u),              // i32x4.extmul_low_i8x16_u
            191 => Ok(FD::I32x4ExtmulHighI8x16u),             // i32x4.extmul_high_i8x16_u
            192 => Ok(FD::I64x2Abs),                          // i64x2.abs
            193 => Ok(FD::I64x2Neg),                          // i64x2.neg
            195 => Ok(FD::I64x2AllTrue),                      // i64x2.all_true
            196 => Ok(FD::I64x2BitMask),                      // i64x2.bit_mask
            199 => Ok(FD::I64x2ExtendLowI32x4s),              // i64x2.extend_low_i32x4_s
            200 => Ok(FD::I64x2ExtendHighI32x4s),             // i64x2.extend_high_i32x4_s
            201 => Ok(FD::I64x2ExtendLowI32x4u),              // i64x2.extend_low_i32x4_u
            202 => Ok(FD::I64x2ExtendHighI32x4u),             // i64x2.extendHighI32x4_u
            203 => Ok(FD::I64x2Shl),                          // i64x2.shl
            204 => Ok(FD::I64x2Shrs),                         // i64x2.shr_s
            205 => Ok(FD::I64x2Shru),                         // i64x2.shr_u
            206 => Ok(FD::I64x2Add),                          // i64x2.add
            209 => Ok(FD::I64x2Sub),                          // i64x2.sub
            213 => Ok(FD::I64x2Mul),                          // i64x2.mul
            220 => Ok(FD::I64x2ExtmulLowI32x4s),              // i64x2.extmul_low_i32x4_s
            221 => Ok(FD::I64x2ExtmulHighI32x4s),             // i64x2.extmul_high_i32x4_s
            222 => Ok(FD::I64x2ExtmulLowI32x4u),              // i64x2.extmul_low_i32x4_u
            223 => Ok(FD::I64x2ExtmulHighI32x4u),             // i64x2.extmul_high_i32x4_u
            103 => Ok(FD::F32x4Ceil),                         // f32x4.ceil
            104 => Ok(FD::F32x4Floor),                        // f32x4.floor
            105 => Ok(FD::F32x4Trunc),                        // f32x4.trunc
            106 => Ok(FD::F32x4Nearest),                      // f32x4.nearest
            224 => Ok(FD::F32x4Abs),                          // f32x4.abs
            225 => Ok(FD::F32x4Neg),                          // f32x4.neg
            227 => Ok(FD::F32x4Sqrt),                         // f32x4.sqrt
            228 => Ok(FD::F32x4Add),                          // f32x4.add
            229 => Ok(FD::F32x4Sub),                          // f32x4.sub
            230 => Ok(FD::F32x4Mul),                          // f32x4.mul
            231 => Ok(FD::F32x4Div),                          // f32x4.div
            232 => Ok(FD::F32x4Min),                          // f32x4.min
            233 => Ok(FD::F32x4Max),                          // f32x4.max
            234 => Ok(FD::F32x4Pmin),                         // f32x4.pmin
            235 => Ok(FD::F32x4Pmax),                         // f32x4.pmax
            116 => Ok(FD::F64x2Ceil),                         // f64x2.ceil
            117 => Ok(FD::F64x2Floor),                        // f64x2.floor
            122 => Ok(FD::F64x2Trunc),                        // f64x2.trunc
            148 => Ok(FD::F64x2Nearest),                      // f64x2.nearest
            236 => Ok(FD::F64x2Abs),                          // f64x2.abs
            237 => Ok(FD::F64x2Neg),                          // f64x2.neg
            239 => Ok(FD::F64x2Sqrt),                         // f64x2.sqrt
            240 => Ok(FD::F64x2Add),                          // f64x2.add
            241 => Ok(FD::F64x2Sub),                          // f64x2.sub
            242 => Ok(FD::F64x2Mul),                          // f64x2.mul
            243 => Ok(FD::F64x2Div),                          // f64x2.div
            244 => Ok(FD::F64x2Min),                          // f64x2.min
            245 => Ok(FD::F64x2Max),                          // f64x2.max
            246 => Ok(FD::F64x2Pmin),                         // f64x2.pmin
            247 => Ok(FD::F64x2Pmax),                         // f64x2.pmax
            248 => Ok(FD::I32x4TruncSatF32x4s),               // i32x4.trunc_sat_f32x4_s
            249 => Ok(FD::I32x4TruncSatF32x4u),               // i32x4.trunc_sat_f32x4_u
            250 => Ok(FD::I32x4ConvertI32x4s),                // i32x4.convert_i32x4_s
            251 => Ok(FD::I32x4ConvertI32x4u),                // i32x4.convert_i32x4_u
            252 => Ok(FD::I32x4TruncSatF64x2sZero),           // i32x4.trunc_sat_f64x2_s_zero
            253 => Ok(FD::I32x4TruncSatF64x2uZero),           // i32x4.trunc_sat_f64x2_u_zero
            254 => Ok(FD::I32x4ConvertLowI32x4s),             // i32x4.convert_low_i32x4_s
            255 => Ok(FD::I32x4ConvertLowI32x4u),             // i32x4.convert_low_i32x4_u
            94 => Ok(FD::I32x4DemoteF64x2zero),               // i32x4.demote_f64x2_zero
            95 => Ok(FD::I32x4PremoteLowF32x4),               // i32x4.premote_low_f32x4
            v => Err(anyhow!("unkonwn fd sub op {v:x}")),
        }
    }
}

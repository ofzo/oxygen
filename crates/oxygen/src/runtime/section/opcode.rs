use super::typings::ValueType;

/// (start, end, len)
#[derive(Debug, Clone)]
pub struct Location(pub usize, pub usize, pub usize);

// https://webassembly.github.io/spec/core/binary/instructions.html
#[derive(Debug, Clone)]
pub enum Opcode {
    // Control code blocktype | t:valtype | x:s33
    // 对于结构化指令，形成嵌套块的指令序列以用于 end(0x0b) 和 else(0x05) 的显式操作码终止。
    Unreachable,                                         // unreachable
    Nop,                                                 // nop
    Block(BlockType, Location),                          // block <bt:blocktype> in*:instr end
    Loop(BlockType, Location),                           // loop <bt:blocktype> in*:instr end
    If(BlockType, Location), // if <bt:blocktype> in*:instr else in*:instr end
    Else(Location),          // else
    End(usize),              // end
    Br(usize, usize),        // br <l:lableidx>
    BrIf(usize, usize),      // br_if <l:lableidx>
    BrTable(usize, Vec<(usize, usize)>, (usize, usize)), // br_table <l*:vec(lableidx)> <lN:lableidx>
    Return,                                              // return
    Call(u32),                                           //call <x:funcidx>
    CallIndirect(u32, u32),                              //call_indirect <x:typeidx> <y:tableidx>

    // reference code
    RefNull(u8),  //ref.null t:reftype
    RefIsNull,    //ref.is_null
    RefFunc(u32), //ref.func x:funcidx

    // Parametric code
    Drop,                          //drop
    Select,                        //select
    SelectType(usize, Vec<usize>), //select t*:vec(valtype)

    // Variable code
    LocalGet(u32),  // local.get <x:localidx>
    LocalSet(u32),  // local.set <x:localidx>
    LocalTee(u32),  // local.tee <x:localidx>
    GlobalGet(u32), // global.get <x:globalidx>
    GlobalSet(u32), // global.set <x:globalidx>

    // table code
    TableGet(u32), // table.get x:tableidx
    TableSet(u32), // table.set x:tableidx

    // memory code  memarg a:u32 o:u32  {align a, offset o}
    I32Load(u32, u32),    // i32.load m:memarg
    I64Load(u32, u32),    // i64.load m:memarg
    F32Load(u32, u32),    // f32.load m:memarg
    F64Load(u32, u32),    // f64.load m:memarg
    I32Load8s(u32, u32),  // i32.load8_s m:memarg
    I32Load8u(u32, u32),  // i32.load 8_um:memarg
    I32Load16s(u32, u32), // i32.load16_s m:memarg
    I32Load16u(u32, u32), // i32.load16_u m:memarg
    I64Load8s(u32, u32),  // i64.load8_s m:memarg
    I64Load8u(u32, u32),  // i64.load8_u m:memarg
    I64Load16s(u32, u32), // i64.load16_s m:memarg
    I64Load16u(u32, u32), // i64.load16_u m:memarg
    I64Load32s(u32, u32), // i64.load32_s m:memarg
    I64Load32u(u32, u32), // i64.load32_u m:memarg
    I32Store(u32, u32),   // i32.store m:memarg
    I64Store(u32, u32),   // i64.store m:memarg
    F32Store(u32, u32),   // f32.store m:memarg
    F64Store(u32, u32),   // f64.store m:memarg
    I32Store8(u32, u32),  // i32.store8 m:memarg
    I32Store16(u32, u32), // i32.store16 m:memarg
    I64Store8(u32, u32),  // i64.store8 m:memarg
    I64Store16(u32, u32), // i64.store16 m:memarg
    I64Store32(u32, u32), // i64.store32 m:memarg
    MemorySize,           // memory.size
    MemoryGrow,           // memory.grow

    // numeric
    // https://webassembly.github.io/spec/core/binary/instructions.html#numeric-instructions
    I32Const(i32), // i32.const x:i32
    I64Const(i64), // i64.const x:i64
    F32Const(f32), // f32.const x:f32
    F64Const(f64), // f64.const x.f64

    I32Eqz, // i32.eqz
    I32Eq,  // i32.eq
    I32Ne,  // i32.ne
    I32Lts, // i32.lt_s
    I32Ltu, // i32.lt_u
    I32Gts, // i32.gt_s
    I32Gtu, // i32.gt_u
    I32Les, // i32.le_s
    I32Leu, // i32.le_u
    I32Ges, // i32.ge_s
    I32Geu, // i32.ge_u

    I64Eqz, // i64.eqz
    I64Eq,  // i64.eq
    I64Ne,  // i64.ne
    I64Lts, // i64.lt_s
    I64Ltu, // i64.lt_u
    I64Gts, // i64.gt_s
    I64Gtu, // i64.gt_u
    I64Les, // i64.le_s
    I64Leu, // i64.le_u
    I64Ges, // i64.ge_s
    I64Geu, // i64.ge_u

    F32Eq, // f32.eq
    F32Ne, // f32.ne
    F32Lt, // f32.lt
    F32Gt, // f32.gt
    F32Le, // f32.le
    F32Ge, // f32.ge

    F64Eq, // f64.eq
    F64Ne, // f64.ne
    F64Lt, // f64.lt
    F64Gt, // f64.gt
    F64Le, // f64.le
    F64Ge, // f64.ge

    I32Clz,    // i32.clz
    I32Ctz,    // i32.ctz
    I32Popcnt, // i32.popcnt
    I32Add,    // i32.add
    I32Sub,    // i32.sub
    I32Mul,    // i32.mul
    I32DivS,   // i32.div_s
    I32DivU,   // i32.div_u
    I32RemS,   // i32.rem_s
    I32RemU,   // i32.rem_u
    I32And,    // i32.and
    I32Or,     // i32.or
    I32Xor,    // i32.xor
    I32Shl,    // i32.shl
    I32ShlS,   // i32.shl_s
    I32ShlU,   // i32.shl_u
    I32Rotl,   // i32.rotl
    I32Rotr,   // i32.rotr

    I64Clz,    // i64.clz
    I64Ctz,    // i64.ctz
    I64Popcnt, // i64.popcnt
    I64Add,    // i64.add
    I64Sub,    // i64.sub
    I64Mul,    // i64.mul
    I64DivS,   // i64.div_s
    I64DivU,   // i64.div_u
    I64RemS,   // i64.rem_s
    I64RemU,   // i64.rem_u
    I64And,    // i64.and
    I64Or,     // i64.or
    I64Xor,    // i64.xor
    I64Shl,    // i64.shl
    I64ShlS,   // i64.shl_s
    I64ShlU,   // i64.shl_u
    I64Rotl,   // i64.rotl
    I64Rotr,   // i64.rotr

    F32Abs,      // f32.abs
    F32Neg,      // f32.neg
    F32Ceil,     // f32.ceil
    F32Floor,    // f32.floor
    F32Trunc,    // f32.trunc
    F32Nearest,  // f32.nearest
    F32Sqrt,     // f32.sqrt
    F32Add,      // f32.add
    F32Sub,      // f32.sub
    F32Mul,      // f32.mul
    F32Div,      // f32.div
    F32Min,      // f32.min
    F32Max,      // f32.max
    F32Copysign, // f32.copysign

    F64Abs,      // f64.abs
    F64Neg,      // f64.neg
    F64Ceil,     // f64.ceil
    F64Floor,    // f64.floor
    F64Trunc,    // f64.trunc
    F64Nearest,  // f64.nearest
    F64Sqrt,     // f64.sqrt
    F64Add,      // f64.add
    F64Sub,      // f64.sub
    F64Mul,      // f64.mul
    F64Div,      // f64.div
    F64Min,      // f64.min
    F64Max,      // f64.max
    F64Copysign, // f64.copysign

    I32WrapI64,        // i32.wrap_i64
    I32TruncF32s,      // i32.trunc_f32_s
    I32TruncF32u,      // i32.trunc_f32_u
    I32TruncF64s,      // i32.trunc_f64_s
    I32TruncF64u,      // i32.trunc_f64_u
    I64ExtendsI32s,    // i64.extends_i32_s
    I64ExtendsI32u,    // i64.extends_i32_u
    I64TruncF32s,      // i64.trunc_f32_s
    I64TruncF32u,      // i64.trunc_f32_u
    I64TruncF64s,      // i64.trunc_f64_s
    I64TruncF64u,      // i64.trunc_f64_u
    F32ConvertI32s,    // f32.convert_i32_s
    F32ConvertI32u,    // f32.convert_i32_u
    F32ConvertI64s,    // f32.convert_i64_s
    F32ConvertI64u,    // f32.convert_i64_u
    F32DemoteF64,      // f32.demote_f64
    F64ConvertI32s,    // f64.convert_i32_s
    F64ConvertI32u,    // f64.convert_i32_u
    F64ConvertI64s,    // f64.convert_i64_s
    F64ConvertI64u,    // f64.convert_i64_u
    F64DemoteF32,      // f64.demote_f32
    I32ReinterpretF32, // i32.reinterpret_f32
    I64ReinterpretF64, // i64.reinterpret_f64
    F32ReinterpretI32, // f32.reinterpret_i32
    F64ReinterpretI64, // f64.reinterpret_i64

    I32Extends8s,  // i32.extends8_s
    I32Extends16s, // i32.extends16_s
    I64Extends8s,  // i64.extends8_s
    I64Extends16s, // i64.extends16_s
    I64Extends32s, // i64.extends32_s

    // vector
    FD(FD), // fd
    // op
    // OP,
    I32TruncSatF32s, // op 0:u32                     => i32.trunc_sat_f32_s
    I32TruncSatF32u, // op 1:u32                     => i32.trunc_sat_f32_u
    I32TruncSatF64s, // op 2:u32                     => i32.trunc_sat_f64_s
    I32TruncSatF64u, // op 3:u32                     => i32.trunc_sat_f64_u
    I64TruncSatF32s, // op 4:u32                     => i64.trunc_sat_f32_s
    I64TruncSatF32u, // op 5:u32                     => i64.trunc_sat_f32_u
    I64TruncSatF64s, // op 6:u32                     => i64.trunc_sat_f64_s
    I64TruncSatF64u, // op 7:u32                     => i64.trunc_sat_f64_u
    // -- memory
    MemoryInit(usize), // op 8:u32 x:dataidx 0x00           =>  memory.init x
    DataDrop(usize),   // op 9:u32 x:dataidx                =>  data.drop x
    MemoryCopy,        // op 10:u32 0x00 0x00               =>  memory.copy
    MemoryFill,        // op 11:u32 0x00                    =>  memory.fill

    // -- table
    TableInit(usize, usize), // op 12:u32 y:elemidx x:tableidx    =>  table.init x y
    ElemDrop(usize),         // op 13:u32 x:elemidx               =>  elem.drop x
    TableCopy(usize, usize), // op 14:u32 x:tableidx y:tableidx   =>  table.copy x y
    TableGrow(usize),        // op 15:u32 x:tableidx              =>  table.grow x
    TableSize(usize),        // op 16:u32 x:tableidx              =>  table.size x
    TableFill(usize),        // op 17:u32 x:tableidx              =>  table.fill x

    Reserved(u8), // reserved
}

#[derive(Debug)]
enum OP {
    // op <u32>
    // -- numeric
}

#[derive(Debug, Clone)]
// https://webassembly.github.io/spec/core/binary/instructions.html#vector-instructions
pub enum FD {
    // prefix 0xfd
    V128Load(u32, u32),            // v128.load  m:memarg
    V128Load8x8s(u32, u32),        // v128.load8x8_s m:memarg
    V128Load8x8u(u32, u32),        // v128.load8x8_u m:memarg
    V128Load16x4s(u32, u32),       // v128.load16x4_s m:memarg
    V128Load16x4u(u32, u32),       // v128.load16x4_u m:memarg
    V128Load32x2s(u32, u32),       // v128.load32x2_s m:memarg
    V128Load32x2u(u32, u32),       // v128.load32x2_u m:memarg
    V128Load8splat(u32, u32),      // v128.load8_splat m:memarg
    V128Load16splat(u32, u32),     // v128.load16_splat m:memarg
    V128Load32splat(u32, u32),     // v128.load32_splat m:memarg
    V128Load64splat(u32, u32),     // v128.load64_splat m:memarg
    V128Load32zero(u32, u32),      // v128.load32_zero m:memarg
    V128Load64zero(u32, u32),      // v128.load64_zero m:memarg
    V128Store(u32, u32),           // v128.store m:memarg
    V128Load8lane(u32, u32, u8),   // v128.load8_lane m:memarg l:laneidx:byte
    V128Load16lane(u32, u32, u8),  // v128.load16_lane  m:memarg l:laneidx:byte
    V128Load32lane(u32, u32, u8),  // v128.load32_lane  m:memarg l:laneidx:byte
    V128Load64lane(u32, u32, u8),  // v128.load64_lane  m:memarg l:laneidx:byte
    V128Store8lane(u32, u32, u8),  // v128.store8_lane  m:memarg l:laneidx:byte
    V128Store16lane(u32, u32, u8), // v128.store16_lane  m:memarg l:laneidx:byte
    V128Store32lane(u32, u32, u8), // v128.store32_lane  m:memarg l:laneidx:byte
    V128Store64lane(u32, u32, u8), // v128.store64_lane  m:memarg l:laneidx:byte

    V128Const(i128),       // v128.const b:bytes(16):i128
    I8x16Shuffle(Vec<u8>), // i8x16.shuffle (l:laneidx:byte)*16

    I8x16ExtractLaneS(u8), // i8x16.extract_lane_s l:laneidx
    I8x16ExtractLaneU(u8), // i8x16.extract_lane_u l:laneidx
    I8x16ReplaceLane(u8),  // i8x16.replace_lane   l:laneidx
    I16x8ExtractLaneS(u8), // i16x8.extract_lane_s l:laneidx
    I16x8ExtractLaneU(u8), // i16x8.extract_lane_u l:laneidx
    I16x8ReplaceLane(u8),  // i16x8.replace_lane   l:laneidx
    I32x4ExtractLane(u8),  // i32x4.extract_lane   l:laneidx
    I32x4ReplaceLane(u8),  // i32x4.replace_lane   l:laneidx
    I64x2ExtractLane(u8),  // i64x2.extract_lane   l:laneidx
    I64x2ReplaceLane(u8),  // i64x2.replace_lane   l:laneidx
    F32x4ExtractLane(u8),  // f32x4.extract_lane   l:laneidx
    F32x4ReplaceLane(u8),  // f32x4.replace_lane   l:laneidx
    F64x2ExtractLane(u8),  // f64x2.extract_lane   l:laneidx
    F64x2ReplaceLane(u8),  // f64x2.replace_lane   l:laneidx

    I8x16Swizzle, // i8x16.swizzle
    I8x16Splat,   // i8x16.splat
    I16x8Splat,   // i16x8.splat
    I32x4Splat,   // i32x4.splat
    I64x2Splat,   // i64x2.splat
    F32x4Splat,   // f32x4.splat
    F64x2Splat,   // f64x2.splat

    I8x16Eq,  // i8x16.eq
    I8x16Ne,  // i8x16.ne
    I8x16Lts, // i8x16.lt_s
    I8x16Ltu, // i8x16.lt_u
    I8x16Gts, // i8x16.gt_s
    I8x16Gtu, // i8x16.gt_u
    I8x16Les, // i8x16.le_s
    I8x16Leu, // i8x16.le_u
    I8x16Ges, // i8x16.ge_s
    I8x16Geu, // i8x16.ge_u

    I16x8Eq,  // i16x8.eq
    I16x8Ne,  // i16x8.ne
    I16x8Lts, // i16x8.lt_s
    I16x8Ltu, // i16x8.lt_u
    I16x8Gts, // i16x8.gt_s
    I16x8Gtu, // i16x8.gt_u
    I16x8Les, // i16x8.le_s
    I16x8Leu, // i16x8.le_u
    I16x8Ges, // i16x8.ge_s
    I16x8Geu, // i16x8.ge_u

    I32x4Eq,  // i32x4.eq
    I32x4Ne,  // i32x4.ne
    I32x4Lts, // i32x4.lt_s
    I32x4Ltu, // i32x4.lt_u
    I32x4Gts, // i32x4.gt_s
    I32x4Gtu, // i32x4.gt_u
    I32x4Les, // i32x4.le_s
    I32x4Leu, // i32x4.le_u
    I32x4Ges, // i32x4.ge_s
    I32x4Geu, // i32x4.ge_u

    I64x2Eq,  // i64x2.eq
    I64x2Ne,  // i64x2.ne
    I64x2Lts, // i64x2.lt_s
    I64x2Gts, // i64x2.gt_s
    I64x2Les, // i64x2.le_s
    I64x2Ges, // i64x2.ge_s

    F32x4Eq,  // f64x2.eq
    F32x4Ne,  // f64x2.ne
    F32x4Lts, // f64x2.lt_s
    F32x4Gts, // f64x2.gt_s
    F32x4Les, // f64x2.le_s
    F32x4Ges, // f64x2.ge_s

    F64x2Eq,  // f64x2.eq
    F64x2Ne,  // f64x2.ne
    F64x2Lts, // f64x2.lt_s
    F64x2Gts, // f64x2.gt_s
    F64x2Les, // f64x2.le_s
    F64x2Ges, // f64x2.ge_s

    V128Not,       // v128.not
    V128And,       // v128.and
    V128AndNot,    // v128.and_not
    V128Or,        // v128.or
    V128Xor,       // v128.xor
    V128BitSelect, // v128.bit_select
    V128AnyTrue,   // v128.any_true

    I8x16Abs,         // i8x16.abs
    I8x16Neg,         // i8x16.neg
    I8x16Popcnt,      // i8x16.popcnt
    I8x16AllTrue,     // i8x16.all_true
    I8x16BitMask,     // i8x16.bit_mask
    I8x16Narrow16x8s, // i8x16.narrow_16x8_s
    I8x16Narrow16x8u, // i8x16.narrow_16x8_u
    I8x16Shl,         // i8x16.shl
    I8x16Shrs,        // i8x16.shr_s
    I8x16Shru,        // i8x16.shr_u
    I8x16Add,         // i8x16.add
    I8x16AddSats,     // i8x16.add_sats
    I8x16AddSatu,     // i8x16.add_satu
    I8x16Sub,         // i8x16.sub
    I8x16SubStas,     // i8x16.sub_stas
    I8x16SubStau,     // i8x16.sub_stau
    I8x16Mins,        // i8x16.min_s
    I8x16Minu,        // i8x16.min_u
    I8x16Maxs,        // i8x16.max_s
    I8x16Maxu,        // i8x16.max_u
    I8x16Avgru,       // i8x16.avgr_u

    I16x8ExtaddPariwiseI8x16s, // i16x8.extadd_pariwise.i8x16_s,
    I16x8ExtaddPariwiseI8x16u, // i16x8.extadd_pariwise.i8x16_u,
    I16x8Abs,                  // i16x8.abs,
    I16x8Neg,                  // i16x8.neg,
    I16x8Q15MulrSats,          // i16x8.q15mulr_sat_s,
    I16x8AllTrue,              // i16x8.all_true,
    I16x8BitMask,              // i16x8.bit_task,
    I16x8NarrowI32x4s,         // i16x8.narrow_i32x4_s,
    I16x8NarrowI32x4u,         // i16x8.narrow_i32x4_u,
    I16x8ExtendLowI8x16s,      // i16x8.extend_low_i8x16_s,
    I16x8ExtendHighI8x16s,     // i16x8.extend_high_i8x16_s,
    I16x8ExtendLowI8x16u,      // i16x8.extend_low_i8x16_u,
    I16x8ExtendHighI8x16u,     // i16x8.extend_high_i8x16_u,
    I16x8Shl,                  // i16x8.shl,
    I16x8Shrs,                 // i16x8.shr_s,
    I16x8Shru,                 // i16x8.shr_u,
    I16x8Add,                  // i16x8.add,
    I16x8AddSats,              // i16x8.add_sat_s,
    I16x8AddSatu,              // i16x8.add_sat_u,
    I16x8Sub,                  // i16x8.sub,
    I16x8SubSats,              // i16x8.sub_sat_s,
    I16x8SubSatu,              // i16x8.sub_sat_u,
    I16x8Mul,                  // i16x8.mul,
    I16x8Mins,                 // i16x8.min_s,
    I16x8Minu,                 // i16x8.min_u,
    I16x8Maxs,                 // i16x8.max_s,
    I16x8Maxu,                 // i16x8.max_u,
    I16x8Avgru,                // i16x8.avgr_u,
    I16x8ExtmulLowI8x16s,      // i16x8.extmul_low_i8x16_s,
    I16x8ExtmulHighI8x16s,     // i16x8.extmul_high_i8x16_s,
    I16x8ExtmulLowI8x16u,      // i16x8.extmul_low_i8x16_u,
    I16x8ExtmulHighI8x16u,     // i16x8.extmul_high_i8x16_u,
    //
    I32x4ExtaddPariwiseI8x16s, // i32x4.extadd_pariwise_i8x16_s
    I32x4ExtaddPariwiseI8x16u, // i32x4.extadd_pariwise_i8x16_u
    I32x4Abs,                  // i32x4.abs
    I32x4Neg,                  // i32x4.neg
    I32x4AllTrue,              // i32x4.all_true
    I32x4BitMask,              // i32x4.bit_mask
    I32x4ExtendLowI8x16s,      // i32x4.extend_low_i8x16_s
    I32x4ExtendHighI8x16s,     // i32x4.extend_high_i8x16_s
    I32x4ExtendLowI8x16u,      // i32x4.extend_low_i8x16_u
    I32x4ExtendHighI8x16u,     // i32x4.extend_high_i8x16_u
    I32x4Shl,                  // i32x4.shl
    I32x4Shrs,                 // i32x4.shr_s
    I32x4Shru,                 // i32x4.shr_u
    I32x4Add,                  // i32x4.add
    I32x4Sub,                  // i32x4.sub
    I32x4Mul,                  // i32x4.mul
    I32x4Mins,                 // i32x4.min_s
    I32x4Minu,                 // i32x4.min_u
    I32x4Maxs,                 // i32x4.max_s
    I32x4Maxu,                 // i32x4.max_u
    I32x4DotI16x8,             // i32x4.dot_i16x8
    I32x4ExtmulLowI8x16s,      // i32x4.extmul_low_i8x16_s
    I32x4ExtmulHighI8x16s,     // i32x4.extmul_high_i8x16_s
    I32x4ExtmulLowI8x16u,      // i32x4.extmul_low_i8x16_u
    I32x4ExtmulHighI8x16u,     // i32x4.extmul_high_i8x16_u
    //
    I64x2Abs,              // i64x2.abs
    I64x2Neg,              // i64x2.neg
    I64x2AllTrue,          // i64x2.all_true
    I64x2BitMask,          // i64x2.bit_mask
    I64x2ExtendLowI32x4s,  // i64x2.extend_low_i32x4_s
    I64x2ExtendHighI32x4s, // i64x2.extend_high_i32x4_s
    I64x2ExtendLowI32x4u,  // i64x2.extend_low_i32x4_u
    I64x2ExtendHighI32x4u, // i64x2.extendHighI32x4_u
    I64x2Shl,              // i64x2.shl
    I64x2Shrs,             // i64x2.shr_s
    I64x2Shru,             // i64x2.shr_u
    I64x2Add,              // i64x2.add
    I64x2Sub,              // i64x2.sub
    I64x2Mul,              // i64x2.mul
    I64x2ExtmulLowI32x4s,  // i64x2.extmul_low_i32x4_s
    I64x2ExtmulHighI32x4s, // i64x2.extmul_high_i32x4_s
    I64x2ExtmulLowI32x4u,  // i64x2.extmul_low_i32x4_u
    I64x2ExtmulHighI32x4u, // i64x2.extmul_high_i32x4_u
    //
    F32x4Ceil,    // f32x4.ceil
    F32x4Floor,   // f32x4.floor
    F32x4Trunc,   // f32x4.trunc
    F32x4Nearest, // f32x4.nearest
    F32x4Abs,     // f32x4.abs
    F32x4Neg,     // f32x4.neg
    F32x4Sqrt,    // f32x4.sqrt
    F32x4Add,     // f32x4.add
    F32x4Sub,     // f32x4.sub
    F32x4Mul,     // f32x4.mul
    F32x4Div,     // f32x4.div
    F32x4Min,     // f32x4.min
    F32x4Max,     // f32x4.max
    F32x4Pmin,    // f32x4.pmin
    F32x4Pmax,    // f32x4.pmax
    //
    F64x2Ceil,    // f64x2.ceil
    F64x2Floor,   // f64x2.floor
    F64x2Trunc,   // f64x2.trunc
    F64x2Nearest, // f64x2.nearest
    F64x2Abs,     // f64x2.abs
    F64x2Neg,     // f64x2.neg
    F64x2Sqrt,    // f64x2.sqrt
    F64x2Add,     // f64x2.add
    F64x2Sub,     // f64x2.sub
    F64x2Mul,     // f64x2.mul
    F64x2Div,     // f64x2.div
    F64x2Min,     // f64x2.min
    F64x2Max,     // f64x2.max
    F64x2Pmin,    // f64x2.pmin
    F64x2Pmax,    // f64x2.pmax
    //
    I32x4TruncSatF32x4s,     // i32x4.trunc_sat_f32x4_s
    I32x4TruncSatF32x4u,     // i32x4.trunc_sat_f32x4_u
    I32x4ConvertI32x4s,      // i32x4.convert_i32x4_s
    I32x4ConvertI32x4u,      // i32x4.convert_i32x4_u
    I32x4TruncSatF64x2sZero, // i32x4.trunc_sat_f64x2_s_zero
    I32x4TruncSatF64x2uZero, // i32x4.trunc_sat_f64x2_u_zero
    I32x4ConvertLowI32x4s,   // i32x4.convert_low_i32x4_s
    I32x4ConvertLowI32x4u,   // i32x4.convert_low_i32x4_u
    I32x4DemoteF64x2zero,    // i32x4.demote_f64x2_zero
    I32x4PremoteLowF32x4,    // i32x4.premote_low_f32x4
}

#[derive(Debug, Clone)]
pub enum BlockType {
    NOP,
    ValueType(ValueType),
    Value(u32),
}
impl BlockType {
    pub fn from_u32(v: u32) -> Self {
        match v {
            0x40 => Self::NOP,
            v => match ValueType::from_u8(v as u8) {
                Ok(v) => Self::ValueType(v),
                Err(_) => Self::Value(v),
            },
        }
    }
}

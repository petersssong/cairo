#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use num_bigint::{BigInt, ToBigInt};

use crate::hints::Hint;
use crate::instructions::{Instruction, InstructionBody};
use crate::operand::{DerefOrImmediate, Operation, Register, ResOperand};

#[cfg(test)]
#[path = "assembler_test.rs"]
mod test;

/// Cairo instruction structure flags.
#[derive(Debug, Eq, PartialEq)]
pub enum Op1Addr {
    Imm,
    AP,
    FP,
    Op0,
}
#[derive(Debug, Eq, PartialEq)]
pub enum Res {
    Op1,
    Add,
    Mul,
    Unconstrained,
}
#[derive(Debug, Eq, PartialEq)]
pub enum PcUpdate {
    Regular,
    Jump,
    JumpRel,
    Jnz,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ApUpdate {
    Regular,
    Add,
    Add1,
    Add2,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FpUpdate {
    Regular,
    ApPlus2,
    Dst,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Opcode {
    Nop,
    AssertEq,
    Call,
    Ret,
}

#[derive(Debug, Eq, PartialEq)]
pub enum OpcodeExtension {
    Stone,
    Blake2s,
    Blake2sFinalize,
    QM31,
}

/// The low level representation of a cairo instruction.
#[derive(Debug, Eq, PartialEq)]
pub struct InstructionRepr {
    pub off0: i16,
    pub off1: i16,
    pub off2: i16,
    pub imm: Option<BigInt>,
    pub dst_register: Register,
    pub op0_register: Register,
    pub op1_addr: Op1Addr,
    pub res: Res,
    pub pc_update: PcUpdate,
    pub ap_update: ApUpdate,
    pub fp_update: FpUpdate,
    pub opcode: Opcode,
    pub opcode_extension: OpcodeExtension,
}

#[cfg(feature = "serde")]
use cairo_lang_utils::bigint::{deserialize_big_ints, serialize_big_ints};

/// An assembled representation of a cairo program.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone)]
pub struct AssembledCairoProgram {
    /// The bytecode of the program.
    #[cfg_attr(
        feature = "serde",
        serde(serialize_with = "serialize_big_ints", deserialize_with = "deserialize_big_ints")
    )]
    pub bytecode: Vec<BigInt>,
    /// The list of hints, and the instruction index they refer to.
    pub hints: Vec<(usize, Vec<Hint>)>,
}

impl Instruction {
    pub fn assemble(&self) -> InstructionRepr {
        match &self.body {
            InstructionBody::AddAp(insn) => {
                assert!(!self.inc_ap, "An add_ap instruction cannot have an ap++.");
                let res = insn.operand.to_res_description();
                InstructionRepr {
                    off0: -1,
                    off1: res.off1,
                    off2: res.off2,
                    imm: res.imm,
                    dst_register: Register::FP,
                    op0_register: res.op0_register,
                    op1_addr: res.op1_addr,
                    res: res.res,
                    pc_update: PcUpdate::Regular,
                    ap_update: ApUpdate::Add,
                    fp_update: FpUpdate::Regular,
                    opcode: Opcode::Nop,
                    opcode_extension: OpcodeExtension::Stone,
                }
            }
            InstructionBody::AssertEq(insn) => {
                let res = insn.b.to_res_description();
                InstructionRepr {
                    off0: insn.a.offset,
                    off1: res.off1,
                    off2: res.off2,
                    imm: res.imm,
                    dst_register: insn.a.register,
                    op0_register: res.op0_register,
                    op1_addr: res.op1_addr,
                    res: res.res,
                    pc_update: PcUpdate::Regular,
                    ap_update: if self.inc_ap { ApUpdate::Add1 } else { ApUpdate::Regular },
                    fp_update: FpUpdate::Regular,
                    opcode: Opcode::AssertEq,
                    opcode_extension: OpcodeExtension::Stone,
                }
            }
            InstructionBody::QM31AssertEq(insn) => {
                let res = insn.b.to_res_description();
                InstructionRepr {
                    off0: insn.a.offset,
                    off1: res.off1,
                    off2: res.off2,
                    imm: res.imm,
                    dst_register: insn.a.register,
                    op0_register: res.op0_register,
                    op1_addr: res.op1_addr,
                    res: res.res,
                    pc_update: PcUpdate::Regular,
                    ap_update: if self.inc_ap { ApUpdate::Add1 } else { ApUpdate::Regular },
                    fp_update: FpUpdate::Regular,
                    opcode: Opcode::AssertEq,
                    opcode_extension: OpcodeExtension::QM31,
                }
            }
            InstructionBody::Call(insn) => {
                assert!(!self.inc_ap, "A call instruction cannot have an ap++.");
                let res = insn.target.to_res_description();
                InstructionRepr {
                    off0: 0,
                    off1: 1,
                    off2: res.off2,
                    imm: res.imm,
                    dst_register: Register::AP,
                    op0_register: Register::AP,
                    op1_addr: res.op1_addr,
                    res: Res::Op1,
                    pc_update: if insn.relative { PcUpdate::JumpRel } else { PcUpdate::Jump },
                    ap_update: ApUpdate::Add2,
                    fp_update: FpUpdate::ApPlus2,
                    opcode: Opcode::Call,
                    opcode_extension: OpcodeExtension::Stone,
                }
            }
            InstructionBody::Jump(insn) => {
                let res = insn.target.to_res_description();
                InstructionRepr {
                    off0: -1,
                    off1: res.off1,
                    off2: res.off2,
                    imm: res.imm,
                    dst_register: Register::FP,
                    op0_register: Register::FP,
                    op1_addr: res.op1_addr,
                    res: Res::Op1,
                    pc_update: if insn.relative { PcUpdate::JumpRel } else { PcUpdate::Jump },
                    ap_update: if self.inc_ap { ApUpdate::Add1 } else { ApUpdate::Regular },
                    fp_update: FpUpdate::Regular,
                    opcode: Opcode::Nop,
                    opcode_extension: OpcodeExtension::Stone,
                }
            }
            InstructionBody::Jnz(insn) => {
                let res = insn.jump_offset.to_res_description();
                InstructionRepr {
                    off0: insn.condition.offset,
                    off1: -1,
                    off2: res.off2,
                    imm: res.imm,
                    dst_register: insn.condition.register,
                    op0_register: Register::FP,
                    op1_addr: res.op1_addr,
                    res: Res::Unconstrained,
                    pc_update: PcUpdate::Jnz,
                    ap_update: if self.inc_ap { ApUpdate::Add1 } else { ApUpdate::Regular },
                    fp_update: FpUpdate::Regular,
                    opcode: Opcode::Nop,
                    opcode_extension: OpcodeExtension::Stone,
                }
            }
            InstructionBody::Ret(_) => {
                assert!(!self.inc_ap);
                InstructionRepr {
                    off0: -2,
                    off1: -1,
                    off2: -1,
                    imm: None,
                    dst_register: Register::FP,
                    op0_register: Register::FP,
                    op1_addr: Op1Addr::FP,
                    res: Res::Op1,
                    pc_update: PcUpdate::Jump,
                    ap_update: ApUpdate::Regular,
                    fp_update: FpUpdate::Dst,
                    opcode: Opcode::Ret,
                    opcode_extension: OpcodeExtension::Stone,
                }
            }
            InstructionBody::Blake2sCompress(insn) => {
                assert!(self.inc_ap);
                InstructionRepr {
                    off0: insn.byte_count.offset,
                    off1: insn.state.offset,
                    off2: insn.message.offset,
                    imm: None,
                    dst_register: insn.byte_count.register,
                    op0_register: insn.state.register,
                    op1_addr: insn.message.register.to_op1_addr(),
                    res: Res::Op1,
                    pc_update: PcUpdate::Regular,
                    ap_update: ApUpdate::Add1,
                    fp_update: FpUpdate::Regular,
                    opcode: Opcode::Nop,
                    opcode_extension: if insn.finalize {
                        OpcodeExtension::Blake2sFinalize
                    } else {
                        OpcodeExtension::Blake2s
                    },
                }
            }
        }
    }
}

impl Register {
    fn to_op1_addr(self) -> Op1Addr {
        match self {
            Register::AP => Op1Addr::AP,
            Register::FP => Op1Addr::FP,
        }
    }
}

impl Operation {
    fn to_res(&self) -> Res {
        match self {
            Operation::Add => Res::Add,
            Operation::Mul => Res::Mul,
        }
    }
}

impl DerefOrImmediate {
    fn to_res_operand(&self) -> ResOperand {
        match self {
            DerefOrImmediate::Deref(operand) => ResOperand::Deref(*operand),
            DerefOrImmediate::Immediate(operand) => ResOperand::Immediate(operand.clone()),
        }
    }
    fn to_res_description(&self) -> ResDescription {
        self.to_res_operand().to_res_description()
    }
}

/// The part of the instruction describing the res operand.
struct ResDescription {
    off1: i16,
    off2: i16,
    imm: Option<BigInt>,
    op0_register: Register,
    op1_addr: Op1Addr,
    res: Res,
}

impl ResOperand {
    fn to_res_description(&self) -> ResDescription {
        match self {
            ResOperand::Deref(operand) => ResDescription {
                off1: -1,
                off2: operand.offset,
                imm: None,
                op0_register: Register::FP,
                op1_addr: operand.register.to_op1_addr(),
                res: Res::Op1,
            },
            ResOperand::DoubleDeref(operand, offset) => ResDescription {
                off1: operand.offset,
                off2: *offset,
                imm: None,
                op0_register: operand.register,
                op1_addr: Op1Addr::Op0,
                res: Res::Op1,
            },
            ResOperand::Immediate(operand) => ResDescription {
                off1: -1,
                off2: 1,
                // TODO(alon): Change immediate to always work with bigint.
                imm: operand.value.to_bigint(),
                op0_register: Register::FP,
                op1_addr: Op1Addr::Imm,
                res: Res::Op1,
            },
            ResOperand::BinOp(operand) => {
                let a_res = ResOperand::Deref(operand.a).to_res_description();
                let b_res = operand.b.to_res_description();
                ResDescription {
                    off1: a_res.off2,
                    off2: b_res.off2,
                    imm: b_res.imm,
                    op0_register: operand.a.register,
                    op1_addr: match operand.b {
                        DerefOrImmediate::Immediate(_) => Op1Addr::Imm,
                        DerefOrImmediate::Deref(b) => b.register.to_op1_addr(),
                    },
                    res: operand.op.to_res(),
                }
            }
        }
    }
}

use crate::asm::parser::AsmInst;
use crate::asm::parser::BitInv;
use crate::asm::parser::BitMode;
use crate::asm::parser::Branch;
use crate::asm::parser::BranchMode;
use crate::asm::parser::DirectBit;
use crate::asm::parser::DirectMem;
use crate::asm::parser::FullAcc;
use crate::asm::parser::HalfAcc;
use crate::asm::parser::Imm8;
use crate::asm::parser::ImmMem;
use crate::asm::parser::ImmMemBytes;
use crate::asm::parser::IndexBytes;
use crate::asm::parser::Indexed;
use crate::asm::parser::Inherent;
use crate::asm::parser::LogicalMem;
use crate::asm::parser::MemAddrMode;
use crate::asm::parser::ShiftAcc;
use crate::asm::parser::StackReg;
use crate::asm::parser::StoreLoad;
use crate::asm::parser::TfmMode;

use byteorder::{ByteOrder, BigEndian};

pub fn generate_object_code(input: Vec<AsmInst>) -> Vec<u8> {
    let mut result = Vec::new();
    for inst in input {
        result.extend(generate_opcode(inst))
    }
    result
}

fn generate_opcode(inst: AsmInst) -> Vec<u8> {
    match inst {
        AsmInst::Inherent(i)               => generate_inherent(i),
        AsmInst::Imm8(i, post)             => generate_imm8(i, post),
        AsmInst::DirectBit(i, post, addr)  => generate_direct_bit(i, post, addr),
        AsmInst::Indexed(i, post, addr)    => generate_indexed(i, post, addr),
        AsmInst::DirectMem(i, addr)        => generate_direct_mem(i, addr),
        AsmInst::LogicalMem(i, imm8, addr) => generate_logical_mem(i, imm8, addr),
        AsmInst::Branch(i, bm)             => generate_branch(i, bm),
        AsmInst::ImmMem(i, imm_addr)       => generate_imm_mem(i, imm_addr),
    }
}

fn generate_inherent(inst: Inherent) -> Vec<u8> {
    match inst {
        Inherent::ABX   => vec![0x3A],
        Inherent::DAA   => vec![0x19],
        Inherent::MUL   => vec![0x3D],
        Inherent::NOP   => vec![0x12],
        Inherent::RTI   => vec![0x3B],
        Inherent::RTS   => vec![0x39],
        Inherent::SYNC  => vec![0x13],
        Inherent::PSHSW => vec![0x10, 0x38],
        Inherent::PSHUW => vec![0x10, 0x3A],
        Inherent::PULSW => vec![0x10, 0x39],
        Inherent::PULUW => vec![0x10, 0x3B],
        Inherent::SEX   => vec![0x1D],
        Inherent::SEXW  => vec![0x14],
        Inherent::SWI   => vec![0x3F],
        Inherent::SWI2  => vec![0x10, 0x3F],
        Inherent::SWI3  => vec![0x11, 0x3F],
        Inherent::ASL(HalfAcc::A) => vec![0x48], // LSL
        Inherent::ASL(HalfAcc::B) => vec![0x58], // LSL
        Inherent::ASL(HalfAcc::D) => vec![0x10, 0x48], // LSL
        Inherent::ASR(HalfAcc::A) => vec![0x47],
        Inherent::ASR(HalfAcc::B) => vec![0x57],
        Inherent::ASR(HalfAcc::D) => vec![0x10, 0x47],
        Inherent::NEG(HalfAcc::A) => vec![0x40],
        Inherent::NEG(HalfAcc::B) => vec![0x50],
        Inherent::NEG(HalfAcc::D) => vec![0x10, 0x40],
        Inherent::CLR(FullAcc::A) => vec![0x4F],
        Inherent::CLR(FullAcc::B) => vec![0x5F],
        Inherent::CLR(FullAcc::D) => vec![0x10, 0x4F],
        Inherent::CLR(FullAcc::E) => vec![0x11, 0x4F],
        Inherent::CLR(FullAcc::F) => vec![0x11, 0x5F],
        Inherent::CLR(FullAcc::W) => vec![0x10, 0x5F],
        Inherent::COM(FullAcc::A) => vec![0x43],
        Inherent::COM(FullAcc::B) => vec![0x53],
        Inherent::COM(FullAcc::D) => vec![0x10, 0x43],
        Inherent::COM(FullAcc::E) => vec![0x11, 0x43],
        Inherent::COM(FullAcc::F) => vec![0x11, 0x53],
        Inherent::COM(FullAcc::W) => vec![0x10, 0x53],
        Inherent::DEC(FullAcc::A) => vec![0x4A],
        Inherent::DEC(FullAcc::B) => vec![0x5A],
        Inherent::DEC(FullAcc::D) => vec![0x10, 0x4A],
        Inherent::DEC(FullAcc::E) => vec![0x11, 0x4A],
        Inherent::DEC(FullAcc::F) => vec![0x11, 0x5A],
        Inherent::DEC(FullAcc::W) => vec![0x10, 0x5A],
        Inherent::INC(FullAcc::A) => vec![0x4C],
        Inherent::INC(FullAcc::B) => vec![0x5C],
        Inherent::INC(FullAcc::D) => vec![0x10, 0x4C],
        Inherent::INC(FullAcc::E) => vec![0x11, 0x4C],
        Inherent::INC(FullAcc::F) => vec![0x11, 0x5C],
        Inherent::INC(FullAcc::W) => vec![0x10, 0x5C],
        Inherent::TST(FullAcc::A) => vec![0x4D],
        Inherent::TST(FullAcc::B) => vec![0x5D],
        Inherent::TST(FullAcc::D) => vec![0x10, 0x4D],
        Inherent::TST(FullAcc::E) => vec![0x11, 0x4D],
        Inherent::TST(FullAcc::F) => vec![0x11, 0x5D],
        Inherent::TST(FullAcc::W) => vec![0x10, 0x5D],
        Inherent::LSR(ShiftAcc::A) => vec![0x44],
        Inherent::LSR(ShiftAcc::B) => vec![0x54],
        Inherent::LSR(ShiftAcc::D) => vec![0x10, 0x44],
        Inherent::LSR(ShiftAcc::W) => vec![0x10, 0x54],
        Inherent::ROL(ShiftAcc::A) => vec![0x49],
        Inherent::ROL(ShiftAcc::B) => vec![0x59],
        Inherent::ROL(ShiftAcc::D) => vec![0x10, 0x49],
        Inherent::ROL(ShiftAcc::W) => vec![0x10, 0x59],
        Inherent::ROR(ShiftAcc::A) => vec![0x46],
        Inherent::ROR(ShiftAcc::B) => vec![0x56],
        Inherent::ROR(ShiftAcc::D) => vec![0x10, 0x46],
        Inherent::ROR(ShiftAcc::W) => vec![0x10, 0x56],
    }
}

fn generate_imm8(inst: Imm8, post: u8) -> Vec<u8> {
    let mut opcode = match inst {
        Imm8::ADCR  => vec![0x10, 0x31],
        Imm8::ADDR  => vec![0x10, 0x30],
        Imm8::ANDR  => vec![0x10, 0x34],
        Imm8::CMPR  => vec![0x10, 0x37],
        Imm8::EORR  => vec![0x10, 0x36],
        Imm8::ORR   => vec![0x10, 0x35],
        Imm8::SBCR  => vec![0x10, 0x33],
        Imm8::SUBR  => vec![0x10, 0x32],
        Imm8::EXG   => vec![0x1E],
        Imm8::TFR   => vec![0x1F],
        Imm8::PSHS  => vec![0x34],
        Imm8::PSHU  => vec![0x36],
        Imm8::PULS  => vec![0x35],
        Imm8::PULU  => vec![0x37],
        Imm8::ANDCC => vec![0x1C],
        Imm8::ORCC  => vec![0x1A],
        Imm8::CWAI  => vec![0x3C],
        Imm8::BITMD => vec![0x11, 0x3C],
        Imm8::LDMD  => vec![0x11, 0x3D],
        Imm8::TFM(TfmMode::PlusPlus)   => vec![0x11, 0x38],
        Imm8::TFM(TfmMode::MinusMinus) => vec![0x11, 0x39],
        Imm8::TFM(TfmMode::PlusNone)   => vec![0x11, 0x3A],
        Imm8::TFM(TfmMode::NonePlus)   => vec![0x11, 0x3B],
    };
    opcode.push(post);
    opcode
}

fn generate_direct_bit(inst: DirectBit, post: u8, direct_addr: u8) -> Vec<u8> {
    let mut opcode = match inst {
        DirectBit::LDBT => vec![0x11, 0x36],
        DirectBit::STBT => vec![0x11, 0x37],
        DirectBit::BitMut(BitMode::AND, BitInv::AsIs)     => vec![0x11, 0x30],
        DirectBit::BitMut(BitMode::AND, BitInv::Inverted) => vec![0x11, 0x31],
        DirectBit::BitMut(BitMode::EOR, BitInv::AsIs)     => vec![0x11, 0x34],
        DirectBit::BitMut(BitMode::EOR, BitInv::Inverted) => vec![0x11, 0x35],
        DirectBit::BitMut(BitMode::OR,  BitInv::AsIs)     => vec![0x11, 0x32],
        DirectBit::BitMut(BitMode::OR,  BitInv::Inverted) => vec![0x11, 0x33],
    };
    opcode.push(post);
    opcode.push(direct_addr);
    opcode
}

fn write_u16(imm: u16) -> Vec<u8> {
    let mut buf = vec![0; 2];
    BigEndian::write_u16(buf.as_mut_slice(), imm);
    buf
}

fn write_u32(imm: u32) -> Vec<u8> {
    let mut buf = vec![0; 2];
    BigEndian::write_u32(buf.as_mut_slice(), imm);
    buf
}

fn generate_addr_indexed(post: u8, addr: IndexBytes) -> Vec<u8> {
    let mut result = vec![post];
    match addr {
        IndexBytes::None => (),
        IndexBytes::One(n) => result.push(n),
        IndexBytes::Two(n) => {
            result.extend(write_u16(n));
        },
    }
    result
}

fn generate_indexed(inst: Indexed, post: u8, addr: IndexBytes) -> Vec<u8> {
    let mut opcode = match inst {
        Indexed::LEA(StackReg::X) => vec![0x30],
        Indexed::LEA(StackReg::Y) => vec![0x31],
        Indexed::LEA(StackReg::U) => vec![0x33],
        Indexed::LEA(StackReg::S) => vec![0x32],
    };
    opcode.extend(generate_addr_indexed(post, addr));
    opcode
}

fn generate_opcode_addr(opcode: (Vec<u8>, Vec<u8>, Vec<u8>), addr: MemAddrMode) -> Vec<u8> {
    let mut result = Vec::new();
    match addr {
        MemAddrMode::Direct(_) => {
            result.extend(opcode.0);
        },
        MemAddrMode::Extended(_) => {
            result.extend(opcode.2);
        },
        MemAddrMode::Indexed(_, _) => {
            result.extend(opcode.1);
        },
    }
    result
}

fn generate_addr(addr: MemAddrMode) -> Vec<u8> {
    let mut result = Vec::new();
    match addr {
        MemAddrMode::Direct(n) => {
            result.push(n);
        },
        MemAddrMode::Extended(n) => {
            result.extend(write_u16(n));
        },
        MemAddrMode::Indexed(post, addr) => {
            result.extend(generate_addr_indexed(post, addr));
        },
    }
    result
}

fn generate_direct_mem(inst: DirectMem, addr: MemAddrMode) -> Vec<u8> {
    // Direct, Indexed, Extended
    let opcode: (Vec<u8>, Vec<u8>, Vec<u8>) = match inst {
        DirectMem::ASL => (vec![0x08], vec![0x68], vec![0x78]),
        DirectMem::ASR => (vec![0x07], vec![0x67], vec![0x77]),
        DirectMem::CLR => (vec![0x0F], vec![0x6F], vec![0x7F]),
        DirectMem::COM => (vec![0x03], vec![0x63], vec![0x73]),
        DirectMem::DEC => (vec![0x0A], vec![0x6A], vec![0x7A]),
        DirectMem::INC => (vec![0x0C], vec![0x6C], vec![0x7C]),
        DirectMem::JMP => (vec![0x0E], vec![0x6E], vec![0x7E]),
        DirectMem::JSR => (vec![0x9D], vec![0xAD], vec![0xBD]),
        DirectMem::LSR => (vec![0x04], vec![0x64], vec![0x74]),
        DirectMem::NEG => (vec![0x00], vec![0x60], vec![0x70]),
        DirectMem::ROL => (vec![0x09], vec![0x69], vec![0x79]),
        DirectMem::ROR => (vec![0x06], vec![0x66], vec![0x76]),
        DirectMem::TST => (vec![0x0D], vec![0x6D], vec![0x7D]),
        DirectMem::ST(StoreLoad::A) => (vec![0x97], vec![0xA7], vec![0xB7]),
        DirectMem::ST(StoreLoad::B) => (vec![0xD7], vec![0xE7], vec![0xF7]),
        DirectMem::ST(StoreLoad::D) => (vec![0xDD], vec![0xED], vec![0xFD]),
        DirectMem::ST(StoreLoad::E) => (vec![0x11, 0x97], vec![0x11, 0xA7], vec![0x11, 0xB7]),
        DirectMem::ST(StoreLoad::F) => (vec![0x11, 0xD7], vec![0x11, 0xE7], vec![0x11, 0xF7]),
        DirectMem::ST(StoreLoad::W) => (vec![0x10, 0x97], vec![0x10, 0xA7], vec![0x10, 0xB7]),
        DirectMem::ST(StoreLoad::Q) => (vec![0x10, 0xDD], vec![0x10, 0xED], vec![0x10, 0xFD]),
        DirectMem::ST(StoreLoad::S) => (vec![0x10, 0xDF], vec![0x10, 0xEF], vec![0x10, 0xFF]),
        DirectMem::ST(StoreLoad::U) => (vec![0xDF], vec![0xEF], vec![0xFF]),
        DirectMem::ST(StoreLoad::X) => (vec![0x9F], vec![0xAF], vec![0xBF]),
        DirectMem::ST(StoreLoad::Y) => (vec![0x10, 0x9F], vec![0x10, 0xAF], vec![0x10, 0xBF]),
    };
    let mut result = Vec::new();
    result.extend(generate_opcode_addr(opcode, addr));
    result.extend(generate_addr(addr));
    result
}

fn generate_logical_mem(inst: LogicalMem, imm8: u8, addr: MemAddrMode) -> Vec<u8> {
    let opcode: (Vec<u8>, Vec<u8>, Vec<u8>) = match inst {
        LogicalMem::AIM => (vec![0x02], vec![0x62], vec![0x72]),
        LogicalMem::EIM => (vec![0x05], vec![0x65], vec![0x75]),
        LogicalMem::OIM => (vec![0x01], vec![0x61], vec![0x71]),
        LogicalMem::TIM => (vec![0x0B], vec![0x6B], vec![0x7B]),
    };
    let mut result = Vec::new();
    result.extend(generate_opcode_addr(opcode, addr));
    result.push(imm8);
    result.extend(generate_addr(addr));
    result
}

fn generate_branch(inst: Branch, mode: BranchMode) -> Vec<u8> {
    let mut opcode = match (inst, mode) {
        // Exception: BRA, BRN, BSR
        (Branch::BRA, BranchMode::Short(_)) => vec![0x20],
        (Branch::BRA, BranchMode::Long(_))  => vec![0x16],
        (Branch::BRN, BranchMode::Short(_)) => vec![0x21],
        (Branch::BRN, BranchMode::Long(_))  => vec![0x10, 0x21],
        (Branch::BSR, BranchMode::Short(_)) => vec![0x8D],
        (Branch::BSR, BranchMode::Long(_))  => vec![0x17],
        // Rest: short 0xXX, long 0x10 0xXX
        (br, bm) => {
            let mut temp = match bm {
                BranchMode::Short(_) => vec![],
                BranchMode::Long(_)  => vec![0x10],
            };
            temp.push(match br {
                Branch::BRA | Branch::BRN | Branch::BSR  => panic!("Covered by special case opcode selection"),
                Branch::BHI => 0x22,
                Branch::BLS => 0x23,
                Branch::BCC => 0x24,
                Branch::BCS => 0x25,
                Branch::BNE => 0x26,
                Branch::BEQ => 0x27,
                Branch::BVC => 0x28,
                Branch::BVS => 0x29,
                Branch::BPL => 0x2A,
                Branch::BMI => 0x2B,
                Branch::BGE => 0x2C,
                Branch::BLT => 0x2D,
                Branch::BGT => 0x2E,
                Branch::BLE => 0x2F,
            });
            temp
        },
    };
    match mode {
        BranchMode::Short(rel) => opcode.push(rel as u8),
        BranchMode::Long(rel)  => opcode.extend(write_u16(rel as u16)),
    }
    opcode
}

// For validation
#[derive(Debug)]
enum ExpImm {Imm8, Imm16, Imm32}

fn generate_imm_mem(inst: ImmMem, imm_addr: ImmMemBytes) -> Vec<u8> {
    // (ExpImm, Immediate, Direct, Indexed, Extended)
    let opcode: (ExpImm, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) = match inst {
        ImmMem::ADC(HalfAcc::A) => (ExpImm::Imm8,  vec![0x89], vec![0x99], vec![0xA9], vec![0xB9]),
        ImmMem::ADC(HalfAcc::B) => (ExpImm::Imm8,  vec![0xC9], vec![0xD9], vec![0xE9], vec![0xF9]),
        ImmMem::ADC(HalfAcc::D) => (ExpImm::Imm16, vec![0x10, 0x89], vec![0x10, 0x99], vec![0x10, 0xA9], vec![0x10, 0xB9]),
        ImmMem::AND(HalfAcc::A) => (ExpImm::Imm8,  vec![0x84], vec![0x94], vec![0xA4], vec![0xB4]),
        ImmMem::AND(HalfAcc::B) => (ExpImm::Imm8,  vec![0xC4], vec![0xD4], vec![0xE4], vec![0xF4]),
        ImmMem::AND(HalfAcc::D) => (ExpImm::Imm16, vec![0x10, 0x84], vec![0x10, 0x94], vec![0x10, 0xA4], vec![0x10, 0xB4]),
        ImmMem::BIT(HalfAcc::A) => (ExpImm::Imm8,  vec![0x85], vec![0x95], vec![0xA5], vec![0xB5]),
        ImmMem::BIT(HalfAcc::B) => (ExpImm::Imm8,  vec![0xC5], vec![0xD5], vec![0xE5], vec![0xF5]),
        ImmMem::BIT(HalfAcc::D) => (ExpImm::Imm16, vec![0x10, 0x85], vec![0x10, 0x95], vec![0x10, 0xA5], vec![0x10, 0xB5]),
        ImmMem::EOR(HalfAcc::A) => (ExpImm::Imm8,  vec![0x88], vec![0x98], vec![0xA8], vec![0xB8]),
        ImmMem::EOR(HalfAcc::B) => (ExpImm::Imm8,  vec![0xC8], vec![0xD8], vec![0xE8], vec![0xF8]),
        ImmMem::EOR(HalfAcc::D) => (ExpImm::Imm16, vec![0x10, 0x88], vec![0x10, 0x98], vec![0x10, 0xA8], vec![0x10, 0xB8]),
        ImmMem::OR(HalfAcc::A)  => (ExpImm::Imm8,  vec![0x8A], vec![0x9A], vec![0xAA], vec![0xBA]),
        ImmMem::OR(HalfAcc::B)  => (ExpImm::Imm8,  vec![0xCA], vec![0xDA], vec![0xEA], vec![0xFA]),
        ImmMem::OR(HalfAcc::D)  => (ExpImm::Imm16, vec![0x10, 0x8A], vec![0x10, 0x9A], vec![0x10, 0xAA], vec![0x10, 0xBA]),
        ImmMem::SBC(HalfAcc::A) => (ExpImm::Imm8,  vec![0x82], vec![0x92], vec![0xA2], vec![0xB2]),
        ImmMem::SBC(HalfAcc::B) => (ExpImm::Imm8,  vec![0xC2], vec![0xD2], vec![0xE2], vec![0xF2]),
        ImmMem::SBC(HalfAcc::D) => (ExpImm::Imm16, vec![0x10, 0x82], vec![0x10, 0x92], vec![0x10, 0xA2], vec![0x10, 0xB2]),
        ImmMem::ADD(FullAcc::A) => (ExpImm::Imm8,  vec![0x8B], vec![0x9B], vec![0xAB], vec![0xBB]),
        ImmMem::ADD(FullAcc::B) => (ExpImm::Imm8,  vec![0xCB], vec![0xDB], vec![0xEB], vec![0xFB]),
        ImmMem::ADD(FullAcc::D) => (ExpImm::Imm16, vec![0xC3], vec![0xD3], vec![0xE3], vec![0xF3]),
        ImmMem::ADD(FullAcc::E) => (ExpImm::Imm8,  vec![0x11, 0x8B], vec![0x11, 0x9B], vec![0x11, 0xAB], vec![0x11, 0xBB]),
        ImmMem::ADD(FullAcc::F) => (ExpImm::Imm8,  vec![0x11, 0xCB], vec![0x11, 0xDB], vec![0x11, 0xEB], vec![0x11, 0xFB]),
        ImmMem::ADD(FullAcc::W) => (ExpImm::Imm16, vec![0x10, 0x8B], vec![0x10, 0x9B], vec![0x10, 0xAB], vec![0x10, 0xBB]),
        ImmMem::SUB(FullAcc::A) => (ExpImm::Imm8,  vec![0x80], vec![0x90], vec![0xA0], vec![0xB0]),
        ImmMem::SUB(FullAcc::B) => (ExpImm::Imm8,  vec![0xC0], vec![0xD0], vec![0xE0], vec![0xF0]),
        ImmMem::SUB(FullAcc::D) => (ExpImm::Imm16, vec![0x83], vec![0x93], vec![0xA3], vec![0xB3]),
        ImmMem::SUB(FullAcc::E) => (ExpImm::Imm8,  vec![0x11, 0x80], vec![0x11, 0x90], vec![0x11, 0xA0], vec![0x11, 0xB0]),
        ImmMem::SUB(FullAcc::F) => (ExpImm::Imm8,  vec![0x11, 0xC0], vec![0x11, 0xD0], vec![0x11, 0xE0], vec![0x11, 0xF0]),
        ImmMem::SUB(FullAcc::W) => (ExpImm::Imm16, vec![0x10, 0x80], vec![0x10, 0x90], vec![0x10, 0xA0], vec![0x10, 0xB0]),
        ImmMem::CmpAcc(FullAcc::A) => (ExpImm::Imm8,  vec![0x81], vec![0x91], vec![0xA1], vec![0xB1]),
        ImmMem::CmpAcc(FullAcc::B) => (ExpImm::Imm8,  vec![0xC1], vec![0xD1], vec![0xE1], vec![0xF1]),
        ImmMem::CmpAcc(FullAcc::D) => (ExpImm::Imm16, vec![0x10, 0x83], vec![0x10, 0x93], vec![0x10, 0xA3], vec![0x10, 0xB3]),
        ImmMem::CmpAcc(FullAcc::E) => (ExpImm::Imm8,  vec![0x11, 0x81], vec![0x11, 0x91], vec![0x11, 0xA1], vec![0x11, 0xB1]),
        ImmMem::CmpAcc(FullAcc::F) => (ExpImm::Imm8,  vec![0x11, 0xC1], vec![0x11, 0xD1], vec![0x11, 0xE1], vec![0x11, 0xF1]),
        ImmMem::CmpAcc(FullAcc::W) => (ExpImm::Imm16, vec![0x10, 0x81], vec![0x10, 0x91], vec![0x10, 0xA1], vec![0x10, 0xB1]),
        ImmMem::CmpStack(StackReg::S) => (ExpImm::Imm16, vec![0x11, 0x8C], vec![0x11, 0x9C], vec![0x11, 0xAC], vec![0x11, 0xBC]),
        ImmMem::CmpStack(StackReg::U) => (ExpImm::Imm16, vec![0x11, 0x83], vec![0x11, 0x93], vec![0x11, 0xA3], vec![0x11, 0xB3]),
        ImmMem::CmpStack(StackReg::X) => (ExpImm::Imm16, vec![0x8C], vec![0x9C], vec![0xAC], vec![0xBC]),
        ImmMem::CmpStack(StackReg::Y) => (ExpImm::Imm16, vec![0x10, 0x8C], vec![0x10, 0x9C], vec![0x10, 0xAC], vec![0x10, 0xBC]),
        ImmMem::LD(StoreLoad::A) => (ExpImm::Imm8,  vec![0x86], vec![0x96], vec![0xA6], vec![0xB6]),
        ImmMem::LD(StoreLoad::B) => (ExpImm::Imm8,  vec![0xC6], vec![0xD6], vec![0xE6], vec![0xF6]),
        ImmMem::LD(StoreLoad::D) => (ExpImm::Imm16, vec![0xCC], vec![0xDC], vec![0xEC], vec![0xFC]),
        ImmMem::LD(StoreLoad::E) => (ExpImm::Imm8,  vec![0x11, 0x86], vec![0x11, 0x96], vec![0x11, 0xA6], vec![0x11, 0xB6]),
        ImmMem::LD(StoreLoad::F) => (ExpImm::Imm8,  vec![0x11, 0xC6], vec![0x11, 0xD6], vec![0x11, 0xE6], vec![0x11, 0xF6]),
        ImmMem::LD(StoreLoad::W) => (ExpImm::Imm16, vec![0x10, 0x86], vec![0x10, 0x96], vec![0x10, 0xA6], vec![0x10, 0xB6]),
        ImmMem::LD(StoreLoad::Q) => (ExpImm::Imm32, vec![0xCD], vec![0x10, 0xDC], vec![0x10, 0xEC], vec![0x10, 0xFC]),
        ImmMem::LD(StoreLoad::S) => (ExpImm::Imm16, vec![0x10, 0xCE], vec![0x10, 0xDE], vec![0x10, 0xEE], vec![0x10, 0xFE]),
        ImmMem::LD(StoreLoad::U) => (ExpImm::Imm16, vec![0xCE], vec![0xDE], vec![0xEE], vec![0xFE]),
        ImmMem::LD(StoreLoad::X) => (ExpImm::Imm16, vec![0x8E], vec![0x9E], vec![0xAE], vec![0xBE]),
        ImmMem::LD(StoreLoad::Y) => (ExpImm::Imm16, vec![0x10, 0x8E], vec![0x10, 0x9E], vec![0x10, 0xAE], vec![0x10, 0xBE]),
        ImmMem::DIVD => (ExpImm::Imm8,  vec![0x11, 0x8D], vec![0x11, 0x9D], vec![0x11, 0xAD], vec![0x11, 0xBD]),
        ImmMem::DIVQ => (ExpImm::Imm16, vec![0x11, 0x8E], vec![0x11, 0x9E], vec![0x11, 0xAE], vec![0x11, 0xBE]),
        ImmMem::MULD => (ExpImm::Imm16, vec![0x11, 0x8F], vec![0x11, 0x9F], vec![0x11, 0xAF], vec![0x11, 0xBF]),
    };
    let mut result = Vec::new();

    // Make sure the Imm addr matches the ExpImm
    match (opcode.0, imm_addr) {
        (ExpImm::Imm8, ImmMemBytes::Imm8(imm)) => {
            result.extend(opcode.1);
            result.push(imm);
        },
        (ExpImm::Imm16, ImmMemBytes::Imm16(imm)) => {
            result.extend(opcode.1);
            result.extend(write_u16(imm));
        },
        (ExpImm::Imm32, ImmMemBytes::Imm32(imm)) => {
            result.extend(opcode.1);
            result.extend(write_u32(imm));
        },
        (_, ImmMemBytes::Mem(addr)) => {
            result.extend(generate_opcode_addr((opcode.2, opcode.3, opcode.4), addr));
            result.extend(generate_addr(addr));
        },
        (exp, act) => panic!("Expected: {:?}, but got {:?} for opcode: {:?}", exp, act, inst),
    }
    result
}


#[cfg(test)]
mod test_assembler {
    use super::*;

    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;

    use crate::asm::parser::parse_asm_inst;

    // To load up the external assembler file + object code for validation (LWTools assembler)
    const manifest: &str = env!("CARGO_MANIFEST_DIR");

    fn assert_zip(exp: Vec<u8>, obj: Vec<u8>) {
        // compare
        let mut iter = exp.into_iter().zip(obj);
        println!("Expected - Observed");
        for (e, o) in iter {
            println!("{:02x} - {:02x}", e, o);
            assert_eq!(e, o);
        }
    }

    fn test_assembly(asm_path: PathBuf, bin_path: PathBuf) {
        let asm: String = fs::read_to_string(asm_path).unwrap();
        let exp: Vec<u8> = fs::read(bin_path).unwrap();
        let obj = generate_object_code(parse_asm_inst(&asm).unwrap().1);
        assert_zip(exp, obj);
    }

    fn test_assembly_str(asm_str: &str, bin_path: PathBuf) {
        let exp: Vec<u8> = fs::read(bin_path).unwrap();
        let obj = generate_object_code(parse_asm_inst(&asm_str).unwrap().1);
        assert_zip(exp, obj);
    }

    #[test]
    fn test_inherent() {
        test_assembly(
            Path::new(manifest).join("test_asm/inherent.asm"),
            Path::new(manifest).join("test_asm/inherent.bin"),
        );
    }

    #[test]
    fn test_imm8() {
        test_assembly(
            Path::new(manifest).join("test_asm/imm8.asm"),
            Path::new(manifest).join("test_asm/imm8.bin"),
        );
    }

    #[test]
    fn test_special_imm8() {
        let obj = generate_object_code(parse_asm_inst(r#"
            BITMD /0
            LDMD NM
            ANDCC E,C
            ORCC E,C
            CWAI E,C
            "#).unwrap().1);
        let exp = vec![
            0x11, 0x3C, 0b1000_0000,
            0x11, 0x3D, 0b0000_0001,
            0x1C, 0b1000_0001,
            0x1A, 0b1000_0001,
            0x3C, 0b1000_0001,
        ];
        assert_eq!(exp, obj);
    }

    #[test]
    fn test_direct_bit() {
        let asm: String = fs::read_to_string(Path::new(manifest).join("test_asm/direct_bit.asm")).unwrap();
        // Need to modify the $40 to <0x40 to match our syntax
        let asm = asm.replace("$40", "<0x40");

        test_assembly_str(
            &asm,
            Path::new(manifest).join("test_asm/direct_bit.bin"),
        );
    }

    #[test]
    fn test_indexed() {
        let asm: String = fs::read_to_string(Path::new(manifest).join("test_asm/indexed.asm")).unwrap();
        // Need to modify the $4040 to 0x4040 to match our syntax
        let asm = asm.replace("$4040", "0x4040");
        // TODO: We might need to figure out how to handle PC vs PCR
        // PC in LWTools encodes the offset exactly, but PCR makes LWTool calculate the offset
        let asm = asm.replace("PC", "PCR");

        test_assembly_str(
            &asm,
            Path::new(manifest).join("test_asm/indexed.bin"),
        );
    }

    #[test]
    fn test_special_indexed() {
        // LWTool treats below as 5bit/8bit indexed not 0bit indexed
        // 6809.uk encodes the 0,Y and ,Y variant the same way
        // LEAX 0,Y == LEA ,Y
        // LEAX 0,W == LEA ,W
        // LEAX [0,Y] == LEA [,Y]
        // LEAX [0,W] == LEA [,W]
        let obj = generate_object_code(parse_asm_inst(r#"
            LEAX 0,Y
            LEAX 0,W
            LEAX [0,Y]
            LEAX [0,W]
            "#).unwrap().1);
        let exp = vec![
            0x30, 0xA4,
            0x30, 0x8F,
            0x30, 0xB4,
            0x30, 0x90,
        ];
        assert_eq!(exp, obj);
    }

    #[test]
    fn test_direct_mem() {
        let asm: String = fs::read_to_string(Path::new(manifest).join("test_asm/direct_mem.asm")).unwrap();
        // Need to modify the <$40 to <0x40 to match our syntax
        let asm = asm.replace("<$40", "<0x40");
        // Need to modify the >$4040 to >0x4040 to match our syntax
        let asm = asm.replace(">$4040", ">0x4040");

        test_assembly_str(
            &asm,
            Path::new(manifest).join("test_asm/direct_mem.bin"),
        );
    }

    #[test]
    fn test_logical_mem() {
        let asm: String = fs::read_to_string(Path::new(manifest).join("test_asm/logical_mem.asm")).unwrap();
        // Need to modify the <$40 to <0x40 to match our syntax
        let asm = asm.replace("<$40", "<0x40");
        // Need to modify the >$4040 to >0x4040 to match our syntax
        let asm = asm.replace(">$4040", ">0x4040");
        // Need to modify the #$18 to 0x18 to match our syntax
        let asm = asm.replace("#$18", "0x18");

        test_assembly_str(
            &asm,
            Path::new(manifest).join("test_asm/logical_mem.bin"),
        );
    }

    // TODO: make this and that branch test a bit better but this will do for now
    fn fix_branch(asm: &str) -> String {
        // Since there's no way to encode rel offset in LWTools, we instead did absolute
        // address which LWTools then adjusted to relative addressing of 0, so this function
        // is for fixing all of these back to 0x00 for our test code
        let addr = vec![
            "$64", "$60", "$5C", "$58", "$54", "$50", "$4C", "$48",
            "$44", "$40", "$3C", "$38", "$34", "$30", "$2C", "$29",
            "$25", "$22", "$20", "$1E", "$1C", "$1A", "$18", "$16",
            "$14", "$12", "$10", "$E", "$C", "$A", "$8", "$6", "$4", "$2",
        ];

        let mut ret = asm.to_string();
        for a in addr {
            ret = ret.replace(a, "0x00");
        }
        ret
    }

    #[test]
    fn test_branch() {
        let asm: String = fs::read_to_string(Path::new(manifest).join("test_asm/branch.asm")).unwrap();
        let asm = fix_branch(&asm);

        test_assembly_str(
            &asm,
            Path::new(manifest).join("test_asm/branch.bin"),
        );
    }

    // imm mem
}

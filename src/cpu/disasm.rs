#[derive(Debug, PartialEq, Eq)]
pub enum Reg8 {
    A, B, C, D, E, H, L, HLderef
}

impl Reg8 {
    pub fn extract(opcode: u8, lsb: u8) -> Self {
        match (opcode >> lsb) & 7 {
            0 => Self::B,
            1 => Self::C,
            2 => Self::D,
            3 => Self::E,
            4 => Self::H,
            5 => Self::L,
            6 => Self::HLderef,
            7 => Self::A,
            _ => unreachable!("Reg8 extraction failed")
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reg16 {
    BC, DE, HL, SP, AF, HLplus, HLminus
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reg16Kind {
    Normal,
    Stk,
    Mem,
}

impl Reg16 {
    pub fn extract(opcode: u8, kind: Reg16Kind) -> Self {
        match (opcode >> 4) & 3 {
            0 => Self::BC,
            1 => Self::DE,
            2 => match kind {
                Reg16Kind::Normal | Reg16Kind::Stk =>
                    Self::HL,
                Reg16Kind::Mem => Self::HLplus
            },
            3 => match kind {
                Reg16Kind::Normal => Self::SP,
                Reg16Kind::Stk => Self::AF,
                Reg16Kind::Mem => Self::HLminus,
            },
            _ => unreachable!("Reg16 extraction failed")
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Condition {
    NonZero,
    Zero,
    NonCarry,
    Carry
}

impl Condition {
    pub fn extract(opcode: u8) -> Self {
        match (opcode >> 3) & 3 {
            0 => Self::NonZero,
            1 => Self::Zero,
            2 => Self::NonCarry,
            3 => Self::Carry,
            _ => unreachable!("Condition extraction failed")
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Operand {
    Imm8,
    Reg8(Reg8),
    Imm16,
    Reg16(Reg16),
    AddrIndirect(Reg16),
    AddrDirect16,
    AddrIndirectLow8(Reg8),
    AddrDirectLow8
}

#[derive(Default, Debug, PartialEq, Eq)]
pub enum Instruction {
    // Block 0
    #[default]
    NOP,

    // dovrebbero starci tutti gli LD

    INC(Operand),
    DEC(Operand),

    LD(Operand, Operand),

    RLCA,
    RRCA,
    RLA,
    RRA,
    DAA,
    CPL,
    SCF,
    CCF,

    JR(Option<Condition>, Operand),

    STOP,

    // Block 1 (reg-to-reg 8 bit loads)
    HALT,

    // Block 2 (8 bit arithmetic) / Block 3
    ADD(Operand, Operand),
    ADC(Operand, Operand),
    SUB(Operand, Operand),
    SBC(Operand, Operand),
    AND(Operand, Operand),
    XOR(Operand, Operand),
    OR(Operand, Operand),
    CP(Operand, Operand),

    // Block 3
    RET(Option<Condition>),
    RETI,
    JP(Option<Condition>, Operand),
    CALL(Option<Condition>, Operand),
    RST,

    POP(Reg16),
    PUSH(Reg16),
    LDH(Operand, Operand),

    DI,
    EI,

    Hardlock
}

impl Instruction {
    #[allow(dead_code)]
    pub fn decode(opcode: u8) -> Option<Self> {
        match opcode {
            0x00 => Some(Self::NOP),
            0x03 | 0x13 | 0x23 | 0x33 =>
                Some(Self::INC(Operand::Reg16(
                    Reg16::extract(opcode, Reg16Kind::Normal)
                ))),
            0x04 | 0x0c | 0x14 | 0x1c |
            0x24 | 0x2c | 0x34 | 0x3c =>
                Some(Self::INC(Operand::Reg8(
                    Reg8::extract(opcode, 3)
                ))),
            0x05 | 0x0d | 0x15 | 0x1d |
            0x25 | 0x2d | 0x35 | 0x3d =>
                Some(Self::DEC(Operand::Reg8(
                    Reg8::extract(opcode, 3)
                ))),
            0x0b | 0x1b | 0x2b | 0x3b =>
                Some(Self::DEC(Operand::Reg16(
                    Reg16::extract(opcode, Reg16Kind::Normal)
                ))),
            0x09 | 0x19 | 0x29 | 0x39 =>
                Some(Self::ADD(
                    Operand::Reg16(Reg16::HL),
                    Operand::Reg16(Reg16::extract(opcode, Reg16Kind::Normal))
                )),

            0x06 | 0x0e | 0x16 | 0x1e |
            0x26 | 0x2e | 0x36 | 0x3e =>
                Some(Self::LD(
                    Operand::Reg8(Reg8::extract(opcode, 3)),
                    Operand::Imm8
                )),

            0x08 => Some(Self::LD(
                    Operand::AddrDirect16,
                    Operand::Reg16(Reg16::SP)
            )),

            0x0a | 0x1a | 0x2a | 0x3a => Some(Self::LD(
                    Operand::Reg8(Reg8::A),
                    Operand::AddrIndirect(Reg16::extract(opcode, Reg16Kind::Mem))
            )),

            0x02 | 0x12 | 0x22 | 0x32 => Some(Self::LD(
                    Operand::AddrIndirect(Reg16::extract(opcode, Reg16Kind::Mem)),
                    Operand::Reg8(Reg8::A)
            )),

            0x01 | 0x11 | 0x21 | 0x31 => Some(Self::LD(
                    Operand::Reg16(Reg16::extract(opcode, Reg16Kind::Normal)),
                    Operand::Imm16
            )),


            0x07 => Some(Self::RLCA),
            0x0f => Some(Self::RRCA),
            0x17 => Some(Self::RLA),
            0x1f => Some(Self::RRA),
            0x27 => Some(Self::DAA),
            0x2f => Some(Self::CPL),
            0x37 => Some(Self::SCF),
            0x3f => Some(Self::CCF),

            0x18 => Some(Self::JR(None, Operand::Imm8)),
            0x20 | 0x28 | 0x30 | 0x38 =>
                Some(Self::JR(
                    Some(Condition::extract(opcode)),
                    Operand::Imm8
                )),

            0x10 => Some(Self::STOP),

            // Block 1
            
            0x76 => Some(Self::HALT),
            0x40..=0x7F => {
                let dst = Operand::Reg8(Reg8::extract(opcode, 3));
                let src = Operand::Reg8(Reg8::extract(opcode, 0));
                Some(Self::LD(dst, src))
            },

            // Block 2
            0x80..=0x87 => Some(Self::ADD(
                Operand::Reg8(Reg8::A),
                Operand::Reg8(
                    Reg8::extract(opcode, 0)
                )
            )),
            0x88..=0x8f => Some(Self::ADC(
                Operand::Reg8(Reg8::A),
                Operand::Reg8(
                    Reg8::extract(opcode, 0)
                )
            )),
            0x90..=0x97 => Some(Self::SUB(
                Operand::Reg8(Reg8::A),
                Operand::Reg8(
                    Reg8::extract(opcode, 0)
                )
            )),
            0x98..=0x9f => Some(Self::SBC(
                Operand::Reg8(Reg8::A),
                Operand::Reg8(
                    Reg8::extract(opcode, 0)
                )
            )),
            0xa0..=0xa7 => Some(Self::AND(
                Operand::Reg8(Reg8::A),
                Operand::Reg8(
                    Reg8::extract(opcode, 0)
                )
            )),
            0xa8..=0xaf => Some(Self::XOR(
                Operand::Reg8(Reg8::A),
                Operand::Reg8(
                    Reg8::extract(opcode, 0)
                )
            )),
            0xb0..=0xb7 => Some(Self::OR(
                Operand::Reg8(Reg8::A),
                Operand::Reg8(
                    Reg8::extract(opcode, 0)
                )
            )),
            0xb8..=0xbf => Some(Self::CP(
                Operand::Reg8(Reg8::A),
                Operand::Reg8(
                    Reg8::extract(opcode, 0)
                )
            )),

            // Block 3
            0xc6 => Some(Self::ADD(Operand::Reg8(Reg8::A), Operand::Imm8)),
            0xce => Some(Self::ADC(Operand::Reg8(Reg8::A), Operand::Imm8)),
            0xd6 => Some(Self::SUB(Operand::Reg8(Reg8::A), Operand::Imm8)),
            0xde => Some(Self::SBC(Operand::Reg8(Reg8::A), Operand::Imm8)),
            0xe6 => Some(Self::AND(Operand::Reg8(Reg8::A), Operand::Imm8)),
            0xee => Some(Self::XOR(Operand::Reg8(Reg8::A), Operand::Imm8)),
            0xf6 => Some(Self::OR(Operand::Reg8(Reg8::A), Operand::Imm8)),
            0xfe => Some(Self::CP(Operand::Reg8(Reg8::A), Operand::Imm8)),

            0xc0 | 0xc8 | 0xd0 | 0xd8 => 
                Some(Self::RET(Some(Condition::extract(opcode)))),
            0xc9 => Some(Self::RET(None)),
            0xd9 => Some(Self::RETI),

            0xc2 | 0xca | 0xd2 | 0xda =>
                Some(Self::JP(
                    Some(Condition::extract(opcode)),
                    Operand::Imm16
                )),

            0xc3 => Some(Self::JP(
                        None,
                        Operand::Imm16
                    )),

            0xe9 => Some(Self::JP(
                        None,
                        Operand::Reg16(Reg16::HL)
                    )),

            0xc4 | 0xcc | 0xd4 | 0xdc =>
                Some(Self::CALL(
                    Some(Condition::extract(opcode)),
                    Operand::Imm16
                )),

            0xcd => Some(Self::CALL(
                None,
                Operand::Imm16
            )),

            0xc7..=0xff if (opcode & 0xc7) == 0xc7 =>
                todo!("RST not implemented"),

            0xc1..=0xf1 if (opcode & 0xcf) == 0xc1 =>
                Some(Self::POP(Reg16::extract(opcode, Reg16Kind::Stk))),

            0xc5..=0xf5 if (opcode & 0xcf) == 0xc5 =>
                Some(Self::PUSH(Reg16::extract(opcode, Reg16Kind::Stk))),

            0xe2 => Some(Self::LDH(
                Operand::AddrIndirectLow8(Reg8::C),
                Operand::Reg8(Reg8::A)
            )),
            0xe0 => Some(Self::LDH(
                Operand::AddrDirectLow8,
                Operand::Reg8(Reg8::A)
            )),
            0xea => Some(Self::LD(
                Operand::AddrDirect16,
                Operand::Reg8(Reg8::A)
            )),
            0xf2 => Some(Self::LDH(
                Operand::Reg8(Reg8::A),
                Operand::AddrIndirectLow8(Reg8::C),
            )),
            0xf0 => Some(Self::LDH(
                Operand::Reg8(Reg8::A),
                Operand::AddrDirectLow8,
            )),
            0xfa => Some(Self::LD(
                Operand::Reg8(Reg8::A),
                Operand::AddrDirect16,
            )),

            0xcb => todo!("Prefix not implemented"),

            0xe8 => Some(Self::ADD(Operand::Reg16(Reg16::SP), Operand::Imm8)),

            0xf3 => Some(Self::DI),
            0xfb => Some(Self::EI),
            
            // Hardlock instructions
            0xd3 | 0xdb | 0xdd | 0xe3 | 0xe4 | 0xeb | 0xec |
            0xed | 0xf4 | 0xfc | 0xfd => Some(Self::Hardlock),
            
            _ => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disasm() {
        let cases = vec![
            (
                vec![0x00],
                Some(Instruction::NOP)
            ),
            (
                vec![0x23],
                Some(Instruction::INC(Operand::Reg16(Reg16::HL)))
            ),
            (
                vec![0x7],
                Some(Instruction::RLCA)
            ),
            (
                vec![0x38, 0x05],
                Some(Instruction::JR(
                    Some(Condition::Carry),
                    Operand::Imm8
                ))
            ),
            (
                vec![0x2d],
                Some(Instruction::DEC(
                    Operand::Reg8(Reg8::L)
                ))
            ),
            (
                vec![0x72],
                Some(Instruction::LD(
                    Operand::Reg8(Reg8::HLderef),
                    Operand::Reg8(Reg8::D)
                ))
            ),
            (
                vec![0x76],
                Some(Instruction::HALT)
            ),
            (
                vec![0xa9],
                Some(Instruction::XOR(
                    Operand::Reg8(Reg8::A),
                    Operand::Reg8(Reg8::C),
                ))
            ),
            (
                vec![0xc6, 0x08],
                Some(Instruction::ADD(
                    Operand::Reg8(Reg8::A),
                    Operand::Imm8,
                ))
            ),

        ];

        for (rom, expected) in cases {
            let instr = Instruction::decode(rom[0]);
            assert!(
                instr == expected,
                "Failed disasm: expected {:?}, got {:?}",
                expected, instr
            );
        }

    }
    
}


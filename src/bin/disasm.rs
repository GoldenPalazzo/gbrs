#[derive(Debug, PartialEq, Eq)]
enum Reg8 {
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
enum Reg16 {
    BC, DE, HL, SP, AF, HLplus, HLminus
}

#[derive(Debug, PartialEq, Eq)]
enum Reg16Kind {
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
enum Condition {
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
enum ArithmeticOperand8 {
    Imm(u8),
    Reg(Reg8),
}

#[derive(Debug, PartialEq, Eq)]
enum ArithmeticOperand16 {
    Imm(u16),
    Reg(Reg16),
}

#[derive(Debug, PartialEq, Eq)]
enum Instruction {
    // Block 0
    NOP,

    // dovrebbero starci tutti gli LD

    INC16(Reg16),
    INC8(Reg8),
    DEC16(Reg16),
    DEC8(Reg8),

    ADDHL(Reg16),

    LD8(Reg8, u8),

    RLCA,
    RRCA,
    RLA,
    RRA,
    DAA,
    CPL,
    SCF,
    CCF,

    JR(Option<Condition>, u8),

    STOP,

    // Block 1 (reg-to-reg 8 bit loads)
    HALT,
    LD8RR(Reg8, Reg8),

    // Block 2 (8 bit arithmetic) / Block 3
    ADD(ArithmeticOperand8),
    ADC(ArithmeticOperand8),
    SUB(ArithmeticOperand8),
    SBC(ArithmeticOperand8),
    AND(ArithmeticOperand8),
    XOR(ArithmeticOperand8),
    OR(ArithmeticOperand8),
    CP(ArithmeticOperand8),

    // Block 3
    RET(Option<Condition>),
    RETI,
    JP(Option<Condition>, ArithmeticOperand16),
    CALL(Option<Condition>, u16),
    RST,

    POP(Reg16),
    PUSH(Reg16),

    ADDSP(u8),

    DI,
    EI,

    Hardlock
}

impl Instruction {
    #[allow(dead_code)]
    pub fn decode(buf: &[u8], addr: usize) -> Option<Self> {
        let opcode = buf[addr];
        match opcode {
            0x00 => Some(Self::NOP),
            0x03 | 0x13 | 0x23 | 0x33 =>
                Some(Self::INC16(
                    Reg16::extract(opcode, Reg16Kind::Normal)
                )),
            0x04 | 0x0c | 0x14 | 0x1c |
            0x24 | 0x2c | 0x34 | 0x3c =>
                Some(Self::INC8(Reg8::extract(opcode, 3))),
            0x05 | 0x0d | 0x15 | 0x1d |
            0x25 | 0x2d | 0x35 | 0x3d =>
                Some(Self::DEC8(Reg8::extract(opcode, 3))),
            0x0b | 0x1b | 0x2b | 0x3b =>
                Some(Self::DEC16(
                    Reg16::extract(opcode, Reg16Kind::Normal)
                )),
            0x09 | 0x19 | 0x29 | 0x39 =>
                Some(Self::ADDHL(
                    Reg16::extract(opcode, Reg16Kind::Normal)
                )),

            0x06 | 0x0e | 0x16 | 0x1e |
            0x26 | 0x2e | 0x36 | 0x3e =>
                Some(Self::LD8(
                    Reg8::extract(opcode, 3),
                    buf[addr+1]
                )),

            0x07 => Some(Self::RLCA),
            0x0f => Some(Self::RRCA),
            0x17 => Some(Self::RLA),
            0x1f => Some(Self::RRA),
            0x27 => Some(Self::DAA),
            0x2f => Some(Self::CPL),
            0x37 => Some(Self::SCF),
            0x3f => Some(Self::CCF),

            0x18 => Some(Self::JR(None, buf[addr+1])),
            0x20 | 0x28 | 0x30 | 0x38 =>
                Some(Self::JR(
                    Some(Condition::extract(opcode)),
                    buf[addr+1]
                )),

            0x10 => Some(Self::STOP),

            // Block 1
            
            0x76 => Some(Self::HALT),
            0x40..=0x7F => {
                let dst = Reg8::extract(opcode, 3);
                let src = Reg8::extract(opcode, 0);
                Some(Self::LD8RR(dst, src))
            },

            // Block 2
            0x80..=0x87 => Some(Self::ADD(
                ArithmeticOperand8::Reg(
                    Reg8::extract(opcode, 0)
                )
            )),
            0x88..=0x8f => Some(Self::ADC(
                ArithmeticOperand8::Reg(
                    Reg8::extract(opcode, 0)
                )
            )),
            0x90..=0x97 => Some(Self::SUB(
                ArithmeticOperand8::Reg(
                    Reg8::extract(opcode, 0)
                )
            )),
            0x98..=0x9f => Some(Self::SBC(
                ArithmeticOperand8::Reg(
                    Reg8::extract(opcode, 0)
                )
            )),
            0xa0..=0xa7 => Some(Self::AND(
                ArithmeticOperand8::Reg(
                    Reg8::extract(opcode, 0)
                )
            )),
            0xa8..=0xaf => Some(Self::XOR(
                ArithmeticOperand8::Reg(
                    Reg8::extract(opcode, 0)
                )
            )),
            0xb0..=0xb7 => Some(Self::OR(
                ArithmeticOperand8::Reg(
                    Reg8::extract(opcode, 0)
                )
            )),
            0xb8..=0xbf => Some(Self::CP(
                ArithmeticOperand8::Reg(
                    Reg8::extract(opcode, 0)
                )
            )),

            // Block 3
            0xc6 => Some(Self::ADD(ArithmeticOperand8::Imm(buf[addr+1]))),
            0xce => Some(Self::ADC(ArithmeticOperand8::Imm(buf[addr+1]))),
            0xd6 => Some(Self::SUB(ArithmeticOperand8::Imm(buf[addr+1]))),
            0xde => Some(Self::SBC(ArithmeticOperand8::Imm(buf[addr+1]))),
            0xe6 => Some(Self::AND(ArithmeticOperand8::Imm(buf[addr+1]))),
            0xee => Some(Self::XOR(ArithmeticOperand8::Imm(buf[addr+1]))),
            0xf6 => Some(Self::OR(ArithmeticOperand8::Imm(buf[addr+1]))),
            0xfe => Some(Self::CP(ArithmeticOperand8::Imm(buf[addr+1]))),

            0xc0 | 0xc8 | 0xd0 | 0xd8 => 
                Some(Self::RET(Some(Condition::extract(opcode)))),
            0xc9 => Some(Self::RET(None)),
            0xd9 => Some(Self::RETI),

            0xc2 | 0xca | 0xd2 | 0xda =>
                Some(Self::JP(
                    Some(Condition::extract(opcode)),
                    ArithmeticOperand16::Imm(
                        u16::from_le_bytes([buf[addr+1],buf[addr+2]])
                    )
                )),

            0xc3 => Some(Self::JP(
                        None,
                        ArithmeticOperand16::Imm(
                            u16::from_le_bytes([buf[addr+1],buf[addr+2]])
                        )
                    )),

            0xe9 => Some(Self::JP(
                        None,
                        ArithmeticOperand16::Reg(
                            Reg16::HL
                        )
                    )),

            0xc4 | 0xcc | 0xd4 | 0xdc =>
                Some(Self::CALL(
                    Some(Condition::extract(opcode)),
                    u16::from_le_bytes([buf[addr+1],buf[addr+2]])
                )),

            0xcd => Some(Self::CALL(
                None,
                u16::from_le_bytes([buf[addr+1],buf[addr+2]])
            )),

            0xc7..=0xff if (opcode & 0xc7) == 0xc7 =>
                todo!("RST not implemented"),

            0xc1..=0xf1 if (opcode & 0xcf) == 0xc1 =>
                Some(Self::POP(Reg16::extract(opcode, Reg16Kind::Stk))),

            0xc1..=0xf1 if (opcode & 0xcf) == 0xc5 =>
                Some(Self::PUSH(Reg16::extract(opcode, Reg16Kind::Stk))),

            0xcb => todo!("Prefix not implemented"),

            0xe8 => Some(Self::ADDSP(buf[addr+1])),


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
                Some(Instruction::INC16(Reg16::HL))
            ),
            (
                vec![0x7],
                Some(Instruction::RLCA)
            ),
            (
                vec![0x38, 0x05],
                Some(Instruction::JR(
                    Some(Condition::Carry),
                    5
                ))
            ),
            (
                vec![0x2d],
                Some(Instruction::DEC8(
                    Reg8::L
                ))
            ),
            (
                vec![0x72],
                Some(Instruction::LD8RR(
                    Reg8::HLderef,
                    Reg8::D
                ))
            ),
            (
                vec![0x76],
                Some(Instruction::HALT)
            ),
            (
                vec![0xa9],
                Some(Instruction::XOR(
                    ArithmeticOperand8::Reg(Reg8::C),
                ))
            ),
            (
                vec![0xc6, 0x08],
                Some(Instruction::ADD(
                    ArithmeticOperand8::Imm(8),
                ))
            ),

        ];

        for (rom, expected) in cases {
            let instr = Instruction::decode(&rom, 0);
            assert!(
                instr == expected,
                "Failed disasm: expected {:?}, got {:?}",
                expected, instr
            );
        }

    }
    
}

fn main() {
    println!("GB disasm");
}

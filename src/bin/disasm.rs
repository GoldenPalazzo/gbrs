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
            _ => panic!("Reg8 extraction failed")
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
            _ => panic!("Reg16 extraction failed")
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
            _ => panic!("Condition extraction failed")
        }
    }
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

    STOP
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

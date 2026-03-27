#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_implementations() {
        assert_eq!(
            add_acc(200, 168, true),
            AluResult {
                val: Some(113),
                z: Some(false),
                n: Some(false),
                h: Some(true),
                c: Some(true),
            }
        );
        assert_eq!(
            sub(0, 0, false),
            AluResult {
                val: Some(0),
                c: Some(false),
                h: Some(false),
                n: Some(true),
                z: Some(true),
            }
        );
        assert_eq!(
            add_acc(255, 255, true),
            AluResult {
                val: Some(255),
                z: Some(false),
                n: Some(false),
                h: Some(true),
                c: Some(true),
            }
        );
        assert_eq!(
            add_hl(0xffab, 0x0123),
            AluResult {
                val: Some(206),
                z: None,
                n: Some(false),
                h: Some(true),
                c: Some(true),
            }
        );
        assert_eq!(
            add_sp(0xffab, 1),
            AluResult {
                val: Some(0xffac),
                z: Some(false),
                n: Some(false),
                h: Some(false),
                c: Some(false),
            }
        );
        assert_eq!(
            add_sp(0xffab, 0xff as u8 as i8),
            AluResult {
                val: Some(0xffaa),
                z: Some(false),
                n: Some(false),
                h: Some(true),
                c: Some(true),
            }
        );
        assert_eq!(
            lrotate(0xf7, true, None),
            AluResult {
                val: Some(0xef),
                z: Some(false),
                n: Some(false),
                h: Some(false),
                c: Some(true),
            }
        );
        assert_eq!(
            rrotate(0xf6, true, Some(false)),
            AluResult {
                val: Some(0x7b),
                z: Some(false),
                n: Some(false),
                h: Some(false),
                c: Some(false),
            }
        );
        assert_eq!(
            complement(0b11101011),
            AluResult {
                val: Some(0b00010100),
                z: None,
                n: Some(true),
                h: Some(true),
                c: None,
            }
        );
    }
}

#[derive(Debug, PartialEq)]
pub struct AluResult {
    pub val: Option<u16>,
    pub z: Option<bool>,
    pub n: Option<bool>,
    pub h: Option<bool>,
    pub c: Option<bool>,
}

pub fn add_acc(a: u8, b: u8, carry: bool) -> AluResult {
    let c = if carry { 1 } else { 0 };
    let res = a.wrapping_add(b).wrapping_add(c);
    AluResult {
        val: Some(res as u16),
        z: Some(res == 0),
        n: Some(false),
        h: Some((a & 0xf) + (b & 0xf) + c > 0xf),
        c: Some((a as u16 + b as u16 + c as u16) > 0xff),
    }
}

pub fn add_hl(hl: u16, b: u16) -> AluResult {
    let res = hl.wrapping_add(b);
    AluResult {
        val: Some(res),
        z: None,
        n: Some(false),
        h: Some((hl & 0xfff) + (b & 0xfff) > 0xfff),
        c: Some(res < hl || res < b),
    }
}

pub fn add_sp(sp: u16, b: i8) -> AluResult {
    let e8_u16 = b as i16 as u16;
    let res = sp.wrapping_add(e8_u16);
    AluResult {
        val: Some(res),
        z: Some(false),
        n: Some(false),
        h: Some((sp & 0xf) + (e8_u16 & 0xf) > 0xf),
        c: Some((sp & 0xff) + (e8_u16 & 0xff) > 0xff),
    }
}

pub fn sub(dst: u8, src: u8, carry: bool) -> AluResult {
    let c = if carry { 1 } else { 0 };
    let res = dst.wrapping_sub(src).wrapping_sub(c);
    AluResult {
        val: Some(res as u16),
        z: Some(res == 0),
        n: Some(true),
        h: Some((dst & 0xf) < (src & 0xf) + c),
        c: Some(src as u16 + c as u16 > dst as u16),
    }
}

pub fn inc_u8(dst: u8) -> AluResult {
    let mut res = add_acc(dst, 1, false);
    res.c = None;
    res
}

pub fn inc_u16(dst: u16) -> AluResult {
    AluResult {
        val: Some(dst.wrapping_add(1)),
        z: None,
        n: None,
        h: None,
        c: None,
    }
}

pub fn dec_u8(dst: u8) -> AluResult {
    let mut res = sub(dst, 1, false);
    res.c = None;
    res
}

pub fn dec_u16(dst: u16) -> AluResult {
    AluResult {
        val: Some(dst.wrapping_sub(1)),
        z: None,
        n: None,
        h: None,
        c: None,
    }
}

pub fn lrotate(a: u8, allow_zero: bool, old_carry: Option<bool>) -> AluResult {
    let mut res = a.rotate_left(1);
    let new_carry = (res & 1) == 1;
    if let Some(c) = old_carry {
        match c {
            true => res |= 1,
            false => res &= 0xfe,
        }
    }
    AluResult {
        val: Some(res as u16),
        z: Some(allow_zero && res == 0),
        n: Some(false),
        h: Some(false),
        c: Some(new_carry),
    }
}

pub fn rrotate(a: u8, allow_zero: bool, old_carry: Option<bool>) -> AluResult {
    let mut res = a.rotate_right(1);
    let new_carry = (a & 1) == 1;
    if let Some(c) = old_carry {
        match c {
            true => res |= 0x80,
            false => res &= 0x7f,
        }
    }
    AluResult {
        val: Some(res as u16),
        z: Some(allow_zero && res == 0),
        n: Some(false),
        h: Some(false),
        c: Some(new_carry),
    }
}

pub fn complement(a: u8) -> AluResult {
    AluResult {
        val: Some((!a) as u16),
        z: None,
        n: Some(true),
        h: Some(true),
        c: None,
    }
}

pub fn and(a: u8, op: u8) -> AluResult {
    let val = (a & op) as u16;
    AluResult {
        val: Some(val),
        z: Some(val == 0),
        n: Some(false),
        h: Some(true),
        c: Some(false),
    }
}

pub fn or(a: u8, op: u8) -> AluResult {
    let val = (a | op) as u16;
    AluResult {
        val: Some(val),
        z: Some(val == 0),
        n: Some(false),
        h: Some(false),
        c: Some(false),
    }
}

pub fn xor(a: u8, op: u8) -> AluResult {
    let val = (a ^ op) as u16;
    AluResult {
        val: Some(val),
        z: Some(val == 0),
        n: Some(false),
        h: Some(false),
        c: Some(false),
    }
}

pub fn compare(a: u8, op: u8) -> AluResult {
    let res = sub(a, op, false);
    AluResult { val: None, ..res }
}

pub fn sla(a: u8) -> AluResult {
    lrotate(a, true, Some(false)) // carry = bit uscente (7), bit 0 = 0
}

pub fn srl(a: u8) -> AluResult {
    rrotate(a, true, Some(false)) // carry = bit 0, bit 7 = 0
}

pub fn sra(a: u8) -> AluResult {
    let mut res = rrotate(a, true, Some(false));
    // preserva il bit 7 originale
    let msb = a & 0x80;
    res.val = Some((res.val.unwrap() as u8 | msb) as u16);
    res.z = Some((res.val.unwrap() as u8) == 0);
    res
}

pub fn swap(a: u8) -> AluResult {
    let res = a.rotate_left(4);
    AluResult {
        val: Some(res as u16),
        z: Some(res == 0),
        n: Some(false),
        h: Some(false),
        c: Some(false),
    }
}

pub fn bit(u: u8, r8: u8) -> AluResult {
    AluResult {
        val: None,
        z: Some(r8 & (1 << u) == 0),
        n: Some(false),
        h: Some(true),
        c: None,
    }
}

pub fn set(u: u8, r8: u8) -> AluResult {
    let res = r8 | (1 << u);
    AluResult {
        val: Some(res as u16),
        z: None,
        n: None,
        h: None,
        c: None,
    }
}

pub fn reset(u: u8, r8: u8) -> AluResult {
    let res = r8 & !(1 << u);
    AluResult {
        val: Some(res as u16),
        z: None,
        n: None,
        h: None,
        c: None,
    }
}

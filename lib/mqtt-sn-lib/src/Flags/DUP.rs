#[derive(Debug, Clone)]
pub struct DUP {
    pub level: u8,
}

impl DUP {
    #[inline(always)]
    pub fn new(flags: u8) -> u8 {
        let level = (flags & 0b1000_0000) >> 7;
        let dup = DUP { level };
        dup.level
    }
}


#[derive(Debug, Clone)]
pub struct Retain {
    pub level: u8,
}

impl Retain {
    #[inline(always)]
    pub fn new(flags: u8) -> u8 {
        let level = (flags & 0b0001_0000) >> 4;
        let retain = Retain { level };
        retain.level
    }
}

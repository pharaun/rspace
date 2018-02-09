use std::ops::Index;
use std::ops::IndexMut;

#[derive(Debug)]
pub struct RegFile {
    _x0: u32,
    reg: [u32; 31]
}

impl RegFile {
    pub fn new(reg: [u32; 31]) -> RegFile {
        RegFile { _x0: 0, reg: reg }
    }
}

impl Index<usize> for RegFile {
    type Output = u32;

    fn index(&self, idx: usize) -> &u32 {
        match idx {
            0 => &0,
            _ => &self.reg[idx-1],
        }
    }
}

impl IndexMut<usize> for RegFile {
    fn index_mut<'a>(&'a mut self, idx: usize) -> &'a mut u32 {
        match idx {
            // TODO: this feel like a hack, can we get rid of _x0?
            0 => & mut self._x0,
            _ => & mut self.reg[idx-1],
        }

    }
}


#[test]
fn regfile_test() {
    let mut reg = RegFile::new([0; 31]);
    reg.reg[0] = 1;
    reg.reg[1] = 2;
    reg.reg[30] = 3;

    assert_eq!(reg[0], 0);
    assert_eq!(reg[1], 1);
    assert_eq!(reg[2], 2);
    assert_eq!(reg[31], 3);

    // Test writing to.
    reg[0] = 10;
    reg[1] = 11;
    reg[2] = 12;
    reg[31] = 13;

    assert_eq!(reg[0], 0);
    assert_eq!(reg[1], 11);
    assert_eq!(reg[2], 12);
    assert_eq!(reg[31], 13);
}

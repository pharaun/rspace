// CSR access stuff
// csr has 4096 bytes (12bit) of addressable space
pub struct Csr {
    _ro_hole: u32,
    csr: [u32; 4096]
}

impl Csr {
    pub fn new(csr: [u32; 4096]) -> Csr {
        Csr {
            _ro_hole: 0,
            csr: csr,
        }
    }

    // CSR access/etc
    pub fn read_write(&mut self, csr: usize, val: u32) -> u32 {
        let ret = self.csr[csr];
        self.csr[csr] = val;
        ret
    }

    pub fn read_set(&mut self, csr: usize, val: u32) -> u32 {
        let ret = self.csr[csr];
        self.csr[csr] = ret | val;
        ret
    }

    pub fn read_clear(&mut self, csr: usize, val: u32) -> u32 {
        let ret = self.csr[csr];
        self.csr[csr] = (!val) & ret;
        ret
    }
}

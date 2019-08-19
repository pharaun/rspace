// CSR access stuff
// csr has 4096 bytes (12bit) of addressable space
pub struct Csr {
    _ro_hole: u32,
    csr: [u32; 4096]
}

// By convention, the upper 4 bits of the CSR address (csr[11:8]) are used to encode the read and
// write accessibility of the CSRs according to privilege level as shown in Table 2.1. The top two
// bits (csr[11:10]) indicate whether the register is read/write (00, 01, or 10) or read-only (11).
// The next two bits (csr[9:8]) encode the lowest privilege level that can access the CSR.
//
// Attempts to access a non-existent CSR raise an illegal instruction exception. Attempts to access
// a CSR without appropriate privilege level or to write a read-only register also raise illegal
// instruction exceptions. A read/write register might also contain some bits that are read-only,
// in which case writes to the read-only bits are ignored.

impl Csr {
    pub fn new(csr: [u32; 4096]) -> Csr {
        Csr {
            _ro_hole: 0,
            csr: csr,
        }
    }

    // CSR access/etc
    pub fn read(&self, csr: usize) -> u32 {
        self.csr[csr]
    }

    pub fn write(&mut self, csr: usize, val: u32) {
        self.csr[csr] = val;
    }

    pub fn set(&mut self, csr: usize, val: u32) {
        let old_val = self.csr[csr];
        self.csr[csr] = old_val | val;
    }

    pub fn clear(&mut self, csr: usize, val: u32) {
        let old_val = self.csr[csr];
        self.csr[csr] = old_val & (!val);
    }
}

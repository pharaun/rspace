// CSR access stuff
// csr has 4096 bytes (12bit) of addressable space
pub struct Csr {
    _ro_hole: u32,
    csr: [u32; 4096]
}

// TODO:
// - implement some form of csr map (to map out existant csr and hook em up to side effect code)
// - Implement reset (ie section 3.3) for the defined csr values, and put cause into mcause and
// reset pc to reset addr
// - Implement NMI (non-maskable-interrupts) (aka emulator/system failure)

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
        if let Some(val) = hardcoded_csr(csr) {
            val
        } else {
            self.csr[csr]
        }
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

// Read only
fn hardcoded_csr(csr: usize) -> Option<u32> {
    match csr {
        // MISA
        0x301 => {
            let mut ret = 0x0;

            // ISA xlen is 32
            ret |= 0b01 << 29;

            // ISA RV32 I
            ret |= 1 << 8;

            // ISA RV32 M
            ret |= 1 << 12;

            // Maybe ISA RV32 X (non standard extensions)
            // ret |= 1 << 23;

            Some(ret)
        },
        // MVENDORID
        // Non comerical implementation
        0xF11 => Some(0x0),
        // MARCHID
        // Not implemented & not established opensource project
        0xF12 => Some(0x0),
        // MIMPID
        // Version 1 of this implementation
        0xF13 => Some(0x1),
        // MHARTID
        //
        // Integer ID of the hardware thread running the code.
        // At least one hart must have a hart ID of zero.
        // Hart ID must be unique.
        //
        // Currently only have one hart per emulator environment so hardcoding.
        0xF14 => Some(0x0),
        // MSTATUS
        //
        // TODO: This isn't a read only but let's expose it here right now pending move to better spot
        0x300 => {
            let ret = MachineStatus::new();
            Some(ret.into())
        },
        // MTVEC
        //
        // TODO: This isn't a read only but let's expose it here right now pending move to better spot
        0x305 => {
            let ret = TrapVec::new();
            Some(ret.into())
        },
        // MIE
        //
        // TODO: This isn't a read only but let's expose it here right now pending move to better spot
        0x304 => {
            let ret = MachineInterrupt::new();
            Some(ret.into())
        },
        // MIP
        //
        // TODO: This isn't a read only but let's expose it here right now pending move to better spot
        0x344 => {
            let ret = MachineInterrupt::new();
            Some(ret.into())
        },
        _ => None,
    }
}


// Machine status
//
// We only provide the M priv level, there's no M + U separation.
// Any field not explicitly defined are impliedly hardcoded to 0.
//
// Omitting mpp field since its 2 value is for the lower priv modes
// and we don't provide S or U priv mode.
//
// mpriv is hardwired to 0 (since we don't support S or U priv mode).
//
// mxr is hardwired to 0 (since we don't support S priv mode) and
// since we also don't offer page-based virtual memory.
//
// tw is hardwired to 0 (since we only offer M priv mode), used with WFI inst.
//
struct MachineStatus {
    mie: bool,
    mpie: bool,
}

impl MachineStatus {
    pub fn new() -> MachineStatus {
        MachineStatus {
            mie: true,
            mpie: false,
        }
    }
}

impl From<MachineStatus> for u32 {
    fn from(original: MachineStatus) -> u32 {
        let mut ret = 0x0;

        if original.mie {
            ret |= 1 << 3;
        }

        if original.mpie {
            ret |= 1 << 7;
        }

        ret
    }
}


// Trap Vector Configuration
//
// The value in the BASE field must always be aligned on a 4-byte boundary,
// and the MODE setting may impose additional alignment constraints on the value in the BASE field.
//
// TODO: ponder making base omit the lowest 2 byte (to force 4-byte boundary).
struct TrapVec {
    mode: VecMode,
    base: u32,
}

// When MODE=Vectored, all synchronous exceptions into machine mode cause the pc
// to be set to the address in the BASE field, whereas interrupts cause the pc to
// be set to the address in the BASE field plus four times the interrupt cause
// number. For example, a machine-mode timer interrupt (see Table 3.6 on page 37)
// causes the pc to be set to BASE+0x1c.
//
// Reset and NMI vector locations are given in a platform specification.
enum VecMode {
    Direct,   // All exceptions set pc to BASE.
    Vectored, // Asynchronous interrupts set pc to BASE+4×cause.
}

impl TrapVec {
    pub fn new() -> TrapVec {
        TrapVec {
            mode: VecMode::Direct,
            base: 0x200,
        }
    }
}

impl From<TrapVec> for u32 {
    fn from(original: TrapVec) -> u32 {
        let mut ret = 0x0;

        // Base
        ret |= original.base << 2;

        // Mode
        ret |= match original.mode {
            VecMode::Direct   => 0b00,
            VecMode::Vectored => 0b01,
        };

        ret
    }
}

// Machine Interrupt (Pending | Enabled)
//
// We only provide the M priv level interrupts
//
// We will need an external interrupt controller that muxes all of the incoming
// interrupt from external hardware into the MEI interrupt.
//
// The MTIP bit is read-only and is cleared by writing to the memory-mapped
// machine-mode timer compare register.
//
// The machine-level MSIP bits are written by accesses to memory-mapped control
// registers, which are used by remote harts to provide machine-mode interprocessor
// interrupts. A hart can write its own MSIP bit using the same memory-mapped
// control register.
//
// The MEIP field in mip is a read-only bit that indicates a machine-mode
// external interrupt is pending. MEIP is set and cleared by a platform-specific
// interrupt controller. The MEIE field in mie enables machine external
// interrupts when set.
//
// The non-maskable interrupt is not made visible via the mip register as its
// presence is implicitly known when executing the NMI trap handler.
//
// Implementations may add additional platform-specific interrupt sources to
// bits 16 and above of the mip and mie registers. Some platforms may avail
// these interrupts for custom use.
//
// Multiple simultaneous interrupts destined for the same privilege mode are
// handled in the following decreasing priority order: MEI, MSI, MTI.
// Synchronous exceptions are of lower priority than all interrupts.
//
struct MachineInterrupt {
    // Software Interrupt
    msi: bool,
    // Timer Interrupt
    mti: bool,
    // External Interrupt
    mei: bool,
}

impl MachineInterrupt {
    pub fn new() -> MachineInterrupt {
        MachineInterrupt {
            msi: false,
            mti: false,
            mei: false,
        }
    }
}

impl From<MachineInterrupt> for u32 {
    fn from(original: MachineInterrupt) -> u32 {
        let mut ret = 0x0;

        if original.msi {
            ret |= 1 << 3;
        }

        if original.mti {
            ret |= 1 << 7;
        }

        if original.mei {
            ret |= 1 << 11;
        }

        ret
    }
}

trait Mem {
    fn load_byte(&self, idx: usize) -> u8;
    fn store_byte(&mut self, idx: usize, data: u8);
    fn size(&self) -> usize;

    fn print(&self);
}

struct Ram {
    size: usize,
    ram: [u8; 8],
}

impl Ram {
    fn new() -> Ram {
        Ram {
            size: 8,
            ram: [0; 8],
        }
    }
}

impl Mem for Ram {
    fn load_byte(&self, idx: usize) -> u8 {
        self.ram[idx]
    }

    fn store_byte(&mut self, idx: usize, data: u8) {
        self.ram[idx] = data;
    }

    fn size(&self) -> usize {
        self.size
    }

    fn print(&self) {
        println!("{:?}", self.ram);
    }
}


struct MemMap {
    map: Vec<Box<dyn Mem>>,
}

impl MemMap {
    fn new() -> MemMap {
        MemMap {
            map: vec![],
        }
    }

    fn add<T: Mem + 'static>(&mut self, mem: T) {
        self.map.push(Box::new(mem));
    }
}

impl Mem for MemMap {
    fn load_byte(&self, idx: usize) -> u8 {
        let mut cur_idx = idx;
        let mut ret = 0x0;

        for t in &(self.map) {
            if cur_idx > t.size() {
                cur_idx -= t.size();
            } else {
                ret = t.load_byte(cur_idx);
                break;
            }
        }
        ret
    }

    fn store_byte(&mut self, idx: usize, data: u8) {
        let mut cur_idx = idx;

        for t in self.map.iter_mut() {
            if cur_idx > t.size() {
                cur_idx -= t.size();
            } else {
                t.store_byte(cur_idx, data);
                break;
            }
        }
    }

    fn size(&self) -> usize {
        let mut size = 0;
        for t in &(self.map) {
            size += t.size();
        }
        size
    }

    fn print(&self) {
        for t in &(self.map) {
            t.print();
        }
    }
}


#[test]
fn ram_test() {
    let mut ram = Ram::new();

    ram.store_byte(1, 0x10);
    ram.store_byte(2, 0x11);

    assert_eq!(ram.ram[1], 0x10);
    assert_eq!(ram.ram[2], 0x11);

    assert_eq!(ram.load_byte(1), 0x10);
    assert_eq!(ram.load_byte(2), 0x11);
}

#[test]
fn mem_map_size_test() {
    let mut mem_map = MemMap::new();
    assert_eq!(mem_map.size(), 0);

    mem_map.add(Ram::new());
    assert_eq!(mem_map.size(), 8);

    mem_map.add(Ram::new());
    assert_eq!(mem_map.size(), 16);
}

#[test]
fn mem_map_store_read_test() {
    let mut mem_map = MemMap::new();
    mem_map.add(Ram::new());
    mem_map.add(Ram::new());

    mem_map.store_byte(1, 0x10);
    mem_map.store_byte(1+8, 0x11);

    assert_eq!(mem_map.load_byte(1), 0x10);
    assert_eq!(mem_map.load_byte(1+8), 0x11);
}

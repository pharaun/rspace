use byteorder::{ByteOrder, LittleEndian};


pub fn read_byte(block: &[u8], offset: usize) -> u8 {
    block[offset]
}

pub fn read_half(block: &[u8], offset: usize) -> u16 {
    LittleEndian::read_u16(&block[offset..=(offset+1)])
}

pub fn read_word(block: &[u8], offset: usize) -> u32 {
    LittleEndian::read_u32(&block[offset..=(offset+3)])
}

pub fn read_dword(block: &[u8], offset: usize) -> u64 {
    LittleEndian::read_u64(&block[offset..=(offset+7)])
}

pub fn write_byte(block: &mut [u8], offset: usize, data: u8) {
    block[offset] = data;
}

pub fn write_half(block: &mut [u8], offset: usize, data: u16) {
    LittleEndian::write_u16(&mut block[offset..=(offset+1)], data);
}

pub fn write_word(block: &mut [u8], offset: usize, data: u32) {
    LittleEndian::write_u32(&mut block[offset..=(offset+3)], data);
}

pub fn write_dword(block: &mut [u8], offset: usize, data: u64) {
    LittleEndian::write_u64(&mut block[offset..=(offset+7)], data);
}

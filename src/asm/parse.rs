include!(concat!(env!("OUT_DIR"), "/asm/parse.rs"));


pub fn is_csr(csr: &str) -> bool {
    match csr {
        "CYCLE"   | "CYCLEH"    => true,
        "TIME"    | "TIMEH"     => true,
        "INSTRET" | "INSTRETH"  => true,
        _                       => false,
    }
}

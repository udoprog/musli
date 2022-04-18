use std::io;

#[inline(never)]
#[no_mangle]
pub fn do_encode(a: u32, b: u32) -> Result<Vec<u8>, io::Error> {
    let mut out = Vec::new();
    out.extend(a.to_le_bytes());
    out.extend(b.to_le_bytes());
    Ok(out)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

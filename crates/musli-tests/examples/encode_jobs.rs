use anyhow::Result;
use musli::{Decode, Encode};

#[derive(Encode, Decode)]
#[musli(packed)]
struct Payload {
    a: u32,
    b: u32,
}

#[inline(never)]
#[no_mangle]
pub fn do_encode(a: u32, b: u32) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    musli_wire::encode(&mut out, &Payload { a, b })?;
    Ok(out)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

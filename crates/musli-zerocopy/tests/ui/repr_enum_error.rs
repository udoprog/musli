use musli_zerocopy::ZeroCopy;

#[derive(ZeroCopy)]
#[repr(C, packed)]
enum ReprPackedC {
    Variant(u32),
}

#[derive(ZeroCopy)]
#[repr(u8, packed)]
enum ReprPackedU8 {
    Variant(u32),
}

#[derive(ZeroCopy)]
#[repr(C)]
enum ReprC {
    Variant(u32),
}

fn main() {
}

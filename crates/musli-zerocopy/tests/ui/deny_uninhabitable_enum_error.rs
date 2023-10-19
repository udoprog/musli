use musli_zerocopy::ZeroCopy;

#[derive(Debug, Clone, Copy, PartialEq, ZeroCopy)]
#[repr(u8)]
enum Uninhabitable {}

fn main() {
}

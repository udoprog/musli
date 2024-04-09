const CAPACITY: usize = 4096;

/// A no-std bytes buffer.
#[allow(unused)]
pub type Bytes = musli_utils::fixed::FixedBytes<CAPACITY>;

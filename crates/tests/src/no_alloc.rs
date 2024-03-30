const CAPACITY: usize = 4096;

/// A no-std bytes buffer.
pub type Bytes = musli_common::exports::fixed::FixedBytes<CAPACITY>;

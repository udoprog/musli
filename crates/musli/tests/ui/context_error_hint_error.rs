use core::fmt;

use musli::context::Capture;
use musli::mode::Binary;

#[derive(Debug)]
struct MyError;

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MyError")
    }
}

fn main() {
    let _cx = Capture::<MyError, _>::new();
}

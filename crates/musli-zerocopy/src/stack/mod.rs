pub(crate) use self::array_stack::ArrayStack;
mod array_stack;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Trait used to define a stack.
pub(crate) trait Stack<T> {
    const CAPACITY: usize;

    fn new() -> Self;

    fn try_push(&mut self, value: T) -> bool;

    fn pop(&mut self) -> Option<T>;
}

#[cfg(feature = "alloc")]
impl<T> Stack<T> for Vec<T> {
    const CAPACITY: usize = usize::MAX;

    #[inline]
    fn new() -> Self {
        Vec::new()
    }

    #[inline]
    fn try_push(&mut self, value: T) -> bool {
        Vec::push(self, value);
        true
    }

    #[inline]
    fn pop(&mut self) -> Option<T> {
        Vec::pop(self)
    }
}

pub struct Box<T, A> where A: Allocator {
    buf: A::RawVec<T>,
}

impl Box<T, A> where A: Allocator {
    /// Construct a new buffer vector.
    ///
    /// ## Examples
    ///
    /// ```
    /// use musli::alloc::Vec;
    ///
    /// musli::alloc::default(|alloc| {
    ///     let mut a = Vec::new_in(alloc);
    ///
    ///     a.push(String::from("Hello"));
    ///     a.push(String::from("World"));
    ///
    ///     assert_eq!(a.as_slice(), ["Hello", "World"]);
    /// });
    /// ```
    #[inline]
    pub fn new_in(value: T, alloc: A) -> Self {
        let buffer = alloc.new_raw_vec::<T>();

        Self {
            buf: ,
            len: 0,
        }
    }
}

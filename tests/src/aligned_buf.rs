use alloc::alloc;

use core::alloc::Layout;
use core::mem::size_of_val;
use core::ptr::NonNull;
use core::slice;

/// A bytes vector that can have a specific alignment.
pub struct AlignedBuf {
    alignment: usize,
    len: usize,
    capacity: usize,
    data: NonNull<u8>,
}

impl AlignedBuf {
    /// Construct an alignable vec with the given alignment.
    #[inline]
    pub fn new(alignment: usize) -> Self {
        assert!(alignment.is_power_of_two());

        Self {
            alignment,
            len: 0,
            capacity: 0,
            data: unsafe { dangling(alignment) },
        }
    }

    #[inline]
    pub fn reserve(&mut self, capacity: usize) {
        let new_capacity = self.len + capacity;
        self.ensure_capacity(new_capacity);
    }

    #[inline]
    pub fn extend_from_slice(&mut self, bytes: &[u8]) {
        self.reserve(bytes.len());

        // SAFETY: We just allocated space for the slice.
        unsafe {
            self.store_bytes(bytes);
        }
    }

    #[inline]
    pub(crate) unsafe fn store_bytes<T>(&mut self, values: &[T]) {
        unsafe {
            let dst = self.data.as_ptr().wrapping_add(self.len);
            dst.copy_from_nonoverlapping(values.as_ptr().cast(), size_of_val(values));
            self.len += size_of_val(values);
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data.as_ptr() as *const _, self.len) }
    }

    #[inline(never)]
    fn ensure_capacity(&mut self, new_capacity: usize) {
        if self.capacity >= new_capacity {
            return;
        }

        let new_capacity = new_capacity.max((self.capacity as f32 * 1.5) as usize);
        let (old_layout, new_layout) = self.layouts(new_capacity);

        if old_layout.size() == 0 {
            self.alloc_init(new_layout);
        } else {
            self.alloc_realloc(old_layout, new_layout);
        }
    }

    /// Perform the initial allocation with the given layout and capacity.
    fn alloc_init(&mut self, new_layout: Layout) {
        unsafe {
            let ptr = alloc::alloc(new_layout);

            if ptr.is_null() {
                alloc::handle_alloc_error(new_layout);
            }

            self.data = NonNull::new_unchecked(ptr);
            self.capacity = new_layout.size();
        }
    }

    /// Reallocate, note that the alignment of the old layout must match the new
    /// one.
    fn alloc_realloc(&mut self, old_layout: Layout, new_layout: Layout) {
        debug_assert_eq!(old_layout.align(), new_layout.align());

        unsafe {
            let ptr = alloc::realloc(self.data.as_ptr(), old_layout, new_layout.size());

            if ptr.is_null() {
                alloc::handle_alloc_error(old_layout);
            }

            // NB: We may simply forget the old allocation, since `realloc` is
            // responsible for freeing it.
            self.data = NonNull::new_unchecked(ptr);
            self.capacity = new_layout.size();
        }
    }

    /// Return a pair of the currently allocated layout, and new layout that is
    /// requested with the given capacity.
    #[inline]
    fn layouts(&self, new_capacity: usize) -> (Layout, Layout) {
        // SAFETY: The existing layout cannot be invalid since it's either
        // checked as it's replacing the old layout, or is initialized with
        // known good values.
        let old_layout =
            unsafe { Layout::from_size_align_unchecked(self.capacity, self.alignment) };
        let layout =
            Layout::from_size_align(new_capacity, self.alignment).expect("Proposed layout invalid");
        (old_layout, layout)
    }
}

impl Drop for AlignedBuf {
    fn drop(&mut self) {
        unsafe {
            if self.capacity != 0 {
                // SAFETY: This is guaranteed to be valid per the construction
                // of this type.
                let layout = Layout::from_size_align_unchecked(self.capacity, self.alignment);
                alloc::dealloc(self.data.as_ptr(), layout);
                self.capacity = 0;
            }
        }
    }
}

const unsafe fn dangling(align: usize) -> NonNull<u8> {
    unsafe { NonNull::new_unchecked(invalid_mut(align)) }
}

// Replace with `core::ptr::invalid_mut` once stable.
#[allow(clippy::useless_transmute)]
const fn invalid_mut<T>(addr: usize) -> *mut T {
    // FIXME(strict_provenance_magic): I am magic and should be a compiler
    // intrinsic. We use transmute rather than a cast so tools like Miri can
    // tell that this is *not* the same as from_exposed_addr. SAFETY: every
    // valid integer is also a valid pointer (as long as you don't dereference
    // that pointer).
    unsafe { core::mem::transmute(addr) }
}

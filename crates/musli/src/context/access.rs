use core::cell::Cell;

/// Guarded access to some underlying state.
pub(crate) struct Access {
    state: Cell<isize>,
}

impl Access {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            state: Cell::new(0),
        }
    }

    #[inline]
    pub(crate) fn shared(&self) -> Shared<'_> {
        let state = self.state.get();

        if state > 0 {
            panic!("Context is exclusively held")
        }

        if state == isize::MIN {
            crate::no_std::abort("access state overflowed");
        }

        self.state.set(state - 1);
        Shared { access: self }
    }

    #[inline]
    pub(crate) fn exclusive(&self) -> Exlusive<'_> {
        let state = self.state.get();

        if state != 0 {
            panic!("Context is already in shared use")
        }

        if state == isize::MIN {
            crate::no_std::abort("access state overflowed");
        }

        self.state.set(1);
        Exlusive { access: self }
    }
}

/// A shared access to some underlying state.
pub(crate) struct Shared<'a> {
    access: &'a Access,
}

impl Drop for Shared<'_> {
    fn drop(&mut self) {
        self.access.state.set(self.access.state.get() + 1);
    }
}

impl Clone for Shared<'_> {
    fn clone(&self) -> Self {
        // Shared state is already acquired, so we simply decrement it one more.
        self.access.state.set(self.access.state.get() - 1);
        Shared {
            access: self.access,
        }
    }
}

/// An exclusive access to some underlying state.
pub(crate) struct Exlusive<'a> {
    access: &'a Access,
}

impl Drop for Exlusive<'_> {
    fn drop(&mut self) {
        self.access.state.set(0);
    }
}

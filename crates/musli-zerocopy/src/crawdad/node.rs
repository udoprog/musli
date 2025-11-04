use super::OFFSET_MASK;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Node {
    pub(super) base: u32,
    pub(super) check: u32,
}

impl Node {
    #[inline(always)]
    pub(super) const fn get_base(&self) -> u32 {
        self.base & OFFSET_MASK
    }

    #[inline(always)]
    pub(super) const fn get_check(&self) -> u32 {
        self.check & OFFSET_MASK
    }

    #[inline(always)]
    pub(super) const fn is_leaf(&self) -> bool {
        self.base & !OFFSET_MASK != 0
    }

    #[inline(always)]
    pub(super) const fn has_leaf(&self) -> bool {
        self.check & !OFFSET_MASK != 0
    }

    #[inline(always)]
    pub(super) const fn is_vacant(&self) -> bool {
        self.base == OFFSET_MASK && self.check == OFFSET_MASK
    }
}

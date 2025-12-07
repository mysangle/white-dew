
use std::{
    alloc::{Allocator, AllocError, Layout},
    ptr::NonNull,
};
use super::release;

/// A debug wrapper for [`release::Arena`].
pub enum Arena {
    Delegated { delegate: &'static release::Arena, borrow: usize },
    Owned { arena: release::Arena },
}

impl Arena {
    pub const fn empty() -> Self {
        Self::Owned { arena: release::Arena::empty() }
    }

    #[inline]
    pub(super) fn delegate_target_unchecked(&self) -> &release::Arena {
        match self {
            Self::Delegated { delegate, .. } => delegate,
            Self::Owned { arena } => arena,
        }
    }

    #[inline]
    pub(super) fn delegate_target(&self) -> &release::Arena {
        match *self {
            Self::Delegated { delegate, borrow } => {
                assert!(
                    borrow == delegate.borrows.get(),
                    "Arena already borrowed by a newer ScratchArena"
                );
                delegate
            }
            Self::Owned { ref arena } => arena,
        }
    }

    pub(super) fn delegated(delegate: &release::Arena) -> Self {
        let borrow = delegate.borrows.get() + 1;
        delegate.borrows.set(borrow);
        // &*(delegate as *const _)를 사용해서 lifetime을 'static으로 변경
        Self::Delegated { delegate: unsafe { &*(delegate as *const _) }, borrow }
    }

    pub unsafe fn reset(&self, to: usize) {
        unsafe { self.delegate_target().reset(to) }
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::empty()
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        if let Self::Delegated { delegate, borrow } = self {
            // 자신 이후에 추가 borrow가 없는 경우
            let borrows = delegate.borrows.get();
            assert_eq!(*borrow, borrows);
            delegate.borrows.set(borrows - 1);
        }
    }
}

unsafe impl Allocator for Arena {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.delegate_target().alloc_raw(layout.size(), layout.align())
    }

    unsafe fn deallocate(&self, _: NonNull<u8>, _: Layout) {}
}

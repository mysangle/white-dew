
use crate::sys;
use std::{
    alloc::{Allocator, AllocError, Layout},
    cell::Cell,
    ptr::NonNull,
    slice,
};

const ALLOC_CHUNK_SIZE: usize = 64 * 1024; // 64KB

/// single thread에서 동작하는 Arena
pub struct Arena {
    base: NonNull<u8>,
    capacity: usize,
    commit: Cell<usize>,
    offset: Cell<usize>,

    pub(super) borrows: Cell<usize>,
}

impl Arena {
    pub const fn empty() -> Self {
        Self {
            base: NonNull::dangling(),
            capacity: 0,
            commit: Cell::new(0),
            offset: Cell::new(0),
            borrows: Cell::new(0),
        }
    }

    pub fn new(capacity: usize) -> Result<Self, AllocError> {
        // 최소 1이상의 용량 확보 + ALLOC_CHUNK_SIZE의 배수로 조정
        // 예: 100 KB를 요청하면 128 KB를 할당
        let capacity = (capacity.max(1) + ALLOC_CHUNK_SIZE - 1) & !(ALLOC_CHUNK_SIZE - 1);
        let base = unsafe { sys::virtual_reserve(capacity)? };

        Ok(Self {
            base,
            capacity,
            commit: Cell::new(0),
            offset: Cell::new(0),
            borrows: Cell::new(0),
        })
    }

    pub fn offset(&self) -> usize {
        self.offset.get()
    }

    /// 1. 할당 해제하는 메모리 영역의 값을 0xDD로 변경한다.
    /// 2. Arena의 offset을 이전 위치로 되돌린다.
    pub unsafe fn reset(&self, to: usize) {
        // Fill the deallocated memory with 0xDD to aid debugging.
        if self.offset.get() > to {
            let commit = self.commit.get();
            let len = (self.offset.get() + 128).min(commit) - to;
            unsafe { slice::from_raw_parts_mut(self.base.add(to).as_ptr(), len).fill(0xDD) };
        }

        self.offset.replace(to);
    }

    #[inline]
    pub(super) fn alloc_raw(&self, bytes: usize, alignment: usize) -> Result<NonNull<[u8]>, AllocError> {
        let commit = self.commit.get();
        let offset = self.offset.get();

        let beg = (offset + alignment - 1) & !(alignment - 1);
        let end = beg + bytes;

        if end > commit {
            // 사용가능한 메모리 영역을 초과하므로 추가 메모리 요청
            return self.alloc_raw_bump(beg, end);
        }

        {
            let ptr = unsafe { self.base.add(offset) };
            let len = (end + 128).min(self.commit.get()) - offset;
            unsafe { slice::from_raw_parts_mut(ptr.as_ptr(), len).fill(0xCD) };
        }

        self.offset.replace(end);
        Ok(unsafe { NonNull::slice_from_raw_parts(self.base.add(beg), bytes) })
    }

    #[cold]
    fn alloc_raw_bump(&self, beg: usize, end: usize) -> Result<NonNull<[u8]>, AllocError> {
        let offset = self.offset.get();
        let commit_old = self.commit.get();
        let commit_new = (end + ALLOC_CHUNK_SIZE - 1) & !(ALLOC_CHUNK_SIZE - 1);

        // 요청을 위해 계산된 크기가 Arena의 전체 용량을 넘는지 확인
        if commit_new > self.capacity
            || unsafe {
                sys::virtual_commit(self.base.add(commit_old), commit_new - commit_old).is_err()
            }
        {
            return Err(AllocError);
        }

        {
            let ptr = unsafe { self.base.add(offset) };
            let len = (end + 128).min(self.commit.get()) - offset;
            unsafe { slice::from_raw_parts_mut(ptr.as_ptr(), len).fill(0xCD) };
        }

        self.commit.replace(commit_new);
        self.offset.replace(end);
        Ok(unsafe { NonNull::slice_from_raw_parts(self.base.add(beg), end - beg) })
    }
}

unsafe impl Allocator for Arena {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.alloc_raw(layout.size(), layout.align())
    }

    unsafe fn deallocate(&self, _: NonNull<u8>, _: Layout) {}
}

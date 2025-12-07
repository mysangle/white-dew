
use crate::helpers::opt_ptr_eq;
use super::{debug, release, Arena};
use std::{alloc::AllocError, ops::Deref};

static mut S_SCRATCH: [release::Arena; 2] =
    const { [release::Arena::empty(), release::Arena::empty()] };

pub struct ScratchArena<'a> {
    arena: debug::Arena,
    // 생성 시점에 저장해두는 Arena의 offset
    offset: usize,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> ScratchArena<'a> {
    fn new(arena: &'a release::Arena) -> Self {
        let offset = arena.offset();
        ScratchArena {
            arena: Arena::delegated(arena),
            _phantom: std::marker::PhantomData,
            offset,
        }
    }
}

impl Drop for ScratchArena<'_> {
    fn drop(&mut self) {
        unsafe { self.arena.reset(self.offset) };
    }
}

impl Deref for ScratchArena<'_> {
    type Target = debug::Arena;

    fn deref(&self) -> &Self::Target {
        &self.arena
    }
}

pub fn init(capacity: usize) -> Result<(), AllocError> {
    unsafe {
        for s in &mut S_SCRATCH[..] {
            *s = release::Arena::new(capacity)?;
        }
    }
    Ok(())
}

pub fn scratch_arena(conflict: Option<&Arena>) -> ScratchArena<'static> {
    unsafe {
        let conflict = conflict.map(|a| a.delegate_target_unchecked());

        let index = opt_ptr_eq(conflict, Some(&S_SCRATCH[0])) as usize;
        let arena = &S_SCRATCH[index];
        ScratchArena::new(arena)
    }
}

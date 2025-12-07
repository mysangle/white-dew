
use std::{
    alloc::AllocError,
    ffi::c_int,
    ptr::{null_mut, NonNull},
};

/// 처음에 mmap으로 전체 가상 메모리 할당을 하지만 PROT_NONE으로 접근 권한을 주지 않는다. => virtual_reserve
/// 나중에 실제로 사용할 때 필요한 부분에 대해서만 mprotect를 사용하여 접근 권한을 준다. => virtual_commit
/// 
/// virtual memory 할당
pub unsafe fn virtual_reserve(size: usize) -> Result<NonNull<u8>, AllocError> {
    unsafe {
        // 익명 메모리(anonymous memory)를 프로세스 주소 공간에 직접 매핑하는 시스템 콜
        let ptr = libc::mmap(
            null_mut(),
            size,
            desired_mprotect(libc::PROT_READ | libc::PROT_WRITE),
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        );
        if ptr.is_null() || std::ptr::eq(ptr, libc::MAP_FAILED) {
            Err(AllocError)
        } else {
            Ok(NonNull::new_unchecked(ptr as *mut u8))
        }
    }
}

/// virtual memory 해제
pub unsafe fn virtual_release(base: NonNull<u8>, size: usize) {
    unsafe {
        libc::munmap(base.cast().as_ptr(), size);
    }
}

const fn desired_mprotect(_: c_int) -> c_int {
    libc::PROT_NONE
}

pub unsafe fn virtual_commit(base: NonNull<u8>, size: usize) -> Result<(), AllocError> {
    unsafe {
        let status = libc::mprotect(base.cast().as_ptr(), size, libc::PROT_READ | libc::PROT_WRITE);
        if status != 0 {
            Err(AllocError)
        } else {
            Ok(())
        }
    }
}

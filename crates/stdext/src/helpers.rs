
use std::{
    alloc::Allocator,
    mem,
    ops::{Bound, Range, RangeBounds},
    ptr,
};

/// 원시 포인터 타입으로 변환
#[inline(always)]
pub fn opt_ptr<T>(a: Option<&T>) -> *const T {
    unsafe { mem::transmute(a) }
}

/// 포인터 동일성 확인
/// match 문과 같은 분기없이 포인터를 직접 비교하는 성능이 나은 방법 사용
/// match (a, b) {
///     (Some(ax), Some(by)) => std::ptr::eq(ax, by), // 둘 다 Some인 경우 포인터 비교
///     (None, None) => true,                         // 둘 다 None인 경우 동일하다고 간주
///      _ => false,
#[inline(always)]
pub fn opt_ptr_eq<T>(a: Option<&T>, b: Option<&T>) -> bool {
    opt_ptr(a) == opt_ptr(b)
}

/// [`Vec::splice`] results in really bad assembly.
/// This doesn't. Don't use [`Vec::splice`].
pub trait ReplaceRange<T: Copy> {
    fn replace_range<R: RangeBounds<usize>>(&mut self, range: R, src: &[T]);
}

impl<T: Copy, A: Allocator> ReplaceRange<T> for Vec<T, A> {
    fn replace_range<R: RangeBounds<usize>>(&mut self, range: R, src: &[T]) {
        let start = match range.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(start) => start + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(end) => end + 1,
            Bound::Excluded(&end) => end,
            Bound::Unbounded => usize::MAX,
        };
        vec_replace_impl(self, start..end, src);
    }
}

fn vec_replace_impl<T: Copy, A: Allocator>(dst: &mut Vec<T, A>, range: Range<usize>, src: &[T]) {
    unsafe {
        let dst_len = dst.len();
        let src_len = src.len();
        let off = range.start.min(dst_len);
        let del_len = range.end.saturating_sub(off).min(dst_len - off);

        if del_len == 0 && src_len == 0 {
            return; // nothing to do
        }

        let tail_len = dst_len - off - del_len;
        let new_len = dst_len - del_len + src_len;

        if src_len > del_len {
            dst.reserve(src_len - del_len);
        }

        // NOTE: drop_in_place() is not needed here, because T is constrained to Copy.

        // SAFETY: as_mut_ptr() must called after reserve() to ensure that the pointer is valid.
        let ptr = dst.as_mut_ptr().add(off);

        // Shift the tail.
        if tail_len > 0 && src_len != del_len {
            ptr::copy(ptr.add(del_len), ptr.add(src_len), tail_len);
        }

        // Copy in the replacement.
        ptr::copy_nonoverlapping(src.as_ptr(), ptr, src_len);
        dst.set_len(new_len);
    }
}

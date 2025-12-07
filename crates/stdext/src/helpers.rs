
use std::mem;

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

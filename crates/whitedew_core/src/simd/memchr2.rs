
use std::ptr;

/// `memchr`, but with two needles.
///
/// Returns the index of the first occurrence of either needle in the
/// `haystack`. If no needle is found, `haystack.len()` is returned.
/// `offset` specifies the index to start searching from.
pub fn memchr2(needle1: u8, needle2: u8, haystack: &[u8], offset: usize) -> usize {
    unsafe {
        let beg = haystack.as_ptr();
        let end = beg.add(haystack.len());
        let it = beg.add(offset.min(haystack.len()));
        let it = memchr2_raw(needle1, needle2, it, end);
        it.offset_from_unsigned(beg)
    }
}

unsafe fn memchr2_raw(needle1: u8, needle2: u8, beg: *const u8, end: *const u8) -> *const u8 {
    // target_arch가 "aarch64"가 아니면 다른 코드 필요
    return unsafe { memchr2_neon(needle1, needle2, beg, end) };

    // #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "loongarch64"))]
    // return unsafe { MEMCHR2_DISPATCH(needle1, needle2, beg, end) };

    // #[allow(unreachable_code)]
    // return unsafe { memchr2_fallback(needle1, needle2, beg, end) };
}

unsafe fn memchr2_neon(needle1: u8, needle2: u8, mut beg: *const u8, end: *const u8) -> *const u8 {
    unsafe {
        use std::arch::aarch64::*;

        if end.offset_from_unsigned(beg) >= 16 {
            let n1 = vdupq_n_u8(needle1);
            let n2 = vdupq_n_u8(needle2);

            loop {
                let v = vld1q_u8(beg as *const _);
                let a = vceqq_u8(v, n1);
                let b = vceqq_u8(v, n2);
                let c = vorrq_u8(a, b);

                // https://community.arm.com/arm-community-blogs/b/servers-and-cloud-computing-blog/posts/porting-x86-vector-bitmask-optimizations-to-arm-neon
                let m = vreinterpretq_u16_u8(c);
                let m = vshrn_n_u16(m, 4);
                let m = vreinterpret_u64_u8(m);
                let m = vget_lane_u64(m, 0);

                if m != 0 {
                    return beg.add(m.trailing_zeros() as usize >> 2);
                }

                beg = beg.add(16);
                if end.offset_from_unsigned(beg) < 16 {
                    break;
                }
            }
        }

        memchr2_fallback(needle1, needle2, beg, end)
    }
}

unsafe fn memchr2_fallback(
    needle1: u8,
    needle2: u8,
    mut beg: *const u8,
    end: *const u8,
) -> *const u8 {
    unsafe {
        while !ptr::eq(beg, end) {
            let ch = *beg;
            if ch == needle1 || ch == needle2 {
                break;
            }
            beg = beg.add(1);
        }
        beg
    }
}

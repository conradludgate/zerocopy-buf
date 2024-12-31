//! Some polyfills for simple behaviour of [`bytes::Buf`]

use core::mem;

use bytes::Buf;

use crate::mu_polyfill;

/// Like [`Buf::copy_to_slice`] but supports uninit slices too.
pub fn copy_to_uninit_slice<'a>(
    this: &mut impl Buf,
    dst: &'a mut [mem::MaybeUninit<u8>],
) -> Option<&'a mut [u8]> {
    if this.remaining() < dst.len() {
        return None;
    }

    let mut c = &mut *dst;
    while !c.is_empty() {
        let src = this.chunk();

        let cnt = usize::min(src.len(), c.len());
        mu_polyfill::copy_from_slice(&mut c[..cnt], &src[..cnt]);

        c = &mut c[cnt..];
        this.advance(cnt);
    }

    // SAFETY: we have initilaised all of the bytes
    Some(unsafe { mu_polyfill::slice_assume_init_mut(dst) })
}

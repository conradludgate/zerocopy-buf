#![no_std]

use bytes::{Buf, BufMut};
use core::mem;
use zerocopy::{FromBytes, Immutable, IntoBytes};

mod mu_polyfill;

pub trait ZeroCopyReadBuf {
    fn try_read<T: FromBytes>(&mut self) -> Option<T>;
}

impl<B: Buf> ZeroCopyReadBuf for B {
    fn try_read<T: FromBytes>(&mut self) -> Option<T> {
        let mut t = mem::MaybeUninit::<T>::uninit();
        let bytes = copy_buf_to_uninit_slice(self, mu_polyfill::as_bytes_mut(&mut t))?;
        T::read_from_bytes(bytes).ok()
    }
}

pub trait ZeroCopyBufMut {
    fn write<T: IntoBytes + Immutable>(&mut self, t: T);
}

impl<B: BufMut> ZeroCopyBufMut for B {
    fn write<T: IntoBytes + Immutable>(&mut self, t: T) {
        self.put_slice(t.as_bytes());
    }
}

/// Like [`Buf::copy_to_slice`] but supports uninit slices too.
fn copy_buf_to_uninit_slice<'a>(
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

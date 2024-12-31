#![no_std]

use bytes::{Buf, BufMut, Bytes, BytesMut};
use core::{
    mem,
    ops::{Deref, DerefMut},
};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Ref, Unaligned};

extern crate alloc;

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

#[derive(Clone)]
pub struct ByteSlice<B>(pub B);

impl Deref for ByteSlice<Bytes> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Deref for ByteSlice<BytesMut> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ByteSlice<BytesMut> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// # Safety
/// We can reasonably assume that [`Bytes`] deref is stable.
/// Specifically, two consecutive calls to deref will not
/// produce different results.
unsafe impl zerocopy::ByteSlice for ByteSlice<Bytes> {}

/// # Safety
/// Cloning a [`Bytes`] is currently always stable. The clone operation
/// Might allocate a new Shared metadata,
/// but it never de-allocates the original bytes buffer.
unsafe impl zerocopy::CloneableByteSlice for ByteSlice<Bytes> {}

/// # Safety
/// We can reasonably assume that [`BytesMut`] deref is stable.
/// Specifically, two consecutive calls to deref will not
/// produce different results.
unsafe impl zerocopy::ByteSlice for ByteSlice<BytesMut> {}

/// # Safety
/// [`Bytes::split_to`] performs the required simple pointer arithmetic
unsafe impl zerocopy::SplitByteSlice for ByteSlice<Bytes> {
    unsafe fn split_at_unchecked(mut self, mid: usize) -> (Self, Self) {
        let lhs = self.0.split_to(mid);
        (Self(lhs), self)
    }
}

/// # Safety
/// [`BytesMut::split_to`] performs the required simple pointer arithmetic
unsafe impl zerocopy::SplitByteSlice for ByteSlice<BytesMut> {
    unsafe fn split_at_unchecked(mut self, mid: usize) -> (Self, Self) {
        let lhs = self.0.split_to(mid);
        (Self(lhs), self)
    }
}

pub trait ZeroCopyBuf {
    type Buf;
    fn try_get<T: KnownLayout + Immutable + Unaligned>(&mut self) -> Option<Ref<Self::Buf, T>>;
}

impl ZeroCopyBuf for Bytes {
    type Buf = ByteSlice<Bytes>;

    fn try_get<T: KnownLayout + Immutable + Unaligned>(&mut self) -> Option<Ref<Self::Buf, T>> {
        if self.remaining() < size_of::<T>() {
            return None;
        }
        let buf = ByteSlice(self.split_to(size_of::<T>()));
        Some(
            Ref::from_bytes(buf)
                .expect("size has been checked, and T is unaligned, so this should never panic"),
        )
    }
}

impl ZeroCopyBuf for BytesMut {
    type Buf = ByteSlice<BytesMut>;

    fn try_get<T: KnownLayout + Immutable + Unaligned>(&mut self) -> Option<Ref<Self::Buf, T>> {
        if self.remaining() < size_of::<T>() {
            return None;
        }
        let buf = ByteSlice(self.split_to(size_of::<T>()));
        Some(
            Ref::from_bytes(buf)
                .expect("size has been checked, and T is unaligned, so this should never panic"),
        )
    }
}

impl ZeroCopyBuf for &[u8] {
    type Buf = Self;

    fn try_get<T: KnownLayout + Immutable + Unaligned>(&mut self) -> Option<Ref<Self::Buf, T>> {
        let (a, b) = self.split_at_checked(size_of::<T>())?;
        *self = b;
        Some(
            Ref::from_bytes(a)
                .expect("size has been checked, and T is unaligned, so this should never panic"),
        )
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

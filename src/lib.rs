//! Extensions for [`bytes::Buf`] with compatibility with [`zerocopy`].

#![no_std]

use bytes::{Buf, BufMut, Bytes, BytesMut};
use core::{
    mem,
    ops::{Deref, DerefMut},
};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Ref, SizeError, Unaligned};

extern crate alloc;

mod buf_polyfill;
mod mu_polyfill;

/// A [`Buf`] that allows reading arbitrary [`zerocopy::FromBytes`] values from the buffer.
pub trait ZeroCopyReadBuf: Buf + Sized {
    /// Read a `T` from the [`Buf`].
    ///
    /// If [`Buf::remaining`] is greater than or equal to the size of `T`,
    /// then a T is returned and the buffer is advanced by the size of `T`.
    ///
    /// If [`Buf::remaining`] is less than the size of `T`, A [`SizeError`] is returned.
    ///
    /// This single method imitates all of the `Buf::get_...` methods.
    /// For example, [`Buf::get_u16`] could be written as:
    /// ```
    /// use zerocopy_buf::ZeroCopyReadBuf;
    ///
    /// let mut data: &[u8] = &b"\x01\x02"[..];
    /// let x = data.try_read::<zerocopy::network_endian::U16>().unwrap();
    /// assert_eq!(x.get(), 0x0102);
    /// ```
    fn try_read<T: FromBytes>(&mut self) -> Result<T, SizeError<(), T>>;
}

type Res<Buf, T> = Result<Ref<Buf, T>, SizeError<Buf, T>>;

/// A [`Buf`] that allows getting arbitrary values from the buffer.
pub trait ZeroCopyBuf: Buf {
    /// The buffer to borrow over. This is usually either `Self` or [`ByteSlice<Self>`]
    type Buf: zerocopy::ByteSlice;

    /// Get a ref to a `T` from the [`Buf`].
    ///
    /// If [`Buf::remaining`] is greater than or equal to the size of `T`,
    /// then a [`Ref<Self::Buf, T>`] is returned and the buffer is advanced by the size of `T`.
    ///
    /// If [`Buf::remaining`] is less than the size of `T`, A [`SizeError`] is returned.
    ///
    /// This single method imitates all of the `Buf::get_...` methods.
    /// For example, [`Buf::get_u16`] could be written as:
    /// ```
    /// use zerocopy_buf::ZeroCopyBuf;
    ///
    /// let mut data: &[u8] = &b"\x01\x02"[..];
    /// let x = data.try_get::<zerocopy::network_endian::U16>().unwrap();
    /// assert_eq!(x.get(), 0x0102);
    /// ```
    fn try_get<T: KnownLayout + Immutable + Unaligned>(&mut self) -> Res<Self::Buf, T>;

    /// Get a ref to a slice of `T` from the [`Buf`].
    ///
    /// If [`Buf::remaining`] is greater than or equal to the size of `T` * count,
    /// then a [`Ref<Self::Buf, T>`] is returned and the buffer is advanced by the size of `T`.
    ///
    /// If [`Buf::remaining`] is less, A [`SizeError`] is returned.
    ///
    /// ```
    /// use zerocopy_buf::ZeroCopyBuf;
    ///
    /// let mut data: &[u8] = &b"\x01\x02\x03\x04"[..];
    /// let x = data.try_get_elems::<zerocopy::network_endian::U16>(2).unwrap();
    /// assert_eq!(x.len(), 2);
    /// assert_eq!(x[0].get(), 0x0102);
    /// assert_eq!(x[1].get(), 0x0304);
    /// ```
    fn try_get_elems<T: KnownLayout + Immutable + Unaligned>(
        &mut self,
        count: usize,
    ) -> Res<Self::Buf, [T]>;
}

/// A [`BufMut`] that uses [`zerocopy::IntoBytes`] to encode
pub trait ZeroCopyBufMut: BufMut {
    /// Write a `T` to the [`BufMut`].
    ///
    /// This single method imitates all of the `BufMut::put_...` methods.
    /// For example, [`BufMut::put_u16`] could be written as:
    /// ```
    /// use zerocopy_buf::ZeroCopyBufMut;
    ///
    /// let mut data = bytes::BytesMut::new();
    /// data.write(zerocopy::network_endian::U16::new(0x0102));
    /// assert_eq!(&data, &b"\x01\x02"[..]);
    /// ```
    fn write<T: IntoBytes + Immutable>(&mut self, t: T);
}

impl<B: Buf> ZeroCopyReadBuf for B {
    fn try_read<T: FromBytes>(&mut self) -> Result<T, SizeError<(), T>> {
        let mut t = mem::MaybeUninit::<T>::uninit();
        let bytes = buf_polyfill::copy_to_uninit_slice(self, mu_polyfill::as_bytes_mut(&mut t))
            .unwrap_or_default();

        T::read_from_bytes(bytes).map_err(|e| e.map_src(|_| ()))
    }
}

impl ZeroCopyBuf for Bytes {
    type Buf = ByteSlice<Bytes>;

    fn try_get<T: KnownLayout + Immutable + Unaligned>(&mut self) -> Res<Self::Buf, T> {
        let buf = if self.remaining() < size_of::<T>() {
            // don't consume data, instead "parse" a dummy empty slice.
            ByteSlice(Bytes::new())
        } else {
            ByteSlice(self.split_to(size_of::<T>()))
        };
        Ref::from_bytes(buf).map_err(SizeError::from)
    }

    fn try_get_elems<T: KnownLayout + Immutable + Unaligned>(
        &mut self,
        count: usize,
    ) -> Res<Self::Buf, [T]> {
        let buf = match size_of::<T>().checked_mul(count) {
            Some(len) if self.remaining() >= len => ByteSlice(self.split_to(len)),
            _ => ByteSlice(Bytes::new()),
        };
        Ref::from_bytes_with_elems(buf, count).map_err(SizeError::from)
    }
}

impl ZeroCopyBuf for BytesMut {
    type Buf = ByteSlice<BytesMut>;

    fn try_get<T: KnownLayout + Immutable + Unaligned>(&mut self) -> Res<Self::Buf, T> {
        let buf = if self.remaining() < size_of::<T>() {
            // don't consume data, instead "parse" a dummy empty slice.
            ByteSlice(BytesMut::new())
        } else {
            ByteSlice(self.split_to(size_of::<T>()))
        };
        Ref::from_bytes(buf).map_err(SizeError::from)
    }

    fn try_get_elems<T: KnownLayout + Immutable + Unaligned>(
        &mut self,
        count: usize,
    ) -> Res<Self::Buf, [T]> {
        let buf = match size_of::<T>().checked_mul(count) {
            Some(len) if self.remaining() >= len => ByteSlice(self.split_to(len)),
            _ => ByteSlice(BytesMut::new()),
        };
        Ref::from_bytes_with_elems(buf, count).map_err(SizeError::from)
    }
}

impl ZeroCopyBuf for &[u8] {
    type Buf = Self;

    fn try_get<T: KnownLayout + Immutable + Unaligned>(&mut self) -> Res<Self::Buf, T> {
        let buf = if size_of::<T>() <= self.len() {
            let (a, b) = self.split_at(size_of::<T>());
            *self = b;
            a
        } else {
            // don't consume data, instead "parse" a dummy empty slice.
            &[]
        };
        Ref::from_bytes(buf).map_err(SizeError::from)
    }

    fn try_get_elems<T: KnownLayout + Immutable + Unaligned>(
        &mut self,
        count: usize,
    ) -> Res<Self::Buf, [T]> {
        let buf = match size_of::<T>().checked_mul(count) {
            Some(len) if self.remaining() >= len => {
                let (a, b) = self.split_at(len);
                *self = b;
                a
            }
            _ => &[],
        };
        Ref::from_bytes_with_elems(buf, count).map_err(SizeError::from)
    }
}

impl<B: BufMut> ZeroCopyBufMut for B {
    fn write<T: IntoBytes + Immutable>(&mut self, t: T) {
        self.put_slice(t.as_bytes());
    }
}

/// A wrapper to implement [`zerocopy::ByteSlice`] on [`bytes`] types.
#[derive(Clone)]
#[repr(transparent)]
pub struct ByteSlice<B>(B);

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

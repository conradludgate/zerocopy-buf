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

    /// Get a ref to a DST `T` from the [`Buf`].
    ///
    /// If [`Buf::remaining`] is greater than or equal to the size of `T` with `count` elements,
    /// then a [`Ref<Self::Buf, T>`] is returned and the buffer is advanced by the size of `T`.
    ///
    /// If [`Buf::remaining`] is less, A [`SizeError`] is returned.
    ///
    /// ```
    /// use zerocopy_buf::ZeroCopyBuf;
    ///
    /// let mut data: &[u8] = &b"\x01\x02\x03\x04"[..];
    /// let x = data.try_get_elems::<[zerocopy::network_endian::U16]>(2).unwrap();
    /// assert_eq!(x.len(), 2);
    /// assert_eq!(x[0].get(), 0x0102);
    /// assert_eq!(x[1].get(), 0x0304);
    /// ```
    fn try_get_elems<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
        count: usize,
    ) -> Res<Self::Buf, T>;

    /// Get a DST `T` from the largest matching prefix of the [`Buf`].
    ///
    /// The length of `T` is inferred from the available bytes. The buffer is
    /// advanced by the size of the returned value; any bytes that do not fit
    /// remain in the buffer.
    fn try_get_prefix<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
    ) -> Res<Self::Buf, T>;

    /// Get a ref to a `T` from the [`Buf`].
    ///
    /// If [`Buf::remaining`] is greater than or equal to the size of `T`,
    /// then a [`Ref<Self::Buf, T>`] is returned and the buffer is **NOT** advanced by the size of `T`.
    ///
    /// If [`Buf::remaining`] is less than the size of `T`, A [`SizeError`] is returned.
    ///
    /// ```
    /// use zerocopy_buf::ZeroCopyBuf;
    /// use zerocopy::{FromBytes, KnownLayout, Immutable, Unaligned};
    ///
    /// #[derive(FromBytes, KnownLayout, Immutable, Unaligned)]
    /// #[repr(C)]
    /// struct PacketHeader {
    ///     len: zerocopy::network_endian::U32,
    /// }
    ///
    /// #[derive(FromBytes, KnownLayout, Immutable, Unaligned)]
    /// #[repr(C)]
    /// struct Packet {
    ///     header: PacketHeader,
    ///     body: [u8],
    /// }
    ///
    /// let mut data: &[u8] = &b"\x00\x00\x00\x0bhello world"[..];
    /// let header = data.try_peek::<PacketHeader>().unwrap();
    /// let payload_len = header.len.get();
    /// assert_eq!(payload_len, 11);
    ///
    /// let packet = data.try_get_elems::<Packet>(payload_len as usize).unwrap();
    /// assert_eq!(packet.body, b"hello world"[..]);
    /// ```
    fn try_peek<T: KnownLayout + Immutable + Unaligned>(&mut self) -> Res<&[u8], T>;

    /// Get a ref to a DST `T` from the [`Buf`].
    ///
    /// If [`Buf::remaining`] is greater than or equal to the size of `T` with `count` elements,
    /// then a [`Ref<Self::Buf, T>`] is returned and the buffer is **NOT** advanced by the size of `T`.
    ///
    /// If [`Buf::remaining`] is less, A [`SizeError`] is returned.
    fn try_peek_elems<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
        count: usize,
    ) -> Res<&[u8], T>;

    /// Get a DST `T` from the largest matching prefix of the [`Buf`] without
    /// advancing it.
    ///
    /// The length of `T` is inferred from the available bytes. Any bytes that
    /// do not fit remain outside the returned reference.
    fn try_peek_prefix<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
    ) -> Res<&[u8], T>;
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
    /// data.write(&zerocopy::network_endian::U16::new(0x0102));
    /// assert_eq!(&data, &b"\x01\x02"[..]);
    /// ```
    fn write<T: IntoBytes + Immutable + ?Sized>(&mut self, t: &T);
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
        let (a, b) = Ref::from_prefix(ByteSlice(mem::take(self)))
            .map_err(SizeError::from)
            .map_err(|e| e.map_src(|s| ByteSlice(mem::replace(self, s.0))))?;
        *self = b.0;
        Ok(a)
    }

    fn try_get_elems<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
        count: usize,
    ) -> Res<Self::Buf, T> {
        let (a, b) = Ref::from_prefix_with_elems(ByteSlice(mem::take(self)), count)
            .map_err(SizeError::from)
            .map_err(|e| e.map_src(|s| ByteSlice(mem::replace(self, s.0))))?;
        *self = b.0;
        Ok(a)
    }

    fn try_get_prefix<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
    ) -> Res<Self::Buf, T> {
        let (a, b) = Ref::from_prefix(ByteSlice(mem::take(self)))
            .map_err(SizeError::from)
            .map_err(|e| e.map_src(|s| ByteSlice(mem::replace(self, s.0))))?;
        *self = b.0;
        Ok(a)
    }

    fn try_peek<T: KnownLayout + Immutable + Unaligned>(&mut self) -> Res<&[u8], T> {
        let (a, _) = Ref::from_prefix(&**self).map_err(SizeError::from)?;
        Ok(a)
    }

    fn try_peek_elems<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
        count: usize,
    ) -> Res<&[u8], T> {
        let (a, _) = Ref::from_prefix_with_elems(&**self, count).map_err(SizeError::from)?;
        Ok(a)
    }

    fn try_peek_prefix<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
    ) -> Res<&[u8], T> {
        let (a, _) = Ref::from_prefix(&**self).map_err(SizeError::from)?;
        Ok(a)
    }
}

impl ZeroCopyBuf for BytesMut {
    type Buf = ByteSlice<BytesMut>;

    fn try_get<T: KnownLayout + Immutable + Unaligned>(&mut self) -> Res<Self::Buf, T> {
        let (a, b) = Ref::from_prefix(ByteSlice(mem::take(self)))
            .map_err(SizeError::from)
            .map_err(|e| e.map_src(|s| ByteSlice(mem::replace(self, s.0))))?;
        *self = b.0;
        Ok(a)
    }

    fn try_get_elems<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
        count: usize,
    ) -> Res<Self::Buf, T> {
        let (a, b) = Ref::from_prefix_with_elems(ByteSlice(mem::take(self)), count)
            .map_err(SizeError::from)
            .map_err(|e| e.map_src(|s| ByteSlice(mem::replace(self, s.0))))?;
        *self = b.0;
        Ok(a)
    }

    fn try_get_prefix<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
    ) -> Res<Self::Buf, T> {
        let (a, b) = Ref::from_prefix(ByteSlice(mem::take(self)))
            .map_err(SizeError::from)
            .map_err(|e| e.map_src(|s| ByteSlice(mem::replace(self, s.0))))?;
        *self = b.0;
        Ok(a)
    }

    fn try_peek<T: KnownLayout + Immutable + Unaligned>(&mut self) -> Res<&[u8], T> {
        let (a, _) = Ref::from_prefix(&**self).map_err(SizeError::from)?;
        Ok(a)
    }

    fn try_peek_elems<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
        count: usize,
    ) -> Res<&[u8], T> {
        let (a, _) = Ref::from_prefix_with_elems(&**self, count).map_err(SizeError::from)?;
        Ok(a)
    }

    fn try_peek_prefix<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
    ) -> Res<&[u8], T> {
        let (a, _) = Ref::from_prefix(&**self).map_err(SizeError::from)?;
        Ok(a)
    }
}

impl ZeroCopyBuf for &[u8] {
    type Buf = Self;

    fn try_get<T: KnownLayout + Immutable + Unaligned>(&mut self) -> Res<Self::Buf, T> {
        let (a, b) = Ref::from_prefix(*self).map_err(SizeError::from)?;
        *self = b;
        Ok(a)
    }

    fn try_get_elems<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
        count: usize,
    ) -> Res<Self::Buf, T> {
        let (a, b) = Ref::from_prefix_with_elems(*self, count).map_err(SizeError::from)?;
        *self = b;
        Ok(a)
    }

    fn try_get_prefix<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
    ) -> Res<Self::Buf, T> {
        let (a, b) = Ref::from_prefix(*self).map_err(SizeError::from)?;
        *self = b;
        Ok(a)
    }

    fn try_peek<T: KnownLayout + Immutable + Unaligned>(&mut self) -> Res<&[u8], T> {
        let (a, _) = Ref::from_prefix(*self).map_err(SizeError::from)?;
        Ok(a)
    }

    fn try_peek_elems<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
        count: usize,
    ) -> Res<&[u8], T> {
        let (a, _) = Ref::from_prefix_with_elems(*self, count).map_err(SizeError::from)?;
        Ok(a)
    }

    fn try_peek_prefix<T: KnownLayout<PointerMetadata = usize> + Immutable + Unaligned + ?Sized>(
        &mut self,
    ) -> Res<&[u8], T> {
        let (a, _) = Ref::from_prefix(*self).map_err(SizeError::from)?;
        Ok(a)
    }
}

impl<B: BufMut> ZeroCopyBufMut for B {
    fn write<T: IntoBytes + Immutable + ?Sized>(&mut self, t: &T) {
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
/// The `ByteSlice` contract requires two dereferences of an unchanged value to
/// return slices with the "same address and length" [1]. `Bytes` documents
/// itself as a "chunk of contiguous memory" and says clones share its
/// underlying memory [2]. Its view is immutable and changes only through
/// explicit split/consuming operations, so the wrapper's dereference is stable.
///
/// [1] https://docs.rs/zerocopy/0.8.9/zerocopy/trait.ByteSlice.html#safety
/// [2] https://docs.rs/bytes/1.0.0/bytes/struct.Bytes.html
unsafe impl zerocopy::ByteSlice for ByteSlice<Bytes> {}

/// # Safety
/// The `CloneableByteSlice` contract requires dereference stability to hold
/// across cloning [1]. `Bytes` documents that clones share the same underlying
/// memory [2], while retaining their view metadata, so cloning preserves the
/// byte view.
///
/// [1] https://docs.rs/zerocopy/0.8.9/zerocopy/trait.CloneableByteSlice.html#safety
/// [2] https://docs.rs/bytes/1.0.0/bytes/struct.Bytes.html
unsafe impl zerocopy::CloneableByteSlice for ByteSlice<Bytes> {}

/// # Safety
/// The `ByteSlice` contract requires two dereferences of an unchanged value to
/// return slices with the "same address and length" [1]. This wrapper owns its
/// `BytesMut`; its dereference methods return slices of the current view, and
/// safe mutable slice access cannot change the slice length or reallocate it.
/// The view changes only through the explicit split implementation below.
///
/// [1] https://docs.rs/zerocopy/0.8.9/zerocopy/trait.ByteSlice.html#safety
unsafe impl zerocopy::ByteSlice for ByteSlice<BytesMut> {}

/// # Safety
/// SAFETY:
/// - The caller must uphold `SplitByteSlice::split_at_unchecked`'s requirement
///   that `mid <= self.len()`.
/// - `Bytes::split_to` returns bytes `[0, mid)` and leaves bytes `[mid, len)`
///   in `self`, so the returned wrappers exactly partition the original view.
///
/// The `Bytes` API documents that it "contains elements `[at, len)`" in self
/// and returns `[0, at)` [1].
///
/// [1] https://docs.rs/bytes/1.0.0/bytes/struct.Bytes.html#method.split_to
unsafe impl zerocopy::SplitByteSlice for ByteSlice<Bytes> {
    /// # Safety
    /// `mid` must satisfy the trait contract: "mid must not be greater than
    /// self.deref().len()".
    /// https://docs.rs/zerocopy/0.8.9/zerocopy/trait.SplitByteSlice.html#method.split_at_unchecked
    unsafe fn split_at_unchecked(mut self, mid: usize) -> (Self, Self) {
        // SAFETY: the trait's caller contract guarantees `mid <= self.len()`;
        // Bytes::split_to then produces exactly the prefix and suffix ranges
        // described below.
        let lhs = self.0.split_to(mid);
        (Self(lhs), self)
    }
}

/// # Safety
/// SAFETY:
/// - The caller must uphold `SplitByteSlice::split_at_unchecked`'s requirement
///   that `mid <= self.len()`.
/// - `BytesMut::split_to` returns bytes `[0, mid)` and leaves bytes `[mid, len)`
///   in `self`, so the returned wrappers exactly partition the original view.
///
/// The `BytesMut` API documents that it "contains elements `[at, len)`" in
/// self and returns `[0, at)` [1].
///
/// [1] https://docs.rs/bytes/1.0.0/bytes/struct.BytesMut.html#method.split_to
unsafe impl zerocopy::SplitByteSlice for ByteSlice<BytesMut> {
    /// # Safety
    /// `mid` must satisfy the trait contract: "mid must not be greater than
    /// self.deref().len()".
    /// https://docs.rs/zerocopy/0.8.9/zerocopy/trait.SplitByteSlice.html#method.split_at_unchecked
    unsafe fn split_at_unchecked(mut self, mid: usize) -> (Self, Self) {
        // SAFETY: the trait's caller contract guarantees `mid <= self.len()`;
        // BytesMut::split_to then produces exactly the prefix and suffix ranges
        // described below.
        let lhs = self.0.split_to(mid);
        (Self(lhs), self)
    }
}

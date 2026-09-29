use core::{
    mem::{self, MaybeUninit},
    slice,
};

/// Same as [`MaybeUninit::slice_assume_init_mut`]
///
/// # Safety
///
/// Every element in `slice` must be initialized to a valid value of `T`.
pub const unsafe fn slice_assume_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    // SAFETY:
    // - MaybeUninit<T> and T have the same size and alignment [1], and the
    //   Reference says an unsized-to-unsized pointer cast preserves metadata [2].
    // - The caller guarantees each element is initialized and valid as T; the
    //   exclusive input borrow remains exclusive for the returned slice [3].
    //
    // [1] "same size, alignment, and ABI as T":
    // https://doc.rust-lang.org/1.91.0/std/mem/union.MaybeUninit.html#layout
    // [2] "the metadata is preserved exactly":
    // https://doc.rust-lang.org/1.91.0/reference/expressions/operator-expr.html#pointer-to-pointer-cast
    // [3] The slice must contain "properly initialized values of type T":
    // https://doc.rust-lang.org/1.91.0/std/slice/fn.from_raw_parts_mut.html#safety
    unsafe { &mut *(slice as *mut [MaybeUninit<T>] as *mut [T]) }
}

/// Same as [`MaybeUninit::copy_from_slice`]
pub fn copy_from_slice<'a, T>(this: &'a mut [MaybeUninit<T>], src: &[T]) -> &'a mut [T]
where
    T: Copy,
{
    assert_eq!(this.len(), src.len());
    for (dst, src) in this.iter_mut().zip(src) {
        dst.write(*src);
    }

    // SAFETY: Every destination element was initialized with a valid, copied T
    // immediately above, and the exclusive borrow is preserved [1].
    //
    // [1] `assume_init_mut` requires the contents to be "fully initialized":
    // https://doc.rust-lang.org/1.91.0/std/mem/union.MaybeUninit.html#method.assume_init_mut
    unsafe { slice_assume_init_mut(this) }
}

/// Same as [`MaybeUninit::as_bytes_mut`]
pub fn as_bytes_mut<T>(this: &mut MaybeUninit<T>) -> &mut [MaybeUninit<u8>] {
    // SAFETY:
    // - MaybeUninit<T> has the same size and alignment as T [1]; casting its
    //   data pointer to MaybeUninit<u8> preserves the pointer address [2].
    // - u8 has alignment 1, and `this` provides exclusive access to this
    //   allocation for exactly size_of::<T>() bytes [3].
    // - MaybeUninit<u8> accepts every initialized or uninitialized byte [4].
    //
    // [1] "same size, alignment, and ABI as T":
    // https://doc.rust-lang.org/1.91.0/std/mem/union.MaybeUninit.html#layout
    // [2] A sized-to-sized raw pointer cast returns the pointer unchanged:
    // https://doc.rust-lang.org/1.91.0/reference/expressions/operator-expr.html#pointer-to-pointer-cast
    // [3] Mutable slices require aligned, valid storage:
    // https://doc.rust-lang.org/1.91.0/std/slice/fn.from_raw_parts_mut.html#safety
    // [4] MaybeUninit has "no validity requirements" for stored bytes:
    // https://doc.rust-lang.org/1.91.0/std/mem/union.MaybeUninit.html#validity
    unsafe {
        slice::from_raw_parts_mut(
            this.as_mut_ptr() as *mut MaybeUninit<u8>,
            mem::size_of::<T>(),
        )
    }
}

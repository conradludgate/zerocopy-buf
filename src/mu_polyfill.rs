use core::{
    mem::{self, MaybeUninit},
    slice,
};

/// Same as [`MaybeUninit::slice_assume_init_mut`]
pub const unsafe fn slice_assume_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    // SAFETY: similar to safety notes for `slice_get_ref`, but we have a
    // mutable reference which is also guaranteed to be valid for writes.
    unsafe { &mut *(slice as *mut [MaybeUninit<T>] as *mut [T]) }
}

/// Same as [`MaybeUninit::copy_from_slice`]
pub fn copy_from_slice<'a, T>(this: &'a mut [MaybeUninit<T>], src: &[T]) -> &'a mut [T]
where
    T: Copy,
{
    // SAFETY: &[T] and &[MaybeUninit<T>] have the same layout
    let uninit_src: &[MaybeUninit<T>] = unsafe { core::mem::transmute(src) };

    this.copy_from_slice(uninit_src);

    // SAFETY: Valid elements have just been copied into `this` so it is initialized
    unsafe { slice_assume_init_mut(this) }
}

/// Same as [`MaybeUninit::as_bytes_mut`]
pub fn as_bytes_mut<T>(this: &mut MaybeUninit<T>) -> &mut [MaybeUninit<u8>] {
    // SAFETY: MaybeUninit<u8> is always valid, even for padding bytes
    unsafe {
        slice::from_raw_parts_mut(
            this.as_mut_ptr() as *mut MaybeUninit<u8>,
            mem::size_of::<T>(),
        )
    }
}

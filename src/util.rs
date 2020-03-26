pub(crate) trait WrappingOffsetFromExt<T: Sized> {
    fn wrapping_offset_from_(self, origin: *const T) -> isize;
}

impl<T: Sized> WrappingOffsetFromExt<T> for *const T {
    #[inline]
    fn wrapping_offset_from_(self, origin: *const T) -> isize {
        let pointee_size = std::mem::size_of::<T>();
        assert!(0 < pointee_size && pointee_size <= isize::max_value() as usize);

        let d = isize::wrapping_sub(self as _, origin as _);
        d.wrapping_div(pointee_size as _)
    }
}

impl<T: Sized> WrappingOffsetFromExt<T> for *mut T {
    fn wrapping_offset_from_(self, origin: *const T) -> isize {
        (self as *const T).wrapping_offset_from_(origin)
    }
}

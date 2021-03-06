# Appendix: Being compatible with `free()`

```rust,noplaypen
use ::safer_ffi::{prelude::*, ptr};

/// A `Box`-like owned pointer type, but which can be freed using `free()`.
#[derive_ReprC]
#[repr(transparent)]
pub struct Malloc<T>(ptr::NonNullOwned<T>);

impl<T> Malloc<T> {
    pub
    fn new (value: T)
      -> Option<Malloc<T>>
    {
        let (size, align) = (
            ::core::mem::size_of::<T>(),
            ::core::mem::align_of::<T>(),
        );
        if size == 0 { return None; }
        let mut ptr = ptr::null_mut();
        unsafe {
            ::libc::posix_memalign(&mut ptr, align, size);
        }
        let mut non_null: ptr::NonNull<T> = ptr::NonNull::new(ptr)?.cast();
        unsafe {
            non_null.as_ptr().write(value);
        }
        Some(Malloc(non_null.into()))
    }
}
```

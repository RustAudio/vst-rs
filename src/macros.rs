//! Internal utility macros

/// Implements `From` and `Into` for enums with `#[repr(usize)]`. Useful for interfacing with C
/// enums.
#[macro_export]
macro_rules! impl_clike {
    ($t:ty, $($c:ty) +) => {
        $(
            impl From<$c> for $t {
                fn from(v: $c) -> $t {
                    use std::mem;
                    unsafe { mem::transmute(v as usize) }
                }
            }

            impl Into<$c> for $t {
                fn into(self) -> $c {
                    self as $c
                }
            }
        )*
    };

    ($t:ty) => {
        impl_clike!($t, i8 i16 i32 i64 isize u8 u16 u32 u64 usize);
    }
}

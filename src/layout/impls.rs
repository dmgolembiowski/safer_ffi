use super::*;

const _: () = { use fmt::Write; macro_rules! impl_CTypes {
    () => (
        impl_CTypes! { @pointers }
        impl_CTypes! { @zsts }
        impl_CTypes! { @integers
            unsafe // Safety: trivial integer equivalence.
            u8 => "uint8_t",

            unsafe // Safety: trivial integer equivalence.
            u16 => "uint16_t",

            unsafe // Safety: trivial integer equivalence.
            u32 => "uint32_t",

            unsafe // Safety: trivial integer equivalence.
            u64 => "uint64_t",

            // unsafe u128 => "uint128_t",

            unsafe // Safety: Contrary to what most people think,
                   // `usize` is not a `size_t` but an `uintptr_t`,
                   // since it has a guaranteed non-`unsafe` transmute (`as`)
                   // with pointers.
            usize => "uintptr_t",


            unsafe // Safety: trivial integer equivalence.
            i8 => "int8_t",

            unsafe // Safety: trivial integer equivalence.
            i16 => "int16_t",

            unsafe // Safety: trivial integer equivalence.
            i32 => "int32_t",

            unsafe // Safety: trivial integer equivalence.
            i64 => "int64_t",

            // unsafe i128 => "int128_t",

            unsafe // Safety: Contrary to what most people think,
                   // `isize` is not a `ssize_t` but an `intptr_t`,
                   // since it has a guaranteed non-`unsafe` transmute (`as`)
                   // with pointers.
            isize => "intptr_t",
        }
        impl_CTypes! { @fns
            (A6, A5, A4, A3, A2, A1)
        }
        impl_CTypes! { @arrays
            // 0
            1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25
            26 27 28 29 30 31 32 40 48 50 60 64 70 75 80 90 96 100 125 128 192
            200 250 256 300 400 500 512 600 700 750 800 900 1000 1024
        }
    );

    (
        @arrays
        $($N:tt)*
    ) => ($(
        // CType
        unsafe // Safety: Rust arrays _are_ `#[repr(C)]`
        impl<Item : CType> CType for [Item; $N] {
            #[cfg(feature = "headers")]
            fn with_short_name<R> (ret: impl FnOnce(&'_ dyn fmt::Display) -> R)
              -> R
            {
                // item_t_N_array
                Item::with_short_name(|item_t| ret(&
                    format_args!(
                        concat!("{}_", stringify!($N), "_array"),
                        item_t,
                    )
                ))
            }

            #[cfg(feature = "headers")]
            fn c_define_self (definer: &'_ mut dyn Definer)
              -> io::Result<()>
            {
                let ref mut buf = [0_u8; 256];
                Self::with_short_name(|short_name| {
                    use ::std::io::Write;
                    write!(&mut buf[..], "{}", short_name)
                        .expect("`short_name()` was too long")
                });
                let short_name = ::core::str::from_utf8(buf).unwrap();
                definer.define(
                    short_name,
                    &mut |definer| {
                        Item::c_define_self(definer)?;
                        write!(definer.out(),
                            concat!(
                                "typedef struct {{ {}[",
                                stringify!($N),
                                "]; }} {};\n\n",
                            ),
                            Item::c_display("idx"),
                            short_name,
                        )
                    }
                )
            }

            #[cfg(feature = "headers")]
            fn c_fmt (
                fmt: &'_ mut fmt::Formatter<'_>,
                var_name: &'_ str,
            ) -> fmt::Result
            {
                // _e.g._, item_t_N_array
                Self::with_short_name(|short_name| {
                    write!(fmt,
                        "{}{sep}{}", short_name, var_name,
                        sep = if var_name.is_empty() { "" } else { " " },
                    )
                })
            }
        }

        // ReprC
        unsafe
        impl<Item : ReprC> ReprC for [Item; $N] {
            type CLayout = [Item::CLayout; $N];

            #[inline]
            fn is_valid (it: &'_ Self::CLayout)
              -> bool
            {
                it.iter().all(Item::is_valid)
            }
        }
    )*);

    (@fns
        (
            $(
                $An:ident $(,
                $Ai:ident)* $(,)?
            )?
        )
    ) => (
        // recurse
        $(
            impl_CTypes! {
                @fns
                ($($Ai ,)*)
            }
        )?

        // CType
        unsafe // Safety: this is the "blessed" type recommended across Rust
               // literature. Still the alignment of function pointers is not
               // as well-defined, as one would wish.
        impl<
            Ret : CType, $(
            $An : CType, $(
            $Ai : CType,
        )*)?> CType
            for Option<unsafe extern "C" fn ($($An, $($Ai ,)*)?) -> Ret>
        {
            #[cfg(feature = "headers")]
            fn with_short_name<R> (ret: impl FnOnce(&'_ dyn fmt::Display) -> R)
              -> R
            {
                ret(&{
                    // ret_t_arg1_t_arg2_t_fptr
                    let mut ret = Ret::with_short_name(|it| it.to_string()); $(
                    $An::with_short_name(|it| write!(&mut ret, "_{}", it))
                        .unwrap()
                    ; $(
                    $Ai::with_short_name(|it| write!(&mut ret, "_{}", it))
                        .unwrap()
                    ; )*)?
                    ret.push_str("_fptr");
                    ret
                })
            }

            #[cfg(feature = "headers")]
            fn c_define_self (definer: &'_ mut Definer)
              -> io::Result<()>
            {
                Ret::c_define_self(definer)?; $(
                $An::c_define_self(definer)?; $(
                $Ai::c_define_self(definer)?; )*)?
                Ok(())
            }

            #[cfg(feature = "headers")]
            fn c_fmt (
                fmt: &'_ mut fmt::Formatter<'_>,
                var_name: &'_ str,
            ) -> fmt::Result
            {
                write!(fmt, "{} ", Ret::c_display(""))?;
                write!(fmt, "(*{})(", var_name)?; $(
                write!(fmt, "{}", $An::c_display(""))?; $(
                write!(fmt, ", {}", $Ai::c_display(""))?; )*)?
                fmt.write_str(")")
            }
        }

        unsafe // Safety: byte-wise the layout is the same, but the safety
               // invariants will still have to be checked at each site.
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> ReprC
            for Option<unsafe extern "C" fn ($($An, $($Ai ,)*)?) -> Ret>
        {
            type CLayout = Option<
                unsafe extern "C"
                fn ($($An::CLayout, $($Ai::CLayout ,)*)?) -> Ret::CLayout
            >;

            #[inline]
            fn is_valid (c_layout: &'_ Self::CLayout)
              -> bool
            {
                true
            }
        }

        unsafe // Safety: byte-wise the layout is the same, but the safety
               // invariants will still have to be checked at each site.
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> ReprC
            for Option</*unsafe*/ extern "C" fn ($($An, $($Ai ,)*)?) -> Ret>
        {
            type CLayout = Option<
                unsafe extern "C"
                fn ($($An::CLayout, $($Ai::CLayout ,)*)?) -> Ret::CLayout
            >;

            #[inline]
            fn is_valid (c_layout: &'_ Self::CLayout)
              -> bool
            {
                true
            }
        }

        /* == ReprC for Option-less == */
        unsafe // Safety: byte-wise the layout is the same, but the safety
               // invariants will still have to be checked at each site.
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> ReprC
            for unsafe extern "C" fn ($($An, $($Ai ,)*)?) -> Ret
        {
            type CLayout = Option<
                unsafe extern "C"
                fn ($($An::CLayout, $($Ai::CLayout ,)*)?) -> Ret::CLayout
            >;

            #[inline]
            fn is_valid (c_layout: &'_ Self::CLayout)
              -> bool
            {
                c_layout.is_some()
            }
        }

        unsafe // Safety: byte-wise the layout is the same, but the safety
               // invariants will still have to be checked at each site.
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> ReprC
            for /*unsafe*/ extern "C" fn ($($An, $($Ai ,)*)?) -> Ret
        {
            type CLayout = Option<
                unsafe extern "C"
                fn ($($An::CLayout, $($Ai::CLayout ,)*)?) -> Ret::CLayout
            >;

            #[inline]
            fn is_valid (c_layout: &'_ Self::CLayout)
              -> bool
            {
                c_layout.is_some()
            }
        }
    );

    (@integers
        $(
            $unsafe:tt
            $RustInt:ident => $CInt:literal,
        )*
    ) => ($(
        $unsafe // Safety: guaranteed by the caller of the macro
        impl CType for $RustInt {
            #[cfg(feature = "headers")]
            fn with_short_name<R> (ret: impl FnOnce(&'_ dyn fmt::Display) -> R)
              -> R
            {
                ret(&$CInt)
            }

            #[cfg(feature = "headers")]
            fn c_define_self (definer: &'_ mut dyn Definer)
              -> io::Result<()>
            {
                definer.define(
                    "<stdint.h>",
                    &mut |definer| write!(definer.out(),
                        "\n#include <stdint.h>\n\n",
                    ),
                )
            }

            #[cfg(feature = "headers")]
            fn c_fmt (
                fmt: &'_ mut fmt::Formatter<'_>,
                var_name: &'_ str,
            ) -> fmt::Result
            {
                write!(fmt,
                    concat!($CInt, "{sep}{}"),
                    var_name,
                    sep = if var_name.is_empty() { "" } else { " " },
                )
            }
        }
        from_CType_impl_ReprC! { $RustInt }
    )*);

    (
        @pointers
    ) => (
        unsafe
        impl<T : CType> CType for *const T {
            #[cfg(feature = "headers")]
            fn with_short_name<R> (ret: impl FnOnce(&'_ dyn fmt::Display) -> R)
              -> R
            {
                T::with_short_name(|it| {
                    ret(&format_args!("{}_const_ptr", it))
                })
            }

            #[cfg(feature = "headers")]
            fn c_define_self (definer: &'_ mut dyn Definer)
              -> io::Result<()>
            {
                T::c_define_self(definer)
            }

            #[cfg(feature = "headers")]
            fn c_fmt (
                fmt: &'_ mut fmt::Formatter<'_>,
                var_name: &'_ str,
            ) -> fmt::Result
            {
                write!(fmt,
                    "{} const *{sep}{}",
                    T::c_display(""),
                    var_name,
                    sep = if var_name.is_empty() { "" } else { " " },
                )
            }
        }
        unsafe
        impl<T : ReprC> ReprC for *const T {
            type CLayout = *const T::CLayout;

            #[inline]
            fn is_valid (c_layout: &'_ Self::CLayout)
              -> bool
            {
                true
            }
        }

        unsafe
        impl<T : CType> CType for *mut T {
            #[cfg(feature = "headers")]
            fn with_short_name<R> (ret: impl FnOnce(&'_ dyn fmt::Display) -> R)
              -> R
            {
                T::with_short_name(|it| {
                    ret(&format_args!("{}_mut_ptr", it))
                })
            }

            #[cfg(feature = "headers")]
            fn c_define_self (definer: &'_ mut dyn Definer)
              -> io::Result<()>
            {
                T::c_define_self(definer)
            }

            #[cfg(feature = "headers")]
            fn c_fmt (
                fmt: &'_ mut fmt::Formatter<'_>,
                var_name: &'_ str,
            ) -> fmt::Result
            {
                write!(fmt,
                    "{} *{sep}{}",
                    T::c_display(""),
                    var_name,
                    sep = if var_name.is_empty() { "" } else { " " },
                )
            }
        }
        unsafe
        impl<T : ReprC> ReprC for *mut T {
            type CLayout = *mut T::CLayout;

            #[inline]
            fn is_valid (c_layout: &'_ Self::CLayout)
              -> bool
            {
                true
            }
        }
    );

    (
        @zsts
    ) => (
        unsafe
        impl ReprC for () {
            type CLayout = Void;

            #[inline]
            fn is_valid (it: &'_ Void)
              -> bool
            {
                true
            }
        }
    );
} impl_CTypes! {} };

macro_rules! impl_ReprC_for {(
    $unsafe:tt {
        $(
            $(@for [$($generics:tt)+])? $T:ty
                => |ref $it:tt : $Layout:ty| $expr:expr
        ),* $(,)?
    }
) => (
    $(
        $unsafe
        impl $(<$($generics)+>)? ReprC
            for $T
        {
            type CLayout = $Layout;

            #[inline]
            fn is_valid (it: &'_ $Layout)
              -> bool
            {
                let $it = it;
                if $expr {
                    true
                } else {
                    #[cfg(feature = "std")]
                    eprintln!(
                        "Error: {:#x?} is not a _valid_ bit pattern for the type `{}`",
                        unsafe {
                            ::core::slice::from_raw_parts(
                                <*const _>::cast::<u8>(it),
                                ::core::mem::size_of_val(it),
                            )
                        },
                        stringify!($T),
                    );
                    false
                }
            }
        }
    )*
)}

impl_ReprC_for! { unsafe {
    bool
        => |ref byte: u8| (*byte & !0b1) == 0
    ,

    @for[T : ReprC]
    ::core::ptr::NonNull<T>
        => |ref it: *mut T::CLayout| it.is_null().not()
    ,
    @for['a, T : 'a + ReprC]
    &'a T
        => |ref it: *const T::CLayout| {
            it.is_null().not() &&
            (*it as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
    @for['a, T : 'a + ReprC]
    &'a mut T
        => |ref it: *mut T::CLayout| {
            it.is_null().not() &&
            (*it as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,

    @for[T : ReprC]
    Option<::core::ptr::NonNull<T>>
        => |ref _: *const T::CLayout| true
    ,
    @for['a, T : 'a + ReprC]
    Option<&'a T>
        => |ref it: *const T::CLayout| {
            (*it as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
    @for['a, T : 'a + ReprC]
    Option<&'a mut T>
        => |ref it: *mut T::CLayout| {
            (*it as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
}}
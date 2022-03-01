//! Contains the `InlineStorage` trait,and related items.

use std::mem::ManuallyDrop;

/// Type used as the inline storage of a RSmallBox<>/NonExhaustive<>.
///
/// # Safety
///
/// Implementors must:
///
/// - Be types for which all bitpatterns are valid.
///
/// - Not implement Drop,and have no drop glue.
///
pub unsafe trait InlineStorage {}

macro_rules! impl_for_arrays {
    ( ty=$ty:ty , len[ $($len:expr),* $(,)* ] ) => (
        $(
            unsafe impl InlineStorage for [$ty;$len] {}
        )*
    )
}

impl_for_arrays! {
    ty=u8,
    len[
        0,1,2,3,4,5,6,7,8,9,
        10,11,12,13,14,15,16,17,18,19,
        20,21,22,23,24,25,26,27,28,29,
        30,31,32,33,34,35,36,37,38,39,
        40,41,42,43,44,45,46,47,48,49,
        50,51,52,53,54,55,56,57,58,59,
        60,61,62,63,64,
    ]
}

impl_for_arrays! {
    ty=u32,
    len[
        0,1,2,3,4,5,6,7,8,9,
        10,11,12,13,14,15,16,17,18,19,
        20,21,22,23,24,25,26,27,28,29,
        30,31,32,33,34,35,36,37,38,39,
        40,41,42,43,44,45,46,47,48,
    ]
}

impl_for_arrays! {
    ty=u64,
    len[
        0,1,2,3,4,5,6,7,8,9,
        10,11,12,13,14,15,16,17,18,19,
        20,21,22,23,24,
    ]
}

impl_for_arrays! {
    ty=usize,
    len[
        0,1,2,3,4,5,6,7,8,9,
        10,11,12,13,14,15,16,17,18,19,
        20,21,22,23,24,25,26,27,28,29,
        30,31,32,33,34,35,36,37,38,39,
        40,41,42,43,44,45,46,47,48,
    ]
}

macro_rules! declare_alignments {
    (
        $(( $docs:expr, $aligner:ident, $alignment:expr ),)*
    ) => (
        $(
            #[doc=$docs]
            #[derive(Debug, PartialEq, Copy, Clone)]
            #[repr(C)]
            #[repr(align($alignment))]
            pub struct $aligner<Inline>(pub Inline);

            unsafe impl<Inline> InlineStorage for $aligner<Inline>
            where
                Inline:InlineStorage,
            {}
        )*
    )
}

/// Helper types related to the alignemnt of inline storage.
pub mod alignment {
    use super::*;

    declare_alignments! {
        ( "Aligns its contents to an address at a multiple of 1 bytes.",AlignTo1,1 ),
        ( "Aligns its contents to an address at a multiple of 2 bytes.",AlignTo2,2 ),
        ( "Aligns its contents to an address at a multiple of 4 bytes.",AlignTo4,4 ),
        ( "Aligns its contents to an address at a multiple of 8 bytes.",AlignTo8,8 ),
        ( "Aligns its contents to an address at a multiple of 16 bytes.",AlignTo16,16 ),
        ( "Aligns its contents to an address at a multiple of 32 bytes.",AlignTo32,32 ),
        ( "Aligns its contents to an address at a multiple of 64 bytes.",AlignTo64,64 ),
        ( "Aligns its contents to an address at a multiple of 128 bytes.",AlignTo128,128 ),
    }

    /// Aligns its contents to an address to an address at
    /// a multiple of the size of a pointer.
    #[repr(C)]
    #[derive(Debug, PartialEq, Copy, Clone)]
    #[cfg_attr(target_pointer_width = "128", repr(C, align(16)))]
    #[cfg_attr(target_pointer_width = "64", repr(C, align(8)))]
    #[cfg_attr(target_pointer_width = "32", repr(C, align(4)))]
    #[cfg_attr(target_pointer_width = "16", repr(C, align(2)))]
    pub struct AlignToUsize<Inline>(pub Inline);

    unsafe impl<Inline> InlineStorage for AlignToUsize<Inline> where Inline: InlineStorage {}
}

///////////////////////////////////////////////////////////////////////////////

/// Used internally to avoid requiring Rust 1.36.0 .
#[repr(transparent)]
pub(crate) struct ScratchSpace<T, Inline> {
    #[allow(dead_code)]
    inner: ScratchSpaceInner<T, Inline>,
}

#[repr(C)]
union ScratchSpaceInner<T, Inline> {
    value: ManuallyDrop<T>,
    storage: ManuallyDrop<Inline>,
    uninit: (),
}

impl<T, Inline> ScratchSpace<T, Inline> {
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn uninit() -> Self
    where
        Inline: InlineStorage,
    {
        Self::assert_fits_within_storage();
        Self {
            inner: ScratchSpaceInner { uninit: () },
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn new(value: T) -> Self
    where
        Inline: InlineStorage,
    {
        Self::assert_fits_within_storage();
        Self {
            inner: ScratchSpaceInner {
                value: ManuallyDrop::new(value),
            },
        }
    }

    /// Asserts that `T` fits within `Inline`,with the correct alignment and size.
    fn assert_fits_within_storage() {
        let align_val = std::mem::align_of::<T>();
        let align_storage = std::mem::align_of::<Inline>();
        assert!(
            align_val <= align_storage,
            "The alignment of the storage is lower than the value:\n\t{} < {}",
            align_storage,
            align_val,
        );
        let size_val = std::mem::size_of::<T>();
        let size_storage = std::mem::size_of::<Inline>();
        assert!(
            size_val <= size_storage,
            "The size of the storage is smaller than the value:\n\t{} < {}",
            size_storage,
            size_val,
        );
    }

    /// Asserts that `T` fits within `Inline`,with the correct alignment and size.
    const fn assert_fits_within_storage_const() {
        let align_val = std::mem::align_of::<T>();
        let align_storage = std::mem::align_of::<Inline>();
        let size_val = std::mem::size_of::<T>();
        let size_storage = std::mem::size_of::<Inline>();

        #[cfg(not(feature = "rust_1_57"))]
        {
            [(/*The alignment of the storage is lower than the value*/)]
                [(align_val > align_storage) as usize];

            [(/*The size of the storage is smaller than the value*/)]
                [(size_val > size_storage) as usize];
        }
        #[cfg(feature = "rust_1_57")]
        {
            const_panic::concat_assert!(
                align_val <= align_storage,
                "The alignment of the storage is lower than the value:\n\t",
                align_storage,
                " < ",
                align_val,
            );
            const_panic::concat_assert!(
                size_val <= size_storage,
                "The size of the storage is smaller than the value:\n\t",
                size_storage,
                " < ",
                size_val,
            );
        }
    }
}

impl<T, Inline> ScratchSpace<T, Inline> {
    /// # Safety
    ///
    /// You must ensure that `T` has a compatible size/alignement with `Inline`,
    /// and that `Inline` si valid for all bitpatterns.
    #[inline]
    #[allow(dead_code)]
    pub(crate) const unsafe fn new_unchecked(value: T) -> Self {
        Self::assert_fits_within_storage_const();
        Self {
            inner: ScratchSpaceInner {
                value: ManuallyDrop::new(value),
            },
        }
    }
}

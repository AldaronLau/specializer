//! Safe specialization on stable Rust with builder-like pattern
//!
//! # Types of Specialization
//!
//! There are two types of specialization:
//!  - Specializing on types (example: special behavior for a generic when the
//!    generic type is `Arc<str>` or some other type) - what's implemented by
//!    this crate
//!  - Specializing on traits (example: special behavior if the generic type
//!    implements `ToString` or some other trait) - requires nightly
//!    specialization feature
//!
//! <details>
//! <summary>
//! Limited Trait Specialization Workaround
//! </summary>
//!
//! While it's not possible to implement specialization on any trait without
//! nightly, it is possible to define a trait that allows specialization of
//! "optional supertraits" defined as associated types.  The main limitations
//! with this method are that all types must opt-in to a custom specialization
//! trait in additon to the trait they do or don't implement being specialized
//! on, and the traits need to be `dyn` compatible.
//!
//! ```rust,ignore
//! use std::{
//!     any::{self, Any},
//!     fmt::Debug,
//! };
//!
//! use specializer::SpecializerBorrowedParam;
//!
//! pub trait Specialize {
//!     type Debug: ?Sized = Self;
//!
//!     fn try_debug(&self) -> &Self::Debug;
//! }
//!
//! #[derive(Debug)]
//! struct TypeWithDebug(u32);
//! struct TypeWithoutDebug(u32);
//!
//! impl Specialize for TypeWithDebug {
//!     type Debug = dyn Debug;
//!
//!     fn try_debug(&self) -> Self::Debug {
//!         self
//!     }
//! }
//!
//! impl Specialize for TypeWithoutDebug {
//!     fn try_debug(&self) -> &Self {
//!         self
//!     }
//! }
//!
//! fn maybe_debug<T>(specialized: &T) -> String
//! where
//!     T: Specialize
//! {
//!     let fallback = |no_debug: &T| {
//!         any::type_name_of_val(no_debug).to_owned()
//!     };
//!
//!     SpecializerBorrowedParam::new(specialized, fallback)
//!        .specialize(|debug: &dyn Debug| format!("{debug:?}"))
//!        .run()
//! }
//!
//! assert_eq!(maybe_debug(TypeWithDebug(&42)), "TypeWithDebug(42)");
//! assert_eq!(maybe_debug(TypeWithoutDebug(&42)), "TypeWithoutDebug");
//! ```
//!
//! </details>
//!
//! # Getting Started
//!
//! For the simplest example see [`Specializer::specialize_param()`].
//!
//! The other types may be required depending on your use case:
//!
//! | Async | Takes    | Returns  | Type                               |
//! |-------|----------|----------|------------------------------------|
//! | False | Owned    | Owned    | [`Specializer`]                    |
//! | False | Owned    | Borrowed | `SpecializerBorrowedReturn`        |
//! | False | Borrowed | Owned    | `SpecializerBorrowedParam`         |
//! | False | Borrowed | Borrowed | `SpecializerBorrowed`              |
//! | True  | Owned    | Owned    | `AsyncSpecializer`                 |
//! | True  | Owned    | Borrowed | `AsyncSpecializerBorrowedReturn`   |
//! | True  | Borrowed | Owned    | `AsyncSpecializerBorrowedParam`    |
//! | True  | Borrowed | Borrowed | `AsyncSpecializerBorrowed`         |
//!
//! ## Borrowing
//!
//! You can specialize on borrowed types using the `*SpecializerBorrowed*`
//! specializers as long as the borrowed types implement
//! [`CastIdentityBorrowed`], which is automatically implemented for `&T` and
//! `&mut T`, `where T: 'static`.

#![doc(
    html_logo_url = "https://ardaku.github.io/mm/logo.svg",
    html_favicon_url = "https://ardaku.github.io/mm/icon.svg"
)]
#![no_std]
#![forbid(unsafe_code)]
#![warn(
    anonymous_parameters,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences
)]
#![deny(
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_html_tags,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,
    rustdoc::unescaped_backticks,
    rustdoc::redundant_explicit_links
)]

mod specializer;

use core::any::Any;

pub use self::specializer::Specializer;

/// Attempt to cast owned `T` to `U`.
///
/// Returns `None` if they are not the same type.
///
/// ```rust
/// fn only_string<T: 'static>(t: T) -> Option<String> {
///     specializer::cast_identity::<T, String>(t)
/// }
///
/// assert!(only_string(()).is_none());
/// assert!(only_string(1).is_none());
/// assert!(only_string("Hello").is_none());
/// assert_eq!(only_string("Hello".to_string()).as_deref(), Some("Hello"));
/// ```
#[inline(always)]
pub fn cast_identity<T, U>(ty: T) -> Option<U>
where
    T: 'static,
    U: 'static,
{
    <(dyn Any + 'static)>::downcast_mut::<Option<U>>(&mut Some(ty))?.take()
}

/// Attempt to cast `&T` to `&U`.
///
/// Returns `None` if they are not the same type.
///
/// ```rust
/// fn only_string<T: 'static>(t: &T) -> Option<&String> {
///     specializer::cast_identity_ref::<T, String>(t)
/// }
///
/// assert!(only_string(&()).is_none());
/// assert!(only_string(&1).is_none());
/// assert!(only_string(&"Hello").is_none());
/// assert_eq!(
///     only_string(&"Hello".to_string()).map(|x| x.as_str()),
///     Some("Hello"),
/// );
/// ```
#[inline(always)]
pub fn cast_identity_ref<T, U>(ty: &T) -> Option<&U>
where
    T: 'static,
    U: 'static,
{
    <(dyn Any + 'static)>::downcast_ref::<U>(ty)
}

/// Attempt to cast `&mut T` to `&mut U`.
///
/// Returns `None` if they are not the same type.
///
/// ```rust
/// fn only_string<T: 'static>(t: &mut T) -> Option<&mut String> {
///     specializer::cast_identity_mut::<T, String>(t)
/// }
///
/// assert!(only_string(&mut ()).is_none());
/// assert!(only_string(&mut 1).is_none());
/// assert!(only_string(&mut "Hello").is_none());
/// assert_eq!(
///     only_string(&mut "Hello".to_string()),
///     Some(&mut "Hello".to_string()),
/// );
/// ```
#[inline(always)]
pub fn cast_identity_mut<T, U>(ty: &mut T) -> Option<&mut U>
where
    T: 'static,
    U: 'static,
{
    <(dyn Any + 'static)>::downcast_mut::<U>(ty)
}

/// Attempt to cast borrowed `T` to `U`.
///
/// ```rust
/// fn only_string_ref<T: 'static>(t: &T) -> Option<&String> {
///     specializer::cast_identity_borrowed::<&T, &String>(t)
/// }
///
/// assert!(only_string_ref(&()).is_none());
/// assert!(only_string_ref(&1).is_none());
/// assert!(only_string_ref(&"Hello").is_none());
/// assert_eq!(
///     only_string_ref(&"Hello".to_string()).map(|x| x.as_str()),
///     Some("Hello"),
/// );
///
/// fn only_string_mut<T: 'static>(t: &mut T) -> Option<&mut String> {
///     specializer::cast_identity_borrowed::<&mut T, &mut String>(t)
/// }
///
/// assert!(only_string_mut(&mut ()).is_none());
/// assert!(only_string_mut(&mut 1).is_none());
/// assert!(only_string_mut(&mut "Hello").is_none());
/// assert_eq!(
///     only_string_mut(&mut "Hello".to_string()),
///     Some(&mut "Hello".to_string()),
/// );
/// ```
#[inline(always)]
pub fn cast_identity_borrowed<'a, T, U>(ty: T) -> Option<U>
where
    T: CastIdentityBorrowed<'a, U>,
{
    T::cast_identity(ty)
}

/// Identity cast on a borrowed type
///
/// ```rust
/// use specializer::CastIdentityBorrowed;
///
/// #[derive(Debug, PartialEq)]
/// enum MyThings<'a, T> {
///     Nothing,
///     Ref(&'a T),
///     Mut(&'a mut T),
///     Owned(T),
/// }
///
/// impl<'a, T, U> CastIdentityBorrowed<'a, MyThings<'a, U>> for MyThings<'a, T>
/// where
///     T: 'static,
///     U: 'static,
/// {
///     fn cast_identity(self) -> Option<MyThings<'a, U>> {
///         Some(match self {
///             MyThings::Nothing => MyThings::Nothing,
///             MyThings::Ref(thing) => {
///                 MyThings::Ref(specializer::cast_identity_ref(thing)?)
///             }
///             MyThings::Mut(thing) => {
///                 MyThings::Mut(specializer::cast_identity_mut(thing)?)
///             }
///             MyThings::Owned(thing) => {
///                 MyThings::Owned(specializer::cast_identity(thing)?)
///             }
///         })
///     }
/// }
///
/// fn only_u32_things<T>(things: MyThings<'_, T>) -> Option<MyThings<'_, u32>>
/// where
///     T: 'static
/// {
///     specializer::cast_identity_borrowed(things)
/// }
///
/// assert_eq!(
///     only_u32_things(MyThings::Mut(&mut 42u32)),
///     Some(MyThings::Mut(&mut 42)),
/// );
/// assert_eq!(
///     only_u32_things(MyThings::Ref(&42u32)),
///     Some(MyThings::Ref(&42)),
/// );
/// assert_eq!(
///     only_u32_things(MyThings::Owned(42u32)),
///     Some(MyThings::Owned(42)),
/// );
/// assert_eq!(
///     only_u32_things(MyThings::<u32>::Nothing),
///     Some(MyThings::Nothing),
/// );
///
/// assert!(only_u32_things(MyThings::Mut(&mut 42i32)).is_none());
/// assert!(only_u32_things(MyThings::Ref(&42i32)).is_none());
/// assert!(only_u32_things(MyThings::Owned(42i32)).is_none());
/// // Specialization for this variant is not required
/// assert_eq!(
///     only_u32_things(MyThings::<i32>::Nothing),
///     Some(MyThings::<u32>::Nothing),
/// );
/// ```
pub trait CastIdentityBorrowed<'a, U> {
    /// Attempt to cast `self` to `U`.
    fn cast_identity(self) -> Option<U>;
}

impl<'a, T, U> CastIdentityBorrowed<'a, &'a U> for &'a T
where
    T: 'static,
    U: 'static,
{
    fn cast_identity(self) -> Option<&'a U> {
        cast_identity_ref(self)
    }
}

impl<'a, T, U> CastIdentityBorrowed<'a, &'a mut U> for &'a mut T
where
    T: 'static,
    U: 'static,
{
    fn cast_identity(self) -> Option<&'a mut U> {
        cast_identity_mut(self)
    }
}

//! Safe specialization on stable Rust with builder-like pattern

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

use core::{
    any::{Any, TypeId},
    marker::PhantomData,
};

/// Specialized behavior runner
#[derive(Debug)]
pub struct Specializer<T, U, F>(F, PhantomData<fn(T) -> U>);

impl<T, U, F> Specializer<T, U, F>
where
    F: FnOnce(T) -> U,
    T: 'static,
    U: 'static,
{
    /// Create a new specializer with a fallback function.
    pub const fn new_fallback(f: F) -> Self {
        Self(f, PhantomData)
    }

    /// Specialize on the parameter and the return type of the closure.
    pub fn specialize<P, R>(
        self,
        f: impl FnOnce(P) -> R,
    ) -> Specializer<T, U, impl FnOnce(T) -> U>
    where
        P: 'static,
        R: 'static,
    {
        let Specializer(fallback, phantom_data) = self;
        let f = |t: T| -> U {
            if TypeId::of::<T>() == TypeId::of::<P>()
                && TypeId::of::<U>() == TypeId::of::<R>()
            {
                let param = cast_identity::<T, P>(t).unwrap();

                return cast_identity::<R, U>(f(param)).unwrap();
            }

            fallback(t)
        };

        Specializer(f, phantom_data)
    }

    /// Specialize on the parameter of the closure.
    pub fn specialize_param<P>(
        self,
        f: impl FnOnce(P) -> U,
    ) -> Specializer<T, U, impl FnOnce(T) -> U>
    where
        P: 'static,
    {
        self.specialize::<P, U>(f)
    }

    /// Specialize on the return type of the closure.
    pub fn specialize_return<R>(
        self,
        f: impl FnOnce(T) -> R,
    ) -> Specializer<T, U, impl FnOnce(T) -> U>
    where
        R: 'static,
    {
        self.specialize::<T, R>(f)
    }

    /// Run the specializer.
    pub fn run(self, ty: T) -> U {
        (self.0)(ty)
    }
}

/// Attempt to cast `T` to `U`.
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
pub fn cast_identity_mut<T, U>(ty: &mut T) -> Option<&mut U>
where
    T: 'static,
    U: 'static,
{
    <(dyn Any + 'static)>::downcast_mut::<U>(ty)
}

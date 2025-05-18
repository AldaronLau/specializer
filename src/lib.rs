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
    #[inline(always)]
    pub const fn new_fallback(f: F) -> Self {
        Self(f, PhantomData)
    }

    /// Specialize on the parameter and the return type of the closure.
    #[inline]
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
    ///
    /// ```rust
    /// use specializer::Specializer;
    ///
    /// fn specialized<T>(ty: T) -> String
    /// where
    ///     T: 'static
    /// {
    ///     Specializer::new_fallback(|_| "unknown".to_owned())
    ///         .specialize_param(|int: i32| (int * 2).to_string())
    ///         .specialize_param(|string: String| string)
    ///         .run(ty)
    /// }
    ///
    /// assert_eq!(specialized(3), "6");
    /// assert_eq!(specialized("Hello world".to_string()), "Hello world");
    /// assert_eq!(specialized(()), "unknown");
    /// ```
    #[inline]
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
    ///
    /// ```rust
    /// use specializer::Specializer;
    ///
    /// fn specialized<T>(int: i32) -> T
    /// where
    ///     T: 'static + Default
    /// {
    ///     Specializer::new_fallback(|_: i32| -> T { Default::default() })
    ///         .specialize_return(|int| -> i32 { int * 2 })
    ///         .specialize_return(|int| -> String { int.to_string() })
    ///         .run(int)
    /// }
    ///
    /// assert_eq!(specialized::<i32>(3), 6);
    /// assert_eq!(specialized::<String>(3), "3");
    /// assert_eq!(specialized::<u8>(3), 0);
    /// ```
    #[inline]
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
    #[inline]
    pub fn run(self, ty: T) -> U {
        (self.0)(ty)
    }
}

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

/// Trait for specializing on borrowed types.
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

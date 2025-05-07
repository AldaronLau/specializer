//! Safe specialization on stable Rust with builder-like pattern 

#![allow(dead_code)]

use core::{
    any::{Any, TypeId},
    marker::PhantomData,
};

/// Specialized behavior runner
pub struct Specializer<T, U, F>(F, PhantomData<fn(T) -> U>);

impl<T, U, F> Specializer<T, U, F>
where
    F: FnOnce(T) -> U,
    T: 'static,
    U: 'static,
{
    pub const fn new_fallback(f: F) -> Self {
        Self(f, PhantomData)
    }

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

    pub fn specialize_param<P>(
        self,
        f: impl FnOnce(P) -> U,
    ) -> Specializer<T, U, impl FnOnce(T) -> U>
    where
        P: 'static,
    {
        self.specialize::<P, U>(f)
    }

    pub fn specialize_return<R>(
        self,
        f: impl FnOnce(T) -> R,
    ) -> Specializer<T, U, impl FnOnce(T) -> U>
    where
        R: 'static,
    {
        self.specialize::<T, R>(f)
    }

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
    (&mut Some(ty) as &mut dyn Any)
        .downcast_mut::<Option<U>>()?
        .take()
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
    (ty as &dyn Any).downcast_ref::<U>()
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
    (ty as &mut dyn Any).downcast_mut::<U>()
}

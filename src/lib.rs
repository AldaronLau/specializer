//! Specialization

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
pub fn cast_identity<T, U>(ty: T) -> Option<U>
where
    T: 'static,
    U: 'static,
{
    (&mut Some(ty) as &mut dyn Any)
        .downcast_mut::<Option<U>>()?
        .take()
}

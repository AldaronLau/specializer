use core::{any::TypeId, convert, marker::PhantomData};

use crate::CastIdentityBorrowed;

/// Specialized behavior runner (Owned -> Borrowed)
#[derive(Debug)]
pub struct SpecializerBorrowedReturn<T, U, F>(T, F, PhantomData<fn(T) -> U>);

impl<T, U, F> SpecializerBorrowedReturn<T, U, F>
where
    F: FnOnce(T) -> U,
    T: 'static,
    U: CastIdentityBorrowed<U>,
{
    /// Create a new specializer with a fallback function.
    #[inline(always)]
    pub const fn new(params: T, f: F) -> Self {
        Self(params, f, PhantomData)
    }

    /// Specialize on the parameter and the return type of the closure.
    ///
    /// ```rust
    /// use specializer::{CastIdentityBorrowed, SpecializerBorrowedReturn};
    ///
    /// fn specialized<'a, T, U>(a: T, b: &'a u32) -> Option<&'a U>
    /// where
    ///     T: 'static,
    ///     U: 'static,
    /// {
    ///     SpecializerBorrowedReturn::new(a, |_ty| None)
    ///         .specialize(|int: i32| -> Option<&i32> { Some(&42) })
    ///         .specialize(|int: u32| -> Option<&u32> { Some(b) })
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized::<i32, i32>(3, &5), Some(&42));
    /// assert_eq!(specialized::<u32, u32>(3, &5), Some(&5));
    /// assert_eq!(specialized::<(), u32>((), &5), None);
    /// ```
    #[inline]
    pub fn specialize<P, R>(
        self,
        f: impl FnOnce(P) -> R,
    ) -> SpecializerBorrowedReturn<T, U, impl FnOnce(T) -> U>
    where
        P: 'static,
        R: CastIdentityBorrowed<U>,
    {
        let SpecializerBorrowedReturn(ty, fallback, phantom_data) = self;
        let f = |t: T| -> U {
            if <R as CastIdentityBorrowed<U>>::is_same()
                && TypeId::of::<T>() == TypeId::of::<P>()
            {
                let param = crate::cast_identity::<T, P>(t).unwrap();

                return crate::cast_identity_borrowed::<R, U>(f(param))
                    .unwrap();
            }

            fallback(t)
        };

        SpecializerBorrowedReturn(ty, f, phantom_data)
    }

    /// Specialize on the parameter and the return type of the closure, mapping
    /// both.
    ///
    /// ```rust
    /// use std::convert;
    ///
    /// use specializer::SpecializerBorrowedReturn;
    ///
    /// fn specialized<'a, T, U>(a: T, b: &'a U) -> &'a U
    /// where
    ///     T: 'static + Clone,
    ///     U: 'static + From<T> + From<u8>,
    /// {
    ///     let to = |ty: T| -> &U { b };
    ///
    ///     SpecializerBorrowedReturn::new(a, to)
    ///         .specialize(|int: i32| -> &i32 { &42 })
    ///         .specialize_map(
    ///             |int: u8| int * 3,
    ///             to,
    ///             convert::identity::<&U>,
    ///         )
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized::<u8, i32>(3, &5), &mut 5);
    /// assert_eq!(specialized::<i32, i32>(3, &5), &mut 42);
    /// assert_eq!(specialized::<i16, i32>(3, &5), &mut 5);
    /// ```
    #[inline]
    pub fn specialize_map<P, R>(
        self,
        p: impl FnOnce(P) -> P,
        f: impl FnOnce(T) -> U,
        r: impl FnOnce(R) -> R,
    ) -> SpecializerBorrowedReturn<T, U, impl FnOnce(T) -> U>
    where
        P: 'static,
        R: CastIdentityBorrowed<U>,
        U: CastIdentityBorrowed<R>,
    {
        let SpecializerBorrowedReturn(ty, fallback, phantom_data) = self;
        let f = |t: T| -> U {
            if <U as CastIdentityBorrowed<R>>::is_same()
                && TypeId::of::<T>() == TypeId::of::<P>()
            {
                let param = crate::cast_identity::<T, P>(t).unwrap();
                let param = crate::cast_identity::<P, T>(p(param)).unwrap();
                let ret =
                    crate::cast_identity_borrowed::<U, R>(f(param)).unwrap();

                return crate::cast_identity_borrowed::<R, U>(r(ret)).unwrap();
            }

            fallback(t)
        };

        SpecializerBorrowedReturn(ty, f, phantom_data)
    }

    /// Specialize on the parameter of the closure.
    ///
    /// ```rust
    /// use specializer::{CastIdentityBorrowed, SpecializerBorrowedReturn};
    ///
    /// fn specialized<'a, T, U>(a: T, b: &'a U) -> Option<&'a U>
    /// where
    ///     T: 'static,
    ///     U: 'static,
    /// {
    ///     SpecializerBorrowedReturn::new(a, |_ty| None)
    ///         .specialize_param(|int: u32| Some(b))
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized::<i32, i32>(3, &5), None);
    /// assert_eq!(specialized::<u32, u32>(3, &5), Some(&5));
    /// ```
    #[inline]
    pub fn specialize_param<P>(
        self,
        f: impl FnOnce(P) -> U,
    ) -> SpecializerBorrowedReturn<T, U, impl FnOnce(T) -> U>
    where
        P: 'static,
    {
        self.specialize::<P, U>(f)
    }

    /// Specialize on the return type of the closure.
    ///
    /// ```rust
    /// use specializer::{CastIdentityBorrowed, SpecializerBorrowedReturn};
    ///
    /// fn specialized<'a, U>(a: i32, b: &'a u32) -> Option<&'a U>
    /// where
    ///     U: 'static,
    /// {
    ///     SpecializerBorrowedReturn::new(a, |_ty| None)
    ///         .specialize_return(|int| -> Option<&i32> { Some(&42) })
    ///         .specialize_return(|int| -> Option<&u32> { Some(b) })
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized(3, &5), Some(&42i32));
    /// assert_eq!(specialized(3, &5), Some(&5u32));
    /// assert_eq!(specialized::<u8>(3, &5), None);
    /// ```
    #[inline]
    pub fn specialize_return<R>(
        self,
        f: impl FnOnce(T) -> R,
    ) -> SpecializerBorrowedReturn<T, U, impl FnOnce(T) -> U>
    where
        R: CastIdentityBorrowed<U>,
    {
        self.specialize::<T, R>(f)
    }

    /// Specialize on the parameter and the return type of the closure, mapping
    /// the parameter.
    ///
    /// ```rust
    /// use std::convert;
    ///
    /// use specializer::SpecializerBorrowedReturn;
    ///
    /// fn specialized<'a, T, U>(a: T, b: &'a U) -> &'a U
    /// where
    ///     T: 'static + Clone,
    ///     U: 'static + From<T> + From<u8>,
    /// {
    ///     let to = |ty: T| -> &U { b };
    ///
    ///     SpecializerBorrowedReturn::new(a, to)
    ///         .specialize(|int: i32| -> &i32 { &42 })
    ///         .specialize_map_param(|int: u8| int * 3, to)
    ///         .run()
    /// }
    ///
    /// let value = 3;
    ///
    /// assert_eq!(specialized::<u8, i32>(3, &5), &mut 5);
    /// assert_eq!(specialized::<i32, i32>(3, &5), &mut 42);
    /// assert_eq!(specialized::<i16, i32>(3, &5), &mut 5);
    /// ```
    #[inline]
    pub fn specialize_map_param<P>(
        self,
        p: impl FnOnce(P) -> P,
        f: impl FnOnce(T) -> U,
    ) -> SpecializerBorrowedReturn<T, U, impl FnOnce(T) -> U>
    where
        P: 'static,
    {
        self.specialize_map::<P, U>(p, f, convert::identity)
    }

    /// Specialize on the parameter and the return type of the closure, mapping
    /// the parameter.
    ///
    /// ```rust
    /// use std::convert;
    ///
    /// use specializer::SpecializerBorrowedReturn;
    ///
    /// fn specialized<'a, U>(a: i8, b: &'a i32, c: &'a U) -> &'a U
    /// where
    ///     U: 'static,
    /// {
    ///     let to = |ty: i8| -> &U { c };
    ///
    ///     SpecializerBorrowedReturn::new(a, to)
    ///         .specialize_return(|int| -> &i8 { &16 })
    ///         .specialize_map_return(
    ///             to,
    ///             |ty| -> &i32 { b },
    ///         )
    ///         .specialize_map_return(
    ///             |ty| -> &U { c },
    ///             |ty| -> &i16 { &15 },
    ///         )
    ///         .run()
    /// }
    ///
    /// let value = 3;
    ///
    /// assert_eq!(specialized::<i8>(value, &5, &42), &16);
    /// assert_eq!(specialized::<i64>(value, &5, &42), &42);
    /// assert_eq!(specialized::<i32>(value, &5, &42), &5);
    /// assert_eq!(specialized::<i16>(value, &5, &42), &15);
    /// ```
    #[inline]
    pub fn specialize_map_return<R>(
        self,
        f: impl FnOnce(T) -> U,
        r: impl FnOnce(R) -> R,
    ) -> SpecializerBorrowedReturn<T, U, impl FnOnce(T) -> U>
    where
        R: CastIdentityBorrowed<U>,
        U: CastIdentityBorrowed<R>,
    {
        self.specialize_map::<T, R>(convert::identity, f, r)
    }

    /// Run the specializer.
    #[inline]
    pub fn run(self) -> U {
        (self.1)(self.0)
    }
}

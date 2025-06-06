use core::{convert, marker::PhantomData};

use crate::CastIdentityBorrowed;

/// Specialized behavior runner (Borrowed -> Borrowed)
#[derive(Debug)]
pub struct SpecializerBorrowed<T, U, F>(T, F, PhantomData<fn(T) -> U>);

impl<T, U, F> SpecializerBorrowed<T, U, F>
where
    F: FnOnce(T) -> U,
    T: CastIdentityBorrowed<T>,
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
    /// use specializer::{CastIdentityBorrowed, SpecializerBorrowed};
    ///
    /// fn specialized<'a, T, U>(a: &'a mut T, b: &'a u32) -> Option<&'a U>
    /// where
    ///     T: 'static,
    ///     U: 'static,
    /// {
    ///     SpecializerBorrowed::new(a, |_ty| None)
    ///         .specialize(|int: &mut i32| -> Option<&i32> { Some(int) })
    ///         .specialize(|int: &mut u32| -> Option<&u32> { Some(b) })
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized::<i32, i32>(&mut 3, &5), Some(&3));
    /// assert_eq!(specialized::<u32, u32>(&mut 3, &5), Some(&5));
    /// assert_eq!(specialized::<(), u32>(&mut (), &5), None);
    /// ```
    #[inline]
    pub fn specialize<P, R>(
        self,
        f: impl FnOnce(P) -> R,
    ) -> SpecializerBorrowed<T, U, impl FnOnce(T) -> U>
    where
        T: CastIdentityBorrowed<P>,
        R: CastIdentityBorrowed<U>,
    {
        let SpecializerBorrowed(ty, fallback, phantom_data) = self;
        let f = |t: T| -> U {
            if <R as CastIdentityBorrowed<U>>::is_same()
                && <T as CastIdentityBorrowed<P>>::is_same()
            {
                let param = crate::cast_identity_borrowed::<T, P>(t).unwrap();

                return crate::cast_identity_borrowed::<R, U>(f(param))
                    .unwrap();
            }

            fallback(t)
        };

        SpecializerBorrowed(ty, f, phantom_data)
    }

    /// Specialize on the parameter and the return type of the closure, mapping
    /// both.
    ///
    /// ```rust
    /// use std::convert;
    ///
    /// use specializer::SpecializerBorrowed;
    ///
    /// fn specialized<'a, T, U>(a: &'a mut T, b: &'a U) -> &'a U
    /// where
    ///     T: 'static + Clone,
    ///     U: 'static + From<T> + From<u8>,
    /// {
    ///     let to = |ty: &mut T| -> &U { b };
    ///
    ///     SpecializerBorrowed::new(a, to)
    ///         .specialize(|int: &mut i32| -> &i32 {
    ///             *int *= 2;
    ///             int
    ///         })
    ///         .specialize_map(
    ///             |int: &mut u8| {
    ///                 *int *= 3;
    ///                 int
    ///             },
    ///             to,
    ///             convert::identity::<&U>,
    ///         )
    ///         .run()
    /// }
    ///
    /// let mut value = 3;
    ///
    /// assert_eq!(specialized::<u8, i32>(&mut value, &5), &mut 5);
    /// assert_eq!(value, 9);
    ///
    /// let mut value = 3;
    ///
    /// assert_eq!(specialized::<i32, i32>(&mut value, &5), &mut 6);
    /// assert_eq!(value, 6);
    ///
    /// let mut value = 3;
    ///
    /// assert_eq!(specialized::<i16, i32>(&mut value, &5), &mut 5);
    /// assert_eq!(value, 3);
    /// ```
    #[inline]
    pub fn specialize_map<P, R>(
        self,
        p: impl FnOnce(P) -> P,
        f: impl FnOnce(T) -> U,
        r: impl FnOnce(R) -> R,
    ) -> SpecializerBorrowed<T, U, impl FnOnce(T) -> U>
    where
        T: CastIdentityBorrowed<P>,
        P: CastIdentityBorrowed<T>,
        R: CastIdentityBorrowed<U>,
        U: CastIdentityBorrowed<R>,
    {
        let SpecializerBorrowed(ty, fallback, phantom_data) = self;
        let f = |t: T| -> U {
            if <U as CastIdentityBorrowed<R>>::is_same()
                && <T as CastIdentityBorrowed<P>>::is_same()
            {
                let param = crate::cast_identity_borrowed::<T, P>(t).unwrap();
                let param =
                    crate::cast_identity_borrowed::<P, T>(p(param)).unwrap();
                let ret =
                    crate::cast_identity_borrowed::<U, R>(f(param)).unwrap();

                return crate::cast_identity_borrowed::<R, U>(r(ret)).unwrap();
            }

            fallback(t)
        };

        SpecializerBorrowed(ty, f, phantom_data)
    }

    /// Specialize on the parameter of the closure.
    ///
    /// ```rust
    /// use specializer::{CastIdentityBorrowed, SpecializerBorrowed};
    ///
    /// fn specialized<'a, T, U>(a: &'a mut T, b: &'a U) -> Option<&'a U>
    /// where
    ///     T: 'static,
    ///     U: 'static,
    /// {
    ///     SpecializerBorrowed::new(a, |_ty| None)
    ///         .specialize_param(|int: &mut u32| Some(b))
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized::<i32, i32>(&mut 3, &5), None);
    /// assert_eq!(specialized::<u32, u32>(&mut 3, &5), Some(&5));
    /// ```
    #[inline]
    pub fn specialize_param<P>(
        self,
        f: impl FnOnce(P) -> U,
    ) -> SpecializerBorrowed<T, U, impl FnOnce(T) -> U>
    where
        T: CastIdentityBorrowed<P>,
    {
        self.specialize::<P, U>(f)
    }

    /// Specialize on the return type of the closure.
    ///
    /// ```rust
    /// use specializer::{CastIdentityBorrowed, SpecializerBorrowed};
    ///
    /// fn specialized<'a, U>(a: &'a mut i32, b: &'a u32) -> Option<&'a U>
    /// where
    ///     U: 'static,
    /// {
    ///     SpecializerBorrowed::new(a, |_ty| None)
    ///         .specialize_return(|int| -> Option<&i32> { Some(int) })
    ///         .specialize_return(|int| -> Option<&u32> { Some(b) })
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized(&mut 3, &5), Some(&3i32));
    /// assert_eq!(specialized(&mut 3, &5), Some(&5u32));
    /// assert_eq!(specialized::<u8>(&mut 3, &5), None);
    /// ```
    #[inline]
    pub fn specialize_return<R>(
        self,
        f: impl FnOnce(T) -> R,
    ) -> SpecializerBorrowed<T, U, impl FnOnce(T) -> U>
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
    /// use specializer::SpecializerBorrowed;
    ///
    /// fn specialized<'a, T, U>(a: &'a mut T, b: &'a U) -> &'a U
    /// where
    ///     T: 'static + Clone,
    ///     U: 'static + From<T> + From<u8>,
    /// {
    ///     let to = |ty: &mut T| -> &U { b };
    ///
    ///     SpecializerBorrowed::new(a, to)
    ///         .specialize(|int: &mut i32| -> &i32 {
    ///             *int *= 2;
    ///             int
    ///         })
    ///         .specialize_map_param(
    ///             |int: &mut u8| {
    ///                 *int *= 3;
    ///                 int
    ///             },
    ///             to,
    ///         )
    ///         .run()
    /// }
    ///
    /// let mut value = 3;
    ///
    /// assert_eq!(specialized::<u8, i32>(&mut value, &5), &mut 5);
    /// assert_eq!(value, 9);
    ///
    /// let mut value = 3;
    ///
    /// assert_eq!(specialized::<i32, i32>(&mut value, &5), &mut 6);
    /// assert_eq!(value, 6);
    ///
    /// let mut value = 3;
    ///
    /// assert_eq!(specialized::<i16, i32>(&mut value, &5), &mut 5);
    /// assert_eq!(value, 3);
    /// ```
    #[inline]
    pub fn specialize_map_param<P>(
        self,
        p: impl FnOnce(P) -> P,
        f: impl FnOnce(T) -> U,
    ) -> SpecializerBorrowed<T, U, impl FnOnce(T) -> U>
    where
        T: CastIdentityBorrowed<P>,
        P: CastIdentityBorrowed<T>,
    {
        self.specialize_map::<P, U>(p, f, convert::identity)
    }

    /// Specialize on the parameter and the return type of the closure, mapping
    /// the parameter.
    ///
    /// ```rust
    /// use std::convert;
    ///
    /// use specializer::SpecializerBorrowed;
    ///
    /// fn specialized<'a, U>(a: &'a mut i8, b: &'a i32, c: &'a U) -> &'a U
    /// where
    ///     U: 'static,
    /// {
    ///     let to = |ty: &mut i8| -> &U {
    ///         *ty *= 3;
    ///         c
    ///     };
    ///
    ///     SpecializerBorrowed::new(a, to)
    ///         .specialize_return(|int| -> &i8 {
    ///             *int *= 2;
    ///             int
    ///         })
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
    /// let mut value = 3;
    ///
    /// assert_eq!(specialized::<i8>(&mut value, &5, &42), &6);
    /// assert_eq!(value, 6);
    ///
    /// let mut value = 3;
    ///
    /// assert_eq!(specialized::<i64>(&mut value, &5, &42), &42);
    /// assert_eq!(value, 9);
    ///
    /// let mut value = 3;
    ///
    /// assert_eq!(specialized::<i32>(&mut value, &5, &42), &5);
    /// assert_eq!(value, 9);
    ///
    /// let mut value = 3;
    ///
    /// assert_eq!(specialized::<i16>(&mut value, &5, &42), &15);
    /// assert_eq!(value, 3);
    /// ```
    #[inline]
    pub fn specialize_map_return<R>(
        self,
        f: impl FnOnce(T) -> U,
        r: impl FnOnce(R) -> R,
    ) -> SpecializerBorrowed<T, U, impl FnOnce(T) -> U>
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

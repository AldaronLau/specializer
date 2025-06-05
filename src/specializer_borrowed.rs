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
    /*
     * ```rust
     * use specializer::SpecializerBorrowed;
     *
     * fn specialized<T, U>(ty: &mut T) -> U
     * where
     *     T: 'static + Clone,
     *     U: 'static + From<T> + From<u8> + From<i32>,
     * {
     *     SpecializerBorrowed::new(ty, |ty| ty.clone().into())
     *         .specialize_param(|int: &mut i32| { U::from( *int * 2) })
     *         .specialize_param(|int: &mut u8| { U::from(*int * 3) })
     *         .run()
     * }
     *
     * assert_eq!(specialized::<i16, i32>(&mut 3), 3);
     * assert_eq!(specialized::<i32, i32>(&mut 3), 6);
     * assert_eq!(specialized::<u8, i32>(&mut 3), 9);
     * ```
     * */ */
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
    /*
     * ```rust
     * use specializer::SpecializerBorrowed;
     *
     * fn specialized<T>(int: &mut i32) -> T
     * where
     *     T: 'static + Default
     * {
     *     let fallback = |_| -> T { Default::default() };
     *
     *     SpecializerBorrowed::new(int, fallback)
     *         .specialize_return(|&mut int| -> i32 { int * 2 })
     *         .specialize_return(|&mut int| -> String { int.to_string() })
     *         .run()
     * }
     *
     * assert_eq!(specialized::<i32>(&mut 3), 6);
     * assert_eq!(specialized::<String>(&mut 3), "3");
     * assert_eq!(specialized::<u8>(&mut 3), 0);
     * ```
     */
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
    /*
     * ```rust
     * use specializer::SpecializerBorrowed;
     *
     * fn specialized<T, U>(ty: &mut T) -> U
     * where
     *     T: 'static + Clone,
     *     U: 'static + From<T>,
     * {
     *     let f = |x: &mut T| (*x).clone().into();
     *
     *     SpecializerBorrowed::new(ty, f)
     *         .specialize(|int: &mut i32| -> i32 { *int * 2 })
     *         .specialize_map_param(
     *             |int: &mut u8| {
     *                 *int *= 3;
     *                 int
     *             },
     *             f,
     *         )
     *         .run()
     * }
     *
     * assert_eq!(specialized::<i16, i32>(&mut 3), 3);
     * assert_eq!(specialized::<i32, i32>(&mut 3), 6);
     * assert_eq!(specialized::<u8, i32>(&mut 3), 9);
     * ```
     */
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
    /*
     * ```rust
     * use specializer::SpecializerBorrowed;
     *
     * fn specialized<T, U>(ty: &mut T) -> U
     * where
     *     T: 'static + Clone,
     *     U: 'static + From<T>,
     * {
     *     let f = |x: &mut T| (*x).clone().into();
     *
     *     SpecializerBorrowed::new(ty, f)
     *         .specialize_map_return(f, |int: i16| int * 2)
     *         .specialize_map_param(
     *             |int: &mut u8| {
     *                 *int *= 3;
     *                 int
     *             },
     *             f,
     *         )
     *         .run()
     * }
     *
     * assert_eq!(specialized::<i16, i32>(&mut 3), 3);
     * assert_eq!(specialized::<i8, i16>(&mut 3), 6);
     * assert_eq!(specialized::<u8, i32>(&mut 3), 9);
     * ```
     */
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

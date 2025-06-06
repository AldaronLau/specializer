use core::{any::TypeId, convert, marker::PhantomData};

use crate::CastIdentityBorrowed;

/// Specialized behavior runner (Borrowed -> Owned)
#[derive(Debug)]
pub struct SpecializerBorrowedParam<T, U, F>(T, F, PhantomData<fn(T) -> U>);

impl<T, U, F> SpecializerBorrowedParam<T, U, F>
where
    F: FnOnce(T) -> U,
    T: CastIdentityBorrowed<T>,
    U: 'static,
{
    /// Create a new specializer with a fallback function.
    #[inline(always)]
    pub const fn new(params: T, f: F) -> Self {
        Self(params, f, PhantomData)
    }

    /// Specialize on the parameter and the return type of the closure.
    ///
    /// ```rust
    /// use specializer::SpecializerBorrowedParam;
    ///
    /// fn specialized<T, U>(ty: &mut T) -> U
    /// where
    ///     T: 'static + Clone,
    ///     U: 'static + From<T> + From<u8>,
    /// {
    ///     SpecializerBorrowedParam::new(ty, |ty| ty.clone().into())
    ///         .specialize(|int: &mut i32| -> i32 { *int * 2 })
    ///         .specialize_param(|int: &mut u8| { U::from(*int * 3) })
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized::<i16, i32>(&mut 3), 3);
    /// assert_eq!(specialized::<i32, i32>(&mut 3), 6);
    /// assert_eq!(specialized::<u8, i32>(&mut 3), 9);
    /// ```
    #[inline]
    pub fn specialize<P, R>(
        self,
        f: impl FnOnce(P) -> R,
    ) -> SpecializerBorrowedParam<T, U, impl FnOnce(T) -> U>
    where
        T: CastIdentityBorrowed<P>,
        R: 'static,
    {
        let SpecializerBorrowedParam(ty, fallback, phantom_data) = self;
        let f = |t: T| -> U {
            if TypeId::of::<U>() == TypeId::of::<R>()
                && <T as CastIdentityBorrowed<P>>::is_same()
            {
                let param = crate::cast_identity_borrowed::<T, P>(t).unwrap();

                return crate::cast_identity::<R, U>(f(param)).unwrap();
            }

            fallback(t)
        };

        SpecializerBorrowedParam(ty, f, phantom_data)
    }

    /// Specialize on the parameter and the return type of the closure, mapping
    /// both.
    ///
    /// ```rust
    /// use std::convert;
    ///
    /// use specializer::SpecializerBorrowedParam;
    ///
    /// fn specialized<T, U>(ty: &mut T) -> U
    /// where
    ///     T: 'static + Clone,
    ///     U: 'static + From<T> + From<u8>,
    /// {
    ///     let to = |ty: &mut T| ty.clone().into();
    ///
    ///     SpecializerBorrowedParam::new(ty, to)
    ///         .specialize(|int: &mut i32| -> i32 { *int * 2 })
    ///         .specialize_map(
    ///             |int: &mut u8| {
    ///                 *int *= 3;
    ///                 int
    ///             },
    ///             to,
    ///             convert::identity::<U>,
    ///         )
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized::<i16, i32>(&mut 3), 3);
    /// assert_eq!(specialized::<i32, i32>(&mut 3), 6);
    /// assert_eq!(specialized::<u8, i32>(&mut 3), 9);
    /// ```
    #[inline]
    pub fn specialize_map<P, R>(
        self,
        p: impl FnOnce(P) -> P,
        f: impl FnOnce(T) -> U,
        r: impl FnOnce(R) -> R,
    ) -> SpecializerBorrowedParam<T, U, impl FnOnce(T) -> U>
    where
        T: CastIdentityBorrowed<P>,
        P: CastIdentityBorrowed<T>,
        R: 'static,
    {
        let SpecializerBorrowedParam(ty, fallback, phantom_data) = self;
        let f = |t: T| -> U {
            if TypeId::of::<U>() == TypeId::of::<R>()
                && <T as CastIdentityBorrowed<P>>::is_same()
            {
                let param = crate::cast_identity_borrowed::<T, P>(t).unwrap();
                let param =
                    crate::cast_identity_borrowed::<P, T>(p(param)).unwrap();
                let ret = crate::cast_identity::<U, R>(f(param)).unwrap();

                return crate::cast_identity::<R, U>(r(ret)).unwrap();
            }

            fallback(t)
        };

        SpecializerBorrowedParam(ty, f, phantom_data)
    }

    /// Specialize on the parameter of the closure.
    ///
    /// ```rust
    /// use specializer::SpecializerBorrowedParam;
    ///
    /// fn specialized<T, U>(ty: &mut T) -> U
    /// where
    ///     T: 'static + Clone,
    ///     U: 'static + From<T> + From<u8> + From<i32>,
    /// {
    ///     SpecializerBorrowedParam::new(ty, |ty| ty.clone().into())
    ///         .specialize_param(|int: &mut i32| { U::from( *int * 2) })
    ///         .specialize_param(|int: &mut u8| { U::from(*int * 3) })
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized::<i16, i32>(&mut 3), 3);
    /// assert_eq!(specialized::<i32, i32>(&mut 3), 6);
    /// assert_eq!(specialized::<u8, i32>(&mut 3), 9);
    /// ```
    #[inline]
    pub fn specialize_param<P>(
        self,
        f: impl FnOnce(P) -> U,
    ) -> SpecializerBorrowedParam<T, U, impl FnOnce(T) -> U>
    where
        T: CastIdentityBorrowed<P>,
    {
        self.specialize::<P, U>(f)
    }

    /// Specialize on the return type of the closure.
    ///
    /// ```rust
    /// use specializer::SpecializerBorrowedParam;
    ///
    /// fn specialized<T>(int: &mut i32) -> T
    /// where
    ///     T: 'static + Default
    /// {
    ///     let fallback = |_| -> T { Default::default() };
    ///
    ///     SpecializerBorrowedParam::new(int, fallback)
    ///         .specialize_return(|&mut int| -> i32 { int * 2 })
    ///         .specialize_return(|&mut int| -> String { int.to_string() })
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized::<i32>(&mut 3), 6);
    /// assert_eq!(specialized::<String>(&mut 3), "3");
    /// assert_eq!(specialized::<u8>(&mut 3), 0);
    /// ```
    #[inline]
    pub fn specialize_return<R>(
        self,
        f: impl FnOnce(T) -> R,
    ) -> SpecializerBorrowedParam<T, U, impl FnOnce(T) -> U>
    where
        R: 'static,
    {
        self.specialize::<T, R>(f)
    }

    /// Specialize on the parameter and the return type of the closure, mapping
    /// the parameter.
    ///
    /// ```rust
    /// use specializer::SpecializerBorrowedParam;
    ///
    /// fn specialized<T, U>(ty: &mut T) -> U
    /// where
    ///     T: 'static + Clone,
    ///     U: 'static + From<T>,
    /// {
    ///     let f = |x: &mut T| (*x).clone().into();
    ///
    ///     SpecializerBorrowedParam::new(ty, f)
    ///         .specialize(|int: &mut i32| -> i32 { *int * 2 })
    ///         .specialize_map_param(
    ///             |int: &mut u8| {
    ///                 *int *= 3;
    ///                 int
    ///             },
    ///             f,
    ///         )
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized::<i16, i32>(&mut 3), 3);
    /// assert_eq!(specialized::<i32, i32>(&mut 3), 6);
    /// assert_eq!(specialized::<u8, i32>(&mut 3), 9);
    /// ```
    #[inline]
    pub fn specialize_map_param<P>(
        self,
        p: impl FnOnce(P) -> P,
        f: impl FnOnce(T) -> U,
    ) -> SpecializerBorrowedParam<T, U, impl FnOnce(T) -> U>
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
    /// use specializer::SpecializerBorrowedParam;
    ///
    /// fn specialized<T, U>(ty: &mut T) -> U
    /// where
    ///     T: 'static + Clone,
    ///     U: 'static + From<T>,
    /// {
    ///     let f = |x: &mut T| (*x).clone().into();
    ///
    ///     SpecializerBorrowedParam::new(ty, f)
    ///         .specialize_map_return(f, |int: i16| int * 2)
    ///         .specialize_map_param(
    ///             |int: &mut u8| {
    ///                 *int *= 3;
    ///                 int
    ///             },
    ///             f,
    ///         )
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized::<i16, i32>(&mut 3), 3);
    /// assert_eq!(specialized::<i8, i16>(&mut 3), 6);
    /// assert_eq!(specialized::<u8, i32>(&mut 3), 9);
    /// ```
    #[inline]
    pub fn specialize_map_return<R>(
        self,
        f: impl FnOnce(T) -> U,
        r: impl FnOnce(R) -> R,
    ) -> SpecializerBorrowedParam<T, U, impl FnOnce(T) -> U>
    where
        R: 'static,
    {
        self.specialize_map::<T, R>(convert::identity, f, r)
    }

    /// Run the specializer.
    #[inline]
    pub fn run(self) -> U {
        (self.1)(self.0)
    }
}

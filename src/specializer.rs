use core::{any::TypeId, marker::PhantomData, convert};

/// Specialized behavior runner (Owned -> Owned)
#[derive(Debug)]
pub struct Specializer<T, U, F>(T, F, PhantomData<fn(T) -> U>);

impl<T, U, F> Specializer<T, U, F>
where
    F: FnOnce(T) -> U,
    T: 'static,
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
    /// use specializer::Specializer;
    ///
    /// fn specialized<T, U>(ty: T) -> U
    /// where
    ///     T: 'static,
    ///     U: 'static + From<T> + From<u8>,
    /// {
    ///     Specializer::new(ty, From::from)
    ///         .specialize(|int: i32| -> i32 { int * 2 })
    ///         .specialize_param(|int: u8| { U::from(int * 3) })
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized::<i16, i32>(3), 3);
    /// assert_eq!(specialized::<i32, i32>(3), 6);
    /// assert_eq!(specialized::<u8, i32>(3), 9);
    /// ```
    #[inline]
    pub fn specialize<P, R>(
        self,
        f: impl FnOnce(P) -> R,
    ) -> Specializer<T, U, impl FnOnce(T) -> U>
    where
        P: 'static,
        R: 'static,
    {
        let Specializer(ty, fallback, phantom_data) = self;
        let f = |t: T| -> U {
            if TypeId::of::<T>() == TypeId::of::<P>()
                && TypeId::of::<U>() == TypeId::of::<R>()
            {
                let param = crate::cast_identity::<T, P>(t).unwrap();

                return crate::cast_identity::<R, U>(f(param)).unwrap();
            }

            fallback(t)
        };

        Specializer(ty, f, phantom_data)
    }

    /// Specialize on the parameter and the return type of the closure, mapping
    /// both.
    ///
    /// ```rust
    /// use std::convert;
    ///
    /// use specializer::Specializer;
    ///
    /// fn specialized<T, U>(ty: T) -> U
    /// where
    ///     T: 'static,
    ///     U: 'static + From<T>,
    /// {
    ///     Specializer::new(ty, From::from)
    ///         .specialize(|int: i32| -> i32 { int * 2 })
    ///         .specialize_map(
    ///             |int: u8| int * 3,
    ///             From::from,
    ///             convert::identity::<U>,
    ///         )
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized::<i16, i32>(3), 3);
    /// assert_eq!(specialized::<i32, i32>(3), 6);
    /// assert_eq!(specialized::<u8, i32>(3), 9);
    /// ```
    #[inline]
    pub fn specialize_map<P, R>(
        self,
        p: impl FnOnce(P) -> P,
        f: impl FnOnce(T) -> U,
        r: impl FnOnce(R) -> R,
    ) -> Specializer<T, U, impl FnOnce(T) -> U>
    where
        P: 'static,
        R: 'static,
    {
        let Specializer(ty, fallback, phantom_data) = self;
        let f = |t: T| -> U {
            if TypeId::of::<T>() == TypeId::of::<P>()
                && TypeId::of::<U>() == TypeId::of::<R>()
            {
                let param = crate::cast_identity::<T, P>(t).unwrap();
                let param = crate::cast_identity::<P, T>(p(param)).unwrap();
                let ret = crate::cast_identity::<U, R>(f(param)).unwrap();

                return crate::cast_identity::<R, U>(r(ret)).unwrap();
            }

            fallback(t)
        };

        Specializer(ty, f, phantom_data)
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
    ///     let fallback = |_| "unknown".to_owned();
    ///
    ///     Specializer::new(ty, fallback)
    ///         .specialize_param(|int: i32| (int * 2).to_string())
    ///         .specialize_param(|string: String| string)
    ///         .run()
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
    ///     let fallback = |_| -> T { Default::default() };
    ///
    ///     Specializer::new(int, fallback)
    ///         .specialize_return(|int| -> i32 { int * 2 })
    ///         .specialize_return(|int| -> String { int.to_string() })
    ///         .run()
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

    /// Specialize on the parameter and the return type of the closure, mapping
    /// the parameter.
    ///
    /// ```rust
    /// use std::convert;
    ///
    /// use specializer::Specializer;
    ///
    /// fn specialized<T, U>(ty: T) -> U
    /// where
    ///     T: 'static,
    ///     U: 'static + From<T>,
    /// {
    ///     Specializer::new(ty, From::from)
    ///         .specialize(|int: i32| -> i32 { int * 2 })
    ///         .specialize_map_param(|int: u8| int * 3, From::from)
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized::<i16, i32>(3), 3);
    /// assert_eq!(specialized::<i32, i32>(3), 6);
    /// assert_eq!(specialized::<u8, i32>(3), 9);
    /// ```
    #[inline]
    pub fn specialize_map_param<P>(
        self,
        p: impl FnOnce(P) -> P,
        f: impl FnOnce(T) -> U,
    ) -> Specializer<T, U, impl FnOnce(T) -> U>
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
    /// use specializer::Specializer;
    ///
    /// fn specialized<T, U>(ty: T) -> U
    /// where
    ///     T: 'static,
    ///     U: 'static + From<T>,
    /// {
    ///     Specializer::new(ty, From::from)
    ///         .specialize_map_return(From::from, |int: i16| int * 2)
    ///         .specialize_map_param(|int: u8| int * 3, From::from)
    ///         .run()
    /// }
    ///
    /// assert_eq!(specialized::<i16, i32>(3), 3);
    /// assert_eq!(specialized::<i8, i16>(3), 6);
    /// assert_eq!(specialized::<u8, i32>(3), 9);
    /// ```
    #[inline]
    pub fn specialize_map_return<R>(
        self,
        f: impl FnOnce(T) -> U,
        r: impl FnOnce(R) -> R,
    ) -> Specializer<T, U, impl FnOnce(T) -> U>
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

use core::{any::TypeId, future, marker::PhantomData};

use crate::CastIdentityBorrowed;

/// Async specialized behavior runner (Borrowed -> Owned)
#[derive(Debug)]
pub struct AsyncSpecializerBorrowedParam<T, U, F>(
    T,
    F,
    PhantomData<fn(T) -> U>,
);

impl<T, U, F> AsyncSpecializerBorrowedParam<T, U, F>
where
    F: AsyncFnOnce(T) -> U,
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
    /// use specializer::AsyncSpecializerBorrowedParam;
    /// use pasts::Executor;
    ///
    /// async fn specialized<T, U>(ty: &mut T) -> U
    /// where
    ///     T: 'static + Clone,
    ///     U: 'static + From<T> + From<u8>,
    /// {
    ///     AsyncSpecializerBorrowedParam::new(ty, async |ty| ty.clone().into())
    ///         .specialize(async |int: &mut i32| -> i32 { *int * 2 })
    ///         .specialize_param(async |int: &mut u8| { U::from(*int * 3) })
    ///         .run()
    ///         .await
    /// }
    ///
    /// Executor::default().block_on(async {
    ///     assert_eq!(specialized::<i16, i32>(&mut 3).await, 3);
    ///     assert_eq!(specialized::<i32, i32>(&mut 3).await, 6);
    ///     assert_eq!(specialized::<u8, i32>(&mut 3).await, 9);
    /// });
    /// ```
    #[inline]
    pub fn specialize<P, R>(
        self,
        f: impl AsyncFnOnce(P) -> R,
    ) -> AsyncSpecializerBorrowedParam<T, U, impl AsyncFnOnce(T) -> U>
    where
        T: CastIdentityBorrowed<P>,
        R: 'static,
    {
        let AsyncSpecializerBorrowedParam(ty, fallback, phantom_data) = self;
        let f = async |t: T| -> U {
            if TypeId::of::<U>() == TypeId::of::<R>()
                && <T as CastIdentityBorrowed<P>>::is_same()
            {
                let param = crate::cast_identity_borrowed::<T, P>(t).unwrap();

                return crate::cast_identity::<R, U>(f(param).await).unwrap();
            }

            fallback(t).await
        };

        AsyncSpecializerBorrowedParam(ty, f, phantom_data)
    }

    /// Specialize on the parameter and the return type of the closure, mapping
    /// both.
    /*
     * ```rust
     * use std::future;
     *
     * use specializer::AsyncSpecializerBorrowedParam;
     * use pasts::Executor;
     *
     * async fn specialized<T, U>(ty: T) -> U
     * where
     *     T: 'static,
     *     U: 'static + From<T>,
     * {
     *     AsyncSpecializerBorrowedParam::new(ty, async |ty| ty.into())
     *         .specialize(async |int: i32| -> i32 { int * 2 })
     *         .specialize_map(
     *             async |int: u8| int * 3,
     *             async |ty| ty.into(),
     *             future::ready::<U>,
     *         )
     *         .run()
     *         .await
     * }
     *
     * Executor::default().block_on(async {
     *     assert_eq!(specialized::<i16, i32>(3).await, 3);
     *     assert_eq!(specialized::<i32, i32>(3).await, 6);
     *     assert_eq!(specialized::<u8, i32>(3).await, 9);
     * });
     * ``` */
    #[inline]
    pub fn specialize_map<P, R>(
        self,
        p: impl AsyncFnOnce(P) -> P,
        f: impl AsyncFnOnce(T) -> U,
        r: impl AsyncFnOnce(R) -> R,
    ) -> AsyncSpecializerBorrowedParam<T, U, impl AsyncFnOnce(T) -> U>
    where
        T: CastIdentityBorrowed<P>,
        P: CastIdentityBorrowed<T>,
        R: 'static,
    {
        let AsyncSpecializerBorrowedParam(ty, fallback, phantom_data) = self;
        let f = async |t: T| -> U {
            if TypeId::of::<U>() == TypeId::of::<R>()
                && <T as CastIdentityBorrowed<P>>::is_same()
            {
                let param = crate::cast_identity_borrowed::<T, P>(t).unwrap();
                let param =
                    crate::cast_identity_borrowed::<P, T>(p(param).await)
                        .unwrap();
                let ret = crate::cast_identity::<U, R>(f(param).await).unwrap();

                return crate::cast_identity::<R, U>(r(ret).await).unwrap();
            }

            fallback(t).await
        };

        AsyncSpecializerBorrowedParam(ty, f, phantom_data)
    }

    /// Specialize on the parameter of the closure.
    /*
     * ```rust
     * use specializer::AsyncSpecializerBorrowedParam;
     * use pasts::Executor;
     *
     * async fn specialized<T>(ty: T) -> String
     * where
     *     T: 'static
     * {
     *     let fallback = async |_| "unknown".to_owned();
     *
     *     AsyncSpecializerBorrowedParam::new(ty, fallback)
     *         .specialize_param(async |int: i32| (int * 2).to_string())
     *         .specialize_param(async |string: String| string)
     *         .run()
     *         .await
     * }
     *
     * Executor::default().block_on(async {
     *     assert_eq!(specialized(3).await, "6");
     *     assert_eq!(
     *         specialized("Hello world".to_string()).await,
     *         "Hello world",
     *     );
     *     assert_eq!(specialized(()).await, "unknown");
     * });
     * ``` */
    #[inline]
    pub fn specialize_param<P>(
        self,
        f: impl AsyncFnOnce(P) -> U,
    ) -> AsyncSpecializerBorrowedParam<T, U, impl AsyncFnOnce(T) -> U>
    where
        T: CastIdentityBorrowed<P>,
    {
        self.specialize::<P, U>(f)
    }

    /// Specialize on the return type of the closure.
    /*
     * ```rust
     * use specializer::AsyncSpecializerBorrowedParam;
     * use pasts::Executor;
     *
     * async fn specialized<T>(int: i32) -> T
     * where
     *     T: 'static + Default
     * {
     *     let fallback = async |_| -> T { Default::default() };
     *
     *     AsyncSpecializerBorrowedParam::new(int, fallback)
     *         .specialize_return(async |int| -> i32 { int * 2 })
     *         .specialize_return(async |int| -> String { int.to_string() })
     *         .run()
     *         .await
     * }
     *
     * Executor::default().block_on(async {
     *     assert_eq!(specialized::<i32>(3).await, 6);
     *     assert_eq!(specialized::<String>(3).await, "3");
     *     assert_eq!(specialized::<u8>(3).await, 0);
     * });
     * `` */
    #[inline]
    pub fn specialize_return<R>(
        self,
        f: impl AsyncFnOnce(T) -> R,
    ) -> AsyncSpecializerBorrowedParam<T, U, impl AsyncFnOnce(T) -> U>
    where
        R: 'static,
    {
        self.specialize::<T, R>(f)
    }

    /// Specialize on the parameter and the return type of the closure, mapping
    /// the parameter.
    /*
     * ```rust
     * use std::convert;
     *
     * use specializer::AsyncSpecializerBorrowedParam;
     * use pasts::Executor;
     *
     * async fn specialized<T, U>(ty: T) -> U
     * where
     *     T: 'static,
     *     U: 'static + From<T>,
     * {
     *     AsyncSpecializerBorrowedParam::new(ty, async |ty| ty.into())
     *         .specialize(async |int: i32| -> i32 { int * 2 })
     *         .specialize_map_param(
     *             async |int: u8| int * 3,
     *             async |ty| ty.into(),
     *         )
     *         .run()
     *         .await
     * }
     *
     * Executor::default().block_on(async {
     *     assert_eq!(specialized::<i16, i32>(3).await, 3);
     *     assert_eq!(specialized::<i32, i32>(3).await, 6);
     *     assert_eq!(specialized::<u8, i32>(3).await, 9);
     * });
     * `` */
    #[inline]
    pub fn specialize_map_param<P>(
        self,
        p: impl AsyncFnOnce(P) -> P,
        f: impl AsyncFnOnce(T) -> U,
    ) -> AsyncSpecializerBorrowedParam<T, U, impl AsyncFnOnce(T) -> U>
    where
        T: CastIdentityBorrowed<P>,
        P: CastIdentityBorrowed<T>,
    {
        self.specialize_map::<P, U>(p, f, future::ready)
    }

    /// Specialize on the parameter and the return type of the closure, mapping
    /// the parameter.
    /*
     * ```rust
     * use std::convert;
     *
     * use specializer::AsyncSpecializerBorrowedParam;
     * use pasts::Executor;
     *
     * async fn specialized<T, U>(ty: T) -> U
     * where
     *     T: 'static,
     *     U: 'static + From<T>,
     * {
     *     AsyncSpecializerBorrowedParam::new(ty, async |ty| ty.into())
     *         .specialize_map_return(
     *             async |ty| ty.into(),
     *             async |int: i16| int * 2,
     *         )
     *         .specialize_map_param(
     *             async |int: u8| int * 3,
     *             async |ty| ty.into(),
     *         )
     *         .run()
     *         .await
     * }
     *
     * Executor::default().block_on(async {
     *     assert_eq!(specialized::<i16, i32>(3).await, 3);
     *     assert_eq!(specialized::<i8, i16>(3).await, 6);
     *     assert_eq!(specialized::<u8, i32>(3).await, 9);
     * });
     * ``` */
    #[inline]
    pub fn specialize_map_return<R>(
        self,
        f: impl AsyncFnOnce(T) -> U,
        r: impl AsyncFnOnce(R) -> R,
    ) -> AsyncSpecializerBorrowedParam<T, U, impl AsyncFnOnce(T) -> U>
    where
        R: 'static,
    {
        self.specialize_map::<T, R>(future::ready, f, r)
    }

    /// Run the specializer.
    #[inline]
    pub async fn run(self) -> U {
        (self.1)(self.0).await
    }
}

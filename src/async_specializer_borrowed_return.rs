use core::{any::TypeId, future, marker::PhantomData};

use crate::CastIdentityBorrowed;

/// Async specialized behavior runner (Owned -> Borrowed)
#[derive(Debug)]
pub struct AsyncSpecializerBorrowedReturn<T, U, F>(
    T,
    F,
    PhantomData<fn(T) -> U>,
);

impl<T, U, F> AsyncSpecializerBorrowedReturn<T, U, F>
where
    F: AsyncFnOnce(T) -> U,
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
    /// use pasts::Executor;
    /// use specializer::{CastIdentityBorrowed, AsyncSpecializerBorrowedReturn};
    ///
    /// async fn specialized<'a, T, U>(a: T, b: &'a u32)
    ///     -> Option<&'a U>
    /// where
    ///     T: 'static,
    ///     U: 'static,
    /// {
    ///     AsyncSpecializerBorrowedReturn::new(a, async |_ty| None)
    ///         .specialize(async |int: i32| -> Option<&i32> {
    ///             Some(&42)
    ///         })
    ///         .specialize(async |int: u32| -> Option<&u32> {
    ///             Some(&*b)
    ///         })
    ///         .run()
    ///         .await
    /// }
    ///
    /// Executor::default().block_on(async {
    ///     assert_eq!(specialized::<i32, i32>(3, &5).await, Some(&42));
    ///     assert_eq!(specialized::<u32, u32>(3, &5).await, Some(&5));
    ///     assert_eq!(specialized::<(), u32>((), &5).await, None);
    /// })
    /// ```
    #[inline]
    pub fn specialize<P, R>(
        self,
        f: impl AsyncFnOnce(P) -> R,
    ) -> AsyncSpecializerBorrowedReturn<T, U, impl AsyncFnOnce(T) -> U>
    where
        P: 'static,
        R: CastIdentityBorrowed<U>,
    {
        let AsyncSpecializerBorrowedReturn(ty, fallback, phantom_data) = self;
        let f = async |t: T| -> U {
            if <R as CastIdentityBorrowed<U>>::is_same()
                && TypeId::of::<T>() == TypeId::of::<P>()
            {
                let param = crate::cast_identity::<T, P>(t).unwrap();

                return crate::cast_identity_borrowed::<R, U>(f(param).await)
                    .unwrap();
            }

            fallback(t).await
        };

        AsyncSpecializerBorrowedReturn(ty, f, phantom_data)
    }

    /// Specialize on the parameter and the return type of the closure, mapping
    /// both.
    ///
    /// ```rust
    /// use std::future;
    ///
    /// use pasts::Executor;
    /// use specializer::AsyncSpecializerBorrowedReturn;
    ///
    /// async fn specialized<'a, T, U>(a: T, b: &'a U) -> &'a U
    /// where
    ///     T: 'static + Clone,
    ///     U: 'static + From<T> + From<u8>,
    /// {
    ///     let to = async |ty: T| -> &U { b };
    ///
    ///     AsyncSpecializerBorrowedReturn::new(a, to)
    ///         .specialize(async |int: i32| -> &i32 {
    ///             &42
    ///         })
    ///         .specialize_map(
    ///             async |int: u8| int * 3,
    ///             to,
    ///             future::ready::<&U>,
    ///         )
    ///         .run()
    ///         .await
    /// }
    ///
    /// Executor::default().block_on(async {
    ///     assert_eq!(specialized::<u8, i32>(3, &5).await, &mut 5);
    ///     assert_eq!(specialized::<i32, i32>(3, &5).await, &mut 42);
    ///     assert_eq!(specialized::<i16, i32>(3, &5).await, &mut 5);
    /// });
    /// ```
    #[inline]
    pub fn specialize_map<P, R>(
        self,
        p: impl AsyncFnOnce(P) -> P,
        f: impl AsyncFnOnce(T) -> U,
        r: impl AsyncFnOnce(R) -> R,
    ) -> AsyncSpecializerBorrowedReturn<T, U, impl AsyncFnOnce(T) -> U>
    where
        P: 'static,
        R: CastIdentityBorrowed<U>,
        U: CastIdentityBorrowed<R>,
    {
        let AsyncSpecializerBorrowedReturn(ty, fallback, phantom_data) = self;
        let f = async |t: T| -> U {
            if <U as CastIdentityBorrowed<R>>::is_same()
                && TypeId::of::<T>() == TypeId::of::<P>()
            {
                let param = crate::cast_identity::<T, P>(t).unwrap();
                let param =
                    crate::cast_identity::<P, T>(p(param).await).unwrap();
                let ret = crate::cast_identity_borrowed::<U, R>(f(param).await)
                    .unwrap();

                return crate::cast_identity_borrowed::<R, U>(r(ret).await)
                    .unwrap();
            }

            fallback(t).await
        };

        AsyncSpecializerBorrowedReturn(ty, f, phantom_data)
    }

    /// Specialize on the parameter of the closure.
    ///
    /// ```rust
    /// use pasts::Executor;
    /// use specializer::{CastIdentityBorrowed, AsyncSpecializerBorrowedReturn};
    ///
    /// async fn specialized<'a, T, U>(a: T, b: &'a U) -> Option<&'a U>
    /// where
    ///     T: 'static,
    ///     U: 'static,
    /// {
    ///     AsyncSpecializerBorrowedReturn::new(a, async |_ty| None)
    ///         .specialize_param(async |int: u32| Some(b))
    ///         .run()
    ///         .await
    /// }
    ///
    /// Executor::default().block_on(async {
    ///     assert_eq!(specialized::<i32, i32>(3, &5).await, None);
    ///     assert_eq!(specialized::<u32, u32>(3, &5).await, Some(&5));
    /// });
    /// ```
    #[inline]
    pub fn specialize_param<P>(
        self,
        f: impl AsyncFnOnce(P) -> U,
    ) -> AsyncSpecializerBorrowedReturn<T, U, impl AsyncFnOnce(T) -> U>
    where
        P: 'static,
    {
        self.specialize::<P, U>(f)
    }

    /// Specialize on the return type of the closure.
    ///
    /// ```rust
    /// use pasts::Executor;
    /// use specializer::{CastIdentityBorrowed, AsyncSpecializerBorrowedReturn};
    ///
    /// async fn specialized<'a, U>(a: i32, b: &'a u32) -> Option<&'a U>
    /// where
    ///     U: 'static,
    /// {
    ///     AsyncSpecializerBorrowedReturn::new(a, async |_ty| None)
    ///         .specialize_return(async |int| -> Option<&i32> { Some(&42) })
    ///         .specialize_return(async |int| -> Option<&u32> { Some(&*b) })
    ///         .run()
    ///         .await
    /// }
    ///
    /// Executor::default().block_on(async {
    ///     assert_eq!(specialized(3, &5).await, Some(&42i32));
    ///     assert_eq!(specialized(3, &5).await, Some(&5u32));
    ///     assert_eq!(specialized::<u8>(3, &5).await, None);
    /// })
    /// ```
    #[inline]
    pub fn specialize_return<R>(
        self,
        f: impl AsyncFnOnce(T) -> R,
    ) -> AsyncSpecializerBorrowedReturn<T, U, impl AsyncFnOnce(T) -> U>
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
    /// use pasts::Executor;
    /// use specializer::AsyncSpecializerBorrowedReturn;
    ///
    /// async fn specialized<'a, T, U>(a: T, b: &'a U) -> &'a U
    /// where
    ///     T: 'static + Clone,
    ///     U: 'static + From<T> + From<u8>,
    /// {
    ///     let to = async |ty: T| -> &U { b };
    ///
    ///     AsyncSpecializerBorrowedReturn::new(a, to)
    ///         .specialize(async |int: i32| -> &i32 { &42 })
    ///         .specialize_map_param(
    ///             async |int: u8| int * 3,
    ///             to,
    ///         )
    ///         .run()
    ///         .await
    /// }
    ///
    /// Executor::default().block_on(async {
    ///     assert_eq!(specialized::<u8, i32>(3, &5).await, &mut 5);
    ///     assert_eq!(specialized::<i32, i32>(3, &5).await, &mut 42);
    ///     assert_eq!(specialized::<i16, i32>(3, &5).await, &mut 5);
    /// });
    /// ```
    #[inline]
    pub fn specialize_map_param<P>(
        self,
        p: impl AsyncFnOnce(P) -> P,
        f: impl AsyncFnOnce(T) -> U,
    ) -> AsyncSpecializerBorrowedReturn<T, U, impl AsyncFnOnce(T) -> U>
    where
        P: 'static,
    {
        self.specialize_map::<P, U>(p, f, future::ready)
    }

    /// Specialize on the parameter and the return type of the closure, mapping
    /// the parameter.
    ///
    /// ```rust
    /// use std::convert;
    ///
    /// use pasts::Executor;
    /// use specializer::AsyncSpecializerBorrowedReturn;
    ///
    /// async fn specialized<'a, U>(a: i8, b: &'a i32, c: &'a U)
    ///     -> &'a U
    /// where
    ///     U: 'static,
    /// {
    ///     let to = async |ty: i8| -> &U { c };
    ///
    ///     AsyncSpecializerBorrowedReturn::new(a, to)
    ///         .specialize_return(async |int| -> &i8 { &16 })
    ///         .specialize_map_return(
    ///             to,
    ///             async |ty| -> &i32 { b },
    ///         )
    ///         .specialize_map_return(
    ///             async |ty| -> &U { c },
    ///             async |ty| -> &i16 { &15 },
    ///         )
    ///         .run()
    ///         .await
    /// }
    ///
    /// Executor::default().block_on(async {
    ///     let value = 3;
    ///
    ///     assert_eq!(specialized::<i8>(value, &5, &42).await, &16);
    ///     assert_eq!(specialized::<i64>(value, &5, &42).await, &42);
    ///     assert_eq!(specialized::<i32>(value, &5, &42).await, &5);
    ///     assert_eq!(specialized::<i16>(value, &5, &42).await, &15);
    /// });
    /// ```
    #[inline]
    pub fn specialize_map_return<R>(
        self,
        f: impl AsyncFnOnce(T) -> U,
        r: impl AsyncFnOnce(R) -> R,
    ) -> AsyncSpecializerBorrowedReturn<T, U, impl AsyncFnOnce(T) -> U>
    where
        R: CastIdentityBorrowed<U>,
        U: CastIdentityBorrowed<R>,
    {
        self.specialize_map::<T, R>(future::ready, f, r)
    }

    /// Run the specializer.
    #[inline]
    pub async fn run(self) -> U {
        (self.1)(self.0).await
    }
}

use core::{future, marker::PhantomData};

use crate::CastIdentityBorrowed;

/// Specialized behavior runner (Borrowed -> Borrowed)
#[derive(Debug)]
pub struct AsyncSpecializerBorrowed<T, U, F>(T, F, PhantomData<fn(T) -> U>);

impl<T, U, F> AsyncSpecializerBorrowed<T, U, F>
where
    F: AsyncFnOnce(T) -> U,
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
    /// use pasts::Executor;
    /// use specializer::{CastIdentityBorrowed, AsyncSpecializerBorrowed};
    ///
    /// async fn specialized<'a, T, U>(a: &'a mut T, b: &'a u32)
    ///     -> Option<&'a U>
    /// where
    ///     T: 'static,
    ///     U: 'static,
    /// {
    ///     AsyncSpecializerBorrowed::new(a, async |_ty| None)
    ///         .specialize(async |int: &mut i32| -> Option<&i32> {
    ///             Some(&*int)
    ///         })
    ///         .specialize(async |int: &mut u32| -> Option<&u32> {
    ///             Some(&*b)
    ///         })
    ///         .run()
    ///         .await
    /// }
    ///
    /// Executor::default().block_on(async {
    ///     assert_eq!(specialized::<i32, i32>(&mut 3, &5).await, Some(&3));
    ///     assert_eq!(specialized::<u32, u32>(&mut 3, &5).await, Some(&5));
    ///     assert_eq!(specialized::<(), u32>(&mut (), &5).await, None);
    /// })
    /// ```
    #[inline]
    pub fn specialize<P, R>(
        self,
        f: impl AsyncFnOnce(P) -> R,
    ) -> AsyncSpecializerBorrowed<T, U, impl AsyncFnOnce(T) -> U>
    where
        T: CastIdentityBorrowed<P>,
        R: CastIdentityBorrowed<U>,
    {
        let AsyncSpecializerBorrowed(ty, fallback, phantom_data) = self;
        let f = async |t: T| -> U {
            if <R as CastIdentityBorrowed<U>>::is_same()
                && <T as CastIdentityBorrowed<P>>::is_same()
            {
                let param = crate::cast_identity_borrowed::<T, P>(t).unwrap();

                return crate::cast_identity_borrowed::<R, U>(f(param).await)
                    .unwrap();
            }

            fallback(t).await
        };

        AsyncSpecializerBorrowed(ty, f, phantom_data)
    }

    /// Specialize on the parameter and the return type of the closure, mapping
    /// both.
    ///
    /// ```rust
    /// use std::future;
    ///
    /// use pasts::Executor;
    /// use specializer::AsyncSpecializerBorrowed;
    ///
    /// async fn specialized<'a, T, U>(a: &'a mut T, b: &'a U) -> &'a U
    /// where
    ///     T: 'static + Clone,
    ///     U: 'static + From<T> + From<u8>,
    /// {
    ///     let to = async |ty: &mut T| -> &U { b };
    ///
    ///     AsyncSpecializerBorrowed::new(a, to)
    ///         .specialize(async |int: &mut i32| -> &i32 {
    ///             *int *= 2;
    ///             &*int
    ///         })
    ///         .specialize_map(
    ///             async |int: &mut u8| {
    ///                 *int *= 3;
    ///                 int
    ///             },
    ///             to,
    ///             future::ready::<&U>,
    ///         )
    ///         .run()
    ///         .await
    /// }
    ///
    /// Executor::default().block_on(async {
    ///     let mut value = 3;
    ///
    ///     assert_eq!(specialized::<u8, i32>(&mut value, &5).await, &mut 5);
    ///     assert_eq!(value, 9);
    ///
    ///     let mut value = 3;
    ///
    ///     assert_eq!(specialized::<i32, i32>(&mut value, &5).await, &mut 6);
    ///     assert_eq!(value, 6);
    ///
    ///     let mut value = 3;
    ///
    ///     assert_eq!(specialized::<i16, i32>(&mut value, &5).await, &mut 5);
    ///     assert_eq!(value, 3);
    /// });
    /// ```
    #[inline]
    pub fn specialize_map<P, R>(
        self,
        p: impl AsyncFnOnce(P) -> P,
        f: impl AsyncFnOnce(T) -> U,
        r: impl AsyncFnOnce(R) -> R,
    ) -> AsyncSpecializerBorrowed<T, U, impl AsyncFnOnce(T) -> U>
    where
        T: CastIdentityBorrowed<P>,
        P: CastIdentityBorrowed<T>,
        R: CastIdentityBorrowed<U>,
        U: CastIdentityBorrowed<R>,
    {
        let AsyncSpecializerBorrowed(ty, fallback, phantom_data) = self;
        let f = async |t: T| -> U {
            if <U as CastIdentityBorrowed<R>>::is_same()
                && <T as CastIdentityBorrowed<P>>::is_same()
            {
                let param = crate::cast_identity_borrowed::<T, P>(t).unwrap();
                let param =
                    crate::cast_identity_borrowed::<P, T>(p(param).await)
                        .unwrap();
                let ret = crate::cast_identity_borrowed::<U, R>(f(param).await)
                    .unwrap();

                return crate::cast_identity_borrowed::<R, U>(r(ret).await)
                    .unwrap();
            }

            fallback(t).await
        };

        AsyncSpecializerBorrowed(ty, f, phantom_data)
    }

    /// Specialize on the parameter of the closure.
    ///
    /// ```rust
    /// use pasts::Executor;
    /// use specializer::{CastIdentityBorrowed, AsyncSpecializerBorrowed};
    ///
    /// async fn specialized<'a, T, U>(a: &'a mut T, b: &'a U) -> Option<&'a U>
    /// where
    ///     T: 'static,
    ///     U: 'static,
    /// {
    ///     AsyncSpecializerBorrowed::new(a, async |_ty| None)
    ///         .specialize_param(async |int: &mut u32| Some(b))
    ///         .run()
    ///         .await
    /// }
    ///
    /// Executor::default().block_on(async {
    ///     assert_eq!(specialized::<i32, i32>(&mut 3, &5).await, None);
    ///     assert_eq!(specialized::<u32, u32>(&mut 3, &5).await, Some(&5));
    /// });
    /// ```
    #[inline]
    pub fn specialize_param<P>(
        self,
        f: impl AsyncFnOnce(P) -> U,
    ) -> AsyncSpecializerBorrowed<T, U, impl AsyncFnOnce(T) -> U>
    where
        T: CastIdentityBorrowed<P>,
    {
        self.specialize::<P, U>(f)
    }

    /// Specialize on the return type of the closure.
    ///
    /// ```rust
    /// use pasts::Executor;
    /// use specializer::{CastIdentityBorrowed, AsyncSpecializerBorrowed};
    ///
    /// async fn specialized<'a, U>(a: &'a mut i32, b: &'a u32) -> Option<&'a U>
    /// where
    ///     U: 'static,
    /// {
    ///     AsyncSpecializerBorrowed::new(a, async |_ty| None)
    ///         .specialize_return(async |int| -> Option<&i32> { Some(&*int) })
    ///         .specialize_return(async |int| -> Option<&u32> { Some(&*b) })
    ///         .run()
    ///         .await
    /// }
    ///
    /// Executor::default().block_on(async {
    ///     assert_eq!(specialized(&mut 3, &5).await, Some(&3i32));
    ///     assert_eq!(specialized(&mut 3, &5).await, Some(&5u32));
    ///     assert_eq!(specialized::<u8>(&mut 3, &5).await, None);
    /// })
    /// ```
    #[inline]
    pub fn specialize_return<R>(
        self,
        f: impl AsyncFnOnce(T) -> R,
    ) -> AsyncSpecializerBorrowed<T, U, impl AsyncFnOnce(T) -> U>
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
    /// use specializer::AsyncSpecializerBorrowed;
    ///
    /// async fn specialized<'a, T, U>(a: &'a mut T, b: &'a U) -> &'a U
    /// where
    ///     T: 'static + Clone,
    ///     U: 'static + From<T> + From<u8>,
    /// {
    ///     let to = async |ty: &mut T| -> &U { b };
    ///
    ///     AsyncSpecializerBorrowed::new(a, to)
    ///         .specialize(async |int: &mut i32| -> &i32 {
    ///             *int *= 2;
    ///             &*int
    ///         })
    ///         .specialize_map_param(
    ///             async |int: &mut u8| {
    ///                 *int *= 3;
    ///                 int
    ///             },
    ///             to,
    ///         )
    ///         .run()
    ///         .await
    /// }
    ///
    /// Executor::default().block_on(async {
    ///     let mut value = 3;
    ///
    ///     assert_eq!(specialized::<u8, i32>(&mut value, &5).await, &mut 5);
    ///     assert_eq!(value, 9);
    ///
    ///     let mut value = 3;
    ///
    ///     assert_eq!(specialized::<i32, i32>(&mut value, &5).await, &mut 6);
    ///     assert_eq!(value, 6);
    ///
    ///     let mut value = 3;
    ///
    ///     assert_eq!(specialized::<i16, i32>(&mut value, &5).await, &mut 5);
    ///     assert_eq!(value, 3);
    /// });
    /// ```
    #[inline]
    pub fn specialize_map_param<P>(
        self,
        p: impl AsyncFnOnce(P) -> P,
        f: impl AsyncFnOnce(T) -> U,
    ) -> AsyncSpecializerBorrowed<T, U, impl AsyncFnOnce(T) -> U>
    where
        T: CastIdentityBorrowed<P>,
        P: CastIdentityBorrowed<T>,
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
    /// use specializer::AsyncSpecializerBorrowed;
    ///
    /// async fn specialized<'a, U>(a: &'a mut i8, b: &'a i32, c: &'a U)
    ///     -> &'a U
    /// where
    ///     U: 'static,
    /// {
    ///     let to = async |ty: &mut i8| -> &U {
    ///         *ty *= 3;
    ///         c
    ///     };
    ///
    ///     AsyncSpecializerBorrowed::new(a, to)
    ///         .specialize_return(async |int| -> &i8 {
    ///             *int *= 2;
    ///             &*int
    ///         })
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
    ///     let mut value = 3;
    ///
    ///     assert_eq!(specialized::<i8>(&mut value, &5, &42).await, &6);
    ///     assert_eq!(value, 6);
    ///
    ///     let mut value = 3;
    ///
    ///     assert_eq!(specialized::<i64>(&mut value, &5, &42).await, &42);
    ///     assert_eq!(value, 9);
    ///
    ///     let mut value = 3;
    ///
    ///     assert_eq!(specialized::<i32>(&mut value, &5, &42).await, &5);
    ///     assert_eq!(value, 9);
    ///
    ///     let mut value = 3;
    ///
    ///     assert_eq!(specialized::<i16>(&mut value, &5, &42).await, &15);
    ///     assert_eq!(value, 3);
    /// });
    /// ```
    #[inline]
    pub fn specialize_map_return<R>(
        self,
        f: impl AsyncFnOnce(T) -> U,
        r: impl AsyncFnOnce(R) -> R,
    ) -> AsyncSpecializerBorrowed<T, U, impl AsyncFnOnce(T) -> U>
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

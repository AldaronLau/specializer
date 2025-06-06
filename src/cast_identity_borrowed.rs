use core::{any::TypeId, pin::Pin, task::Poll};

/// Identity cast on a borrowed type
///
/// Default implementation always fails the cast operation (`cast_identity()`
/// returns [`None`], and `is_same()` returns [`false`]).
///
/// ```rust
/// use core::any::TypeId;
///
/// use specializer::CastIdentityBorrowed;
///
/// #[derive(Debug, PartialEq)]
/// enum MyThings<'a, T> {
///     Nothing,
///     Ref(&'a T),
///     Mut(&'a mut T),
///     Owned(T),
/// }
///
/// impl<'a, T, U> CastIdentityBorrowed<MyThings<'a, U>> for MyThings<'a, T>
/// where
///     T: 'static,
///     U: 'static,
/// {
///     fn cast_identity(self) -> Option<MyThings<'a, U>> {
///         Some(match self {
///             MyThings::Nothing => MyThings::Nothing,
///             MyThings::Ref(thing) => {
///                 MyThings::Ref(specializer::cast_identity_ref(thing)?)
///             }
///             MyThings::Mut(thing) => {
///                 MyThings::Mut(specializer::cast_identity_mut(thing)?)
///             }
///             MyThings::Owned(thing) => {
///                 MyThings::Owned(specializer::cast_identity(thing)?)
///             }
///         })
///     }
///
///     #[inline(always)]
///     fn is_same() -> bool {
///         TypeId::of::<T>() == TypeId::of::<U>()
///     }
/// }
///
/// fn only_u32_things<T>(things: MyThings<'_, T>) -> Option<MyThings<'_, u32>>
/// where
///     T: 'static
/// {
///     specializer::cast_identity_borrowed(things)
/// }
///
/// assert_eq!(
///     only_u32_things(MyThings::Mut(&mut 42u32)),
///     Some(MyThings::Mut(&mut 42)),
/// );
/// assert_eq!(
///     only_u32_things(MyThings::Ref(&42u32)),
///     Some(MyThings::Ref(&42)),
/// );
/// assert_eq!(
///     only_u32_things(MyThings::Owned(42u32)),
///     Some(MyThings::Owned(42)),
/// );
/// assert_eq!(
///     only_u32_things(MyThings::<u32>::Nothing),
///     Some(MyThings::Nothing),
/// );
///
/// assert!(only_u32_things(MyThings::Mut(&mut 42i32)).is_none());
/// assert!(only_u32_things(MyThings::Ref(&42i32)).is_none());
/// assert!(only_u32_things(MyThings::Owned(42i32)).is_none());
/// assert!(only_u32_things(MyThings::<i32>::Nothing).is_none());
/// ```
pub trait CastIdentityBorrowed<U>: Sized {
    /// Attempt to cast `self` to `U`.
    fn cast_identity(self) -> Option<U> {
        None
    }

    /// Return true if `Self` type is the same as type `U`.
    fn is_same() -> bool {
        false
    }
}

impl<'a, T, U> CastIdentityBorrowed<&'a U> for &'a T
where
    T: 'static,
    U: 'static,
{
    fn cast_identity(self) -> Option<&'a U> {
        crate::cast_identity_ref(self)
    }

    #[inline(always)]
    fn is_same() -> bool {
        TypeId::of::<U>() == TypeId::of::<T>()
    }
}

impl<'a, T, U> CastIdentityBorrowed<&'a mut U> for &'a mut T
where
    T: 'static,
    U: 'static,
{
    fn cast_identity(self) -> Option<&'a mut U> {
        crate::cast_identity_mut(self)
    }

    #[inline(always)]
    fn is_same() -> bool {
        TypeId::of::<U>() == TypeId::of::<T>()
    }
}

impl<'a, T, U> CastIdentityBorrowed<Pin<&'a U>> for Pin<&'a T>
where
    T: 'static + Unpin,
    U: 'static + Unpin,
{
    fn cast_identity(self) -> Option<Pin<&'a U>> {
        Some(Pin::new(crate::cast_identity_ref(self.get_ref())?))
    }

    #[inline(always)]
    fn is_same() -> bool {
        TypeId::of::<U>() == TypeId::of::<T>()
    }
}

impl<'a, T, U> CastIdentityBorrowed<Pin<&'a mut U>> for Pin<&'a mut T>
where
    T: 'static + Unpin,
    U: 'static + Unpin,
{
    fn cast_identity(self) -> Option<Pin<&'a mut U>> {
        Some(Pin::new(crate::cast_identity_mut(self.get_mut())?))
    }

    #[inline(always)]
    fn is_same() -> bool {
        TypeId::of::<U>() == TypeId::of::<T>()
    }
}

impl<T, U> CastIdentityBorrowed<Option<U>> for Option<T>
where
    T: CastIdentityBorrowed<U>,
{
    fn cast_identity(self) -> Option<Option<U>> {
        Some(if let Some(inner) = self {
            Some(crate::cast_identity_borrowed(inner)?)
        } else {
            None
        })
    }

    #[inline(always)]
    fn is_same() -> bool {
        <T as CastIdentityBorrowed<U>>::is_same()
    }
}

impl<T, U> CastIdentityBorrowed<Poll<U>> for Poll<T>
where
    T: CastIdentityBorrowed<U>,
{
    fn cast_identity(self) -> Option<Poll<U>> {
        Some(if let Poll::Ready(inner) = self {
            Poll::Ready(crate::cast_identity_borrowed(inner)?)
        } else {
            Poll::Pending
        })
    }

    #[inline(always)]
    fn is_same() -> bool {
        <T as CastIdentityBorrowed<U>>::is_same()
    }
}

impl<T, U, E, F> CastIdentityBorrowed<Result<U, F>> for Result<T, E>
where
    T: CastIdentityBorrowed<U>,
    E: CastIdentityBorrowed<F>,
{
    fn cast_identity(self) -> Option<Result<U, F>> {
        Some(match self {
            Ok(inner) => Ok(crate::cast_identity_borrowed(inner)?),
            Err(inner) => Err(crate::cast_identity_borrowed(inner)?),
        })
    }

    #[inline(always)]
    fn is_same() -> bool {
        <T as CastIdentityBorrowed<U>>::is_same()
            && <E as CastIdentityBorrowed<F>>::is_same()
    }
}

impl<'a, T, U> CastIdentityBorrowed<&'a U> for &'a mut T {}

impl<'a, T, U> CastIdentityBorrowed<Pin<&'a U>> for &'a mut T {}

impl<'a, T, U> CastIdentityBorrowed<Pin<&'a mut U>> for &'a mut T {}

impl<T, U> CastIdentityBorrowed<Option<U>> for &mut T {}

impl<T, U> CastIdentityBorrowed<Poll<U>> for &mut T {}

impl<T, U, F> CastIdentityBorrowed<Result<U, F>> for &mut T {}

impl<'a, T, U> CastIdentityBorrowed<&'a mut U> for &'a T {}

impl<'a, T, U> CastIdentityBorrowed<Pin<&'a U>> for &'a T {}

impl<'a, T, U> CastIdentityBorrowed<Pin<&'a mut U>> for &'a T {}

impl<T, U> CastIdentityBorrowed<Option<U>> for &T {}

impl<T, U> CastIdentityBorrowed<Poll<U>> for &T {}

impl<T, U, F> CastIdentityBorrowed<Result<U, F>> for &T {}

impl<'a, T, U> CastIdentityBorrowed<&'a U> for Pin<&'a mut T> {}

impl<'a, T, U> CastIdentityBorrowed<Pin<&'a U>> for Pin<&'a mut T> {}

impl<'a, T, U> CastIdentityBorrowed<&'a mut U> for Pin<&'a mut T> {}

impl<T, U> CastIdentityBorrowed<Option<U>> for Pin<&mut T> {}

impl<T, U> CastIdentityBorrowed<Poll<U>> for Pin<&mut T> {}

impl<T, U, F> CastIdentityBorrowed<Result<U, F>> for Pin<&mut T> {}

impl<'a, T, U> CastIdentityBorrowed<&'a mut U> for Pin<&'a T> {}

impl<'a, T, U> CastIdentityBorrowed<&'a U> for Pin<&'a T> {}

impl<'a, T, U> CastIdentityBorrowed<Pin<&'a mut U>> for Pin<&'a T> {}

impl<T, U> CastIdentityBorrowed<Option<U>> for Pin<&T> {}

impl<T, U> CastIdentityBorrowed<Poll<U>> for Pin<&T> {}

impl<T, U, F> CastIdentityBorrowed<Result<U, F>> for Pin<&T> {}

impl<T, U> CastIdentityBorrowed<&mut U> for Option<T> {}

impl<T, U> CastIdentityBorrowed<&U> for Option<T> {}

impl<T, U> CastIdentityBorrowed<Pin<&mut U>> for Option<T> {}

impl<T, U> CastIdentityBorrowed<Pin<&U>> for Option<T> {}

impl<T, U> CastIdentityBorrowed<Poll<U>> for Option<T> {}

impl<T, U, F> CastIdentityBorrowed<Result<U, F>> for Option<T> {}

impl<T, U> CastIdentityBorrowed<&mut U> for Poll<T> {}

impl<T, U> CastIdentityBorrowed<&U> for Poll<T> {}

impl<T, U> CastIdentityBorrowed<Pin<&mut U>> for Poll<T> {}

impl<T, U> CastIdentityBorrowed<Pin<&U>> for Poll<T> {}

impl<T, U> CastIdentityBorrowed<Option<U>> for Poll<T> {}

impl<T, U, F> CastIdentityBorrowed<Result<U, F>> for Poll<T> {}

impl<T, U, E> CastIdentityBorrowed<&mut U> for Result<T, E> {}

impl<T, U, E> CastIdentityBorrowed<&U> for Result<T, E> {}

impl<T, U, E> CastIdentityBorrowed<Pin<&mut U>> for Result<T, E> {}

impl<T, U, E> CastIdentityBorrowed<Pin<&U>> for Result<T, E> {}

impl<T, U, E> CastIdentityBorrowed<Option<U>> for Result<T, E> {}

impl<T, U, E> CastIdentityBorrowed<Poll<U>> for Result<T, E> {}

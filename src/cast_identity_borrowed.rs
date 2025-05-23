/// Identity cast on a borrowed type
///
/// ```rust
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
/// // Specialization for this variant is not required
/// assert_eq!(
///     only_u32_things(MyThings::<i32>::Nothing),
///     Some(MyThings::<u32>::Nothing),
/// );
/// ```
pub trait CastIdentityBorrowed<U> {
    /// Attempt to cast `self` to `U`.
    fn cast_identity(self) -> Option<U>;
}

impl<'a, T, U> CastIdentityBorrowed<&'a U> for &'a T
where
    T: 'static,
    U: 'static,
{
    fn cast_identity(self) -> Option<&'a U> {
        crate::cast_identity_ref(self)
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
}

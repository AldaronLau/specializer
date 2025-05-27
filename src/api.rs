use core::any::Any;

use crate::CastIdentityBorrowed;

/// Attempt to cast owned `T` to `U`.
///
/// Returns `None` if they are not the same type.
///
/// ```rust
/// fn only_string<T: 'static>(t: T) -> Option<String> {
///     specializer::cast_identity::<T, String>(t)
/// }
///
/// assert!(only_string(()).is_none());
/// assert!(only_string(1).is_none());
/// assert!(only_string("Hello").is_none());
/// assert_eq!(only_string("Hello".to_string()).as_deref(), Some("Hello"));
/// ```
#[inline(always)]
pub fn cast_identity<T, U>(ty: T) -> Option<U>
where
    T: 'static,
    U: 'static,
{
    <(dyn Any + 'static)>::downcast_mut::<Option<U>>(&mut Some(ty))?.take()
}

/// Attempt to cast `&T` to `&U`.
///
/// Returns `None` if they are not the same type.
///
/// ```rust
/// fn only_string<T: 'static>(t: &T) -> Option<&String> {
///     specializer::cast_identity_ref::<T, String>(t)
/// }
///
/// assert!(only_string(&()).is_none());
/// assert!(only_string(&1).is_none());
/// assert!(only_string(&"Hello").is_none());
/// assert_eq!(
///     only_string(&"Hello".to_string()).map(|x| x.as_str()),
///     Some("Hello"),
/// );
/// ```
#[inline(always)]
pub fn cast_identity_ref<T, U>(ty: &T) -> Option<&U>
where
    T: 'static,
    U: 'static,
{
    <(dyn Any + 'static)>::downcast_ref::<U>(ty)
}

/// Attempt to cast `&mut T` to `&mut U`.
///
/// Returns `None` if they are not the same type.
///
/// ```rust
/// fn only_string<T: 'static>(t: &mut T) -> Option<&mut String> {
///     specializer::cast_identity_mut::<T, String>(t)
/// }
///
/// assert!(only_string(&mut ()).is_none());
/// assert!(only_string(&mut 1).is_none());
/// assert!(only_string(&mut "Hello").is_none());
/// assert_eq!(
///     only_string(&mut "Hello".to_string()),
///     Some(&mut "Hello".to_string()),
/// );
/// ```
#[inline(always)]
pub fn cast_identity_mut<T, U>(ty: &mut T) -> Option<&mut U>
where
    T: 'static,
    U: 'static,
{
    <(dyn Any + 'static)>::downcast_mut::<U>(ty)
}

/// Attempt to cast borrowed `T` to `U`.
///
/// ```rust
/// fn only_string_ref<T: 'static>(t: &T) -> Option<&String> {
///     specializer::cast_identity_borrowed::<&T, &String>(t)
/// }
///
/// assert!(only_string_ref(&()).is_none());
/// assert!(only_string_ref(&1).is_none());
/// assert!(only_string_ref(&"Hello").is_none());
/// assert_eq!(
///     only_string_ref(&"Hello".to_string()).map(|x| x.as_str()),
///     Some("Hello"),
/// );
///
/// fn only_string_mut<T: 'static>(t: &mut T) -> Option<&mut String> {
///     specializer::cast_identity_borrowed::<&mut T, &mut String>(t)
/// }
///
/// assert!(only_string_mut(&mut ()).is_none());
/// assert!(only_string_mut(&mut 1).is_none());
/// assert!(only_string_mut(&mut "Hello").is_none());
/// assert_eq!(
///     only_string_mut(&mut "Hello".to_string()),
///     Some(&mut "Hello".to_string()),
/// );
/// ```
#[inline(always)]
pub fn cast_identity_borrowed<T, U>(ty: T) -> Option<U>
where
    T: CastIdentityBorrowed<U>,
{
    T::is_same().then(|| T::cast_identity(ty)).flatten()
}

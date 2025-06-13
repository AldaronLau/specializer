//! Limited safe specialization on stable Rust with builder-like pattern
//!
//! # Types of Specialization
//!
//! There are two types of specialization:
//!  - Specializing on types (example: special behavior for a generic when the
//!    generic type is `Arc<str>` or some other type) - what's implemented in
//!    this crate
//!  - Specializing on traits (example: special behavior if the generic type
//!    implements `ToString` or some other trait) - requires nightly
//!    specialization feature
//!
//! # Getting Started
//!
//! For the simplest example see [`Specializer::specialize_param()`].
//!
//! The other types may be required depending on your use case:
//!
//! | Async | Takes    | Returns  | Type                               |
//! |-------|----------|----------|------------------------------------|
//! | False | Owned    | Owned    | [`Specializer`]                    |
//! | False | Owned    | Borrowed | [`SpecializerBorrowedReturn`]      |
//! | False | Borrowed | Owned    | [`SpecializerBorrowedParam`]       |
//! | False | Borrowed | Borrowed | [`SpecializerBorrowed`]            |
//! | True  | Owned    | Owned    | [`AsyncSpecializer`]               |
//! | True  | Owned    | Borrowed | [`AsyncSpecializerBorrowedReturn`] |
//! | True  | Borrowed | Owned    | [`AsyncSpecializerBorrowedParam`]  |
//! | True  | Borrowed | Borrowed | [`AsyncSpecializerBorrowed`]       |
//!
//! ## Borrowing
//!
//! You can specialize on borrowed types using the `*SpecializerBorrowed*`
//! specializers as long as the borrowed types implement
//! [`CastIdentityBorrowed`], which is automatically implemented for `&T` and
//! `&mut T`, `where T: 'static`.

#![doc(
    html_logo_url = "https://ardaku.github.io/mm/logo.svg",
    html_favicon_url = "https://ardaku.github.io/mm/icon.svg"
)]
#![no_std]
#![forbid(unsafe_code)]
#![warn(
    anonymous_parameters,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences
)]
#![deny(
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_html_tags,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,
    rustdoc::unescaped_backticks,
    rustdoc::redundant_explicit_links
)]

mod api;
mod async_specializer;
mod async_specializer_borrowed;
mod async_specializer_borrowed_param;
mod async_specializer_borrowed_return;
mod cast_identity_borrowed;
mod specializer;
mod specializer_borrowed;
mod specializer_borrowed_param;
mod specializer_borrowed_return;

pub use self::{
    api::{
        cast_identity, cast_identity_borrowed, cast_identity_mut,
        cast_identity_ref,
    },
    async_specializer::AsyncSpecializer,
    async_specializer_borrowed::AsyncSpecializerBorrowed,
    async_specializer_borrowed_param::AsyncSpecializerBorrowedParam,
    async_specializer_borrowed_return::AsyncSpecializerBorrowedReturn,
    cast_identity_borrowed::CastIdentityBorrowed,
    specializer::Specializer,
    specializer_borrowed::SpecializerBorrowed,
    specializer_borrowed_param::SpecializerBorrowedParam,
    specializer_borrowed_return::SpecializerBorrowedReturn,
};

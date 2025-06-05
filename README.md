# Specializer

[![tests](https://github.com/AldaronLau/specializer/actions/workflows/ci.yml/badge.svg)](https://github.com/AldaronLau/specializer/actions/workflows/ci.yml)
[![GitHub commit activity](https://img.shields.io/github/commit-activity/y/AldaronLau/specializer)](https://github.com/AldaronLau/specializer)
[![GitHub contributors](https://img.shields.io/github/contributors/AldaronLau/specializer)](https://github.com/AldaronLau/specializer/graphs/contributors)  
[![Crates.io](https://img.shields.io/crates/v/specializer)](https://crates.io/crates/specializer)
[![Crates.io](https://img.shields.io/crates/d/specializer)](https://crates.io/crates/specializer)
[![Crates.io (recent)](https://img.shields.io/crates/dr/specializer)](https://crates.io/crates/specializer)  
[![Crates.io](https://img.shields.io/crates/l/specializer)](https://github.com/search?q=repo%3AAldaronLau%2Fspecializer+path%3A**%2FLICENSE*&type=code)
[![Docs.rs](https://docs.rs/specializer/badge.svg)](https://docs.rs/specializer/)
 
Limited safe specialization on stable Rust with builder-like pattern 

Check out the [documentation] for examples.

### Features

 - Functions to do a fallible cast to/from generics/concrete types
 - Supports casting with `'static` types and borrowed types
 - Sync/async builders for chaining type specialization
 - No-std/no-alloc
 - No unsafe

## MSRV

The current MSRV is Rust 1.85.

Any future MSRV updates will follow the [Ardaku MSRV guidelines].

## Alternatives

 - [castaway](https://crates.io/crates/castaway)
 - [try-specialize](https://crates.io/crates/try-specialize)
 - [coe-rs](https://crates.io/crates/coe-rs)
 - [downcast-rs](https://crates.io/crates/downcast-rs)
 - [syllogism](https://crates.io/crates/syllogism)
 - [identity\_cast](https://crates.io/crates/identity_cast)

## License

Copyright © 2025 The Specializer Contributors.

Licensed under any of
 - Apache License, Version 2.0, ([LICENSE\_APACHE] or
   <https://www.apache.org/licenses/LICENSE-2.0>)
 - Boost Software License, Version 1.0, ([LICENSE\_BOOST] or
   <https://www.boost.org/LICENSE_1_0.txt>)
 - MIT License, ([LICENSE\_MIT] or <https://mit-license.org/>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as described above, without any additional terms or conditions.

## Help

If you want help using or contributing to this library, feel free to send me an
email at <aldaronlau@gmail.com>.

[Ardaku MSRV guidelines]: https://github.com/ardaku/.github/blob/v1/profile/MSRV.md
[LICENSE\_APACHE]: https://github.com/AldaronLau/specializer/blob/v0/LICENSE_APACHE
[LICENSE\_BOOST]: https://github.com/AldaronLau/specializer/blob/v0/LICENSE_BOOST
[LICENSE\_MIT]: https://github.com/AldaronLau/specializer/blob/v0/LICENSE_MIT
[documentation]: https://docs.rs/specializer

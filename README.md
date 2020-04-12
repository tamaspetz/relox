# relox

[![Latest Version]][crates.io]
[![Documentation]][docs.rs]
[![License]]()
[![Latest Release]][github.com.release]
[![Build Status (develop)]][travis.ci]
[![Code Coverage (develop)]][codecov.io]

[Latest Version]: https://img.shields.io/crates/v/relox.svg?style=flat-square&color=blue
[crates.io]: https://crates.io/crates/relox
[Documentation]: https://img.shields.io/badge/docs.rs-relox-blue?style=flat-square
[Latest Release]: https://img.shields.io/github/release/tamaspetz/relox.svg?style=flat-square&logo=github
[github.com.release]: https://github.com/tamaspetz/relox/releases/latest
[docs.rs]: https://docs.rs/relox
[License]: https://img.shields.io/crates/l/relox?style=flat-square
[Build Status (develop)]: https://img.shields.io/travis/com/tamaspetz/relox/develop?style=flat-square&logo=travis%20ci
[travis.ci]: https://travis-ci.com/tamaspetz/relox
[Code Coverage (develop)]: https://img.shields.io/codecov/c/gh/tamaspetz/relox?style=flat-square&logo=codecov
[codecov.io]: https://codecov.io/gh/tamaspetz/relox

# Compress and decompress ELF32 relocation sections

This crate can be used to compress ELF32 relocation sections post-link time.
It also provides a decompressing method which can be used during relocation.

The approach might be useful for embedded system if a relocation section
uses too much static storage.

Decompressing relocations is a trade-off between used static storage and
CPU time required to process relocations during initialization.

## Compressed section layout for ELF32

```rust
/// ELF32 relocations grouped by relocation type.
struct Elf32CRelGroup {
    // Type of the relocation.
    relocation_type: u8,
    // Number of relocations encoded as ULEB128.
    count: u32,
    // Offsets are encoded as ULEB128.
    // First offset is relative to `base_address`,
    // otherwise offset[i+1] is relative to offset[i].
    offsets: [u32; count],
}

/// A compressed ELF32 relocation section.
struct Elf32CRel {
    // Base address of all the relocations.
    base_address: u32,
    // Number of relocation groups.
    count: u8,
    // Relocation groups.
    groups: [Elf32CRelGroup; count],
}
```

## Recommended usage

On host machines, during post-link time processing,
use `host` feature group.

When targeting embedded devices use either `embedded` or `embedded_minimal`
feature group. The latter one enables `no_bounds_check` and
`no_sanity_check` features to further reduce memory footprint.

### List of optional features

* `compress`: include methods and structures related to compressing.
* `decompress`: include methods and structures related to decompressing.
* `no-std`: do not use standard library.
* `no_bounds_check`: use `unsafe` code instead of bounds-checking variants.
* `no_sanity_check`: do not perform extra sanity checks when processing LEB128
  encodings.

## License

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `relox` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

# vswhere-rs

![GitHub Actions](https://github.com/FaultyRAM/vswhere-rs/actions/workflows/ci.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/vswhere.svg)][Crates.io]
[![Docs.rs](https://docs.rs/vswhere/badge.svg)][Docs.rs]

Provides support for invoking and capturing the output of the [vswhere] utility.

## Background

Starting with the 2017 editions, Visual Studio products support side-by-side installation. This
means that the Windows registry is no longer sufficient for storing information about installed
Visual Studio instances, and a more sophisticated detection method is required. vswhere is an
open-source utility from Microsoft that consumes a COM-based query API and generates output in one
of several formats (text, JSON, XML). Through this crate, Rust code can pass arguments to, invoke
and retrieve output from vswhere in an idiomatic manner.

## Requirements

vswhere-rs requires vswhere 2.7.1 or later. vswhere is bundled with Microsoft Visual Studio
Installer, and can also be obtained from other sources; see [the vswhere wiki] for more
information.

## Usage

Add vswhere-rs to your Cargo.toml:

```toml
[target.'cfg(target_os = "windows")'.dependencies]
vswhere = "^0.2.0"
```

Then pass an instance of one of the selection types to a runner function:

```rust
use vswhere::selection::Modern;

fn main() {
    let mut selection = Modern::new();
    let _ = selection.all(true);
    println!("{}", vswhere::run(&selection).unwrap());
}
```

See [the API documentation][Docs.rs] for more information.

## License

Licensed under either of

* Apache License, Version 2.0,
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

[Crates.io]: https://crates.io/crates/vswhere
[Docs.rs]: https://docs.rs/vswhere
[the vswhere wiki]: https://github.com/Microsoft/vswhere/wiki/Installing
[vswhere]: https://github.com/Microsoft/vswhere

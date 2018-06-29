# vswhere-rs

[![Travis](https://img.shields.io/travis/FaultyRAM/vswhere-rs.svg)][1]
[![AppVeyor](https://img.shields.io/appveyor/ci/FaultyRAM/vswhere-rs.svg)][2]
[![Crates.io](https://img.shields.io/crates/v/vswhere.svg)][3]
[![Docs.rs](https://docs.rs/vswhere/badge.svg)][4]

Provides support for invoking and capturing the output of the [vswhere][5] utility.

## Background

Starting with the 2017 editions, Visual Studio products support side-by-side installation. This
means that the Windows registry is no longer sufficient for storing information about installed
Visual Studio instances, and a more sophisticated detection method is required. vswhere is an
open-source utility from Microsoft that consumes a COM-based query API and generates output in one
of several formats (text, JSON, XML). Through this crate, Rust code can pass arguments to, invoke
and retrieve output from vswhere in an idiomatic manner.

## Requirements

This crate works with vswhere 2.52 or newer. vswhere is installed alongside Visual Studio
Installer, but can also be installed separately or over the top of existing installations (refer to
the vswhere GitHub page for more information). Earlier versions of vswhere will not work since they
do not support the `-utf8` flag, which forces vswhere to generate UTF-8 output (by default the
generated output uses the system default encoding). vswhere is typically installed to
`%ProgramFiles(x86)%\Microsoft Visual Studio\Installer`, a stable location; this crate uses that
path by default, but also supports specifying a custom path e.g. for testing purposes.

## Example

```rust
extern crate vswhere;

use vswhere::Config;

fn main() {
    println!("{:?}", Config::run_vswhere().unwrap());
}
```

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

[1]: https://travis-ci.org/FaultyRAM/vswhere-rs
[2]: https://ci.appveyor.com/project/FaultyRAM/vswhere-rs
[3]: https://crates.io/crates/vswhere
[4]: https://docs.rs/vswhere
[5]: https://github.com/Microsoft/vswhere

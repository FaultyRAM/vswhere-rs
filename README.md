# vswhere-rs
[![Travis CI](https://travis-ci.com/FaultyRAM/vswhere-rs.svg)][1]
[![AppVeyor](https://ci.appveyor.com/api/projects/status/a6p7trkglc90jcd3?retina=true&svg=true)][2]
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

This crate works with vswhere 2.5.2 or newer. Older versions are not supported since they do not
accept the `-utf8` flag, which forces vswhere to generate UTF-8 encoded output (otherwise vswhere
uses the system default encoding). vswhere can be installed in one of three different ways:

* Via [Chocolatey][6] (recommended - package page [here][7]);
* As part of Visual Studio Installer (vswhere.exe will be located in
  `%ProgramFiles(x86)%\Microsoft Visual Studio\Installer` - note that the bundled version tends to
  be outdated);
* [Manually][8], by downloading vswhere.exe to the desired location.

## Example

```rust
extern crate vswhere;

use vswhere::Config;

fn main() {
    println!("{:?}", Config::run_default_path().unwrap());
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

[1]: https://travis-ci.com/FaultyRAM/vswhere-rs
[2]: https://ci.appveyor.com/project/FaultyRAM/vswhere-rs
[3]: https://crates.io/crates/vswhere
[4]: https://docs.rs/vswhere
[5]: https://github.com/Microsoft/vswhere
[6]: https://chocolatey.org
[7]: https://chocolatey.org/packages/vswhere
[8]: https://github.com/Microsoft/vswhere/releases

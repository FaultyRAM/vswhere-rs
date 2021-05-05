// Copyright (c) 2018, 2021 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

//! Provides support for invoking and capturing the output of the vswhere utility.

#![deny(
    clippy::all,
    clippy::pedantic,
    warnings,
    future_incompatible,
    rust_2018_idioms,
    rustdoc,
    unused
)]
#![allow(clippy::must_use_candidate)]

mod sealed {
    pub trait Sealed {}
}

use sealed::Sealed;
use std::{
    fmt::{self, Display, Formatter},
    io::Write,
    process::Command,
    str,
};

/// A trait shared by vswhere queries.
///
/// This sealed trait is an implementation detail, and not intended for use outside of this crate.
pub trait Query: Sealed {
    #[doc(hidden)]
    /// Populates a `Command` with command line arguments generated from a query.
    fn populate_args(&self, cmd: &mut Command);
}

#[derive(Clone, Debug)]
/// Constructs a vswhere query for modern products.
pub struct ModernQuery<'a, 'b, 'c, 'd> {
    all: bool,
    prerelease: bool,
    products: &'a [&'b str],
    requires: &'c [&'d str],
    requires_any: bool,
    version: VersionRange,
}

impl<'a, 'b, 'c, 'd> ModernQuery<'a, 'b, 'c, 'd> {
    const EMPTY_PRODUCTS: &'a [&'b str] = &[];
    const EMPTY_REQUIRES: &'c [&'d str] = &[];

    /// Creates a new invocation builder with default parameters.
    pub const fn new() -> Self {
        Self {
            all: false,
            prerelease: false,
            products: Self::EMPTY_PRODUCTS,
            requires: Self::EMPTY_REQUIRES,
            requires_any: false,
            version: VersionRange::new(),
        }
    }

    /// If `true`, vswhere will include incomplete and/or non-functional instances in its results.
    ///
    /// The default value for this setting is `false`.
    pub fn all(&mut self, value: bool) -> &mut Self {
        self.all = value;
        self
    }

    /// If `true`, vswhere will include prelease instances in its results.
    ///
    /// The default value for this setting is `false`.
    pub fn prerelease(&mut self, value: bool) -> &mut Self {
        self.prerelease = value;
        self
    }

    /// Specifies the product ID allowlist that vswhere should use.
    ///
    /// To include all product IDs, use `&["*"]`.
    ///
    /// The default value for this setting is an empty slice (`&[]`). In this case, vswhere will
    /// use a default allowlist, containing product IDs that correspond to the Community,
    /// Professional, and Enterprise editions of Visual Studio.
    pub fn products(&mut self, value: &'a [&'b str]) -> &mut Self {
        self.products = value;
        self
    }

    /// Specifies the component/workload ID allowlist that vswhere should use.
    ///
    /// The default value for this setting is an empty slice (`&[]`). In this case, vswhere will
    /// not use a component/workload ID allowlist.
    pub fn requires(&mut self, value: &'c [&'d str]) -> &mut Self {
        self.requires = value;
        self
    }

    /// If `true`, and a component/workload ID allowlist is specified via `ModernQuery::requires`,
    /// vswhere will instead return instances that have at least one of the component or workload
    /// IDs in the list.
    ///
    /// The default value for this setting is `false`.
    pub fn requires_any(&mut self, value: bool) -> &mut Self {
        self.requires_any = value;
        self
    }

    /// Specifies a range of versions that vswhere will look for.
    ///
    /// Both the lower and upper bounds are inclusive.
    ///
    /// A value of `None` represents an infinite bound, i.e. a lower bound of `None` returns all
    /// versions up to and including the upper bound, while an upper bound of `None` returns all
    /// versions starting from the lower bound.
    ///
    /// By default, both bounds are `None`. In this case, vswhere will not limit search results
    /// based on version.
    pub fn version(&mut self, lower: Option<Version>, upper: Option<Version>) -> &mut Self {
        self.version.lower = lower;
        self.version.upper = upper;
        self
    }
}

impl<'a, 'b, 'c, 'd> Default for ModernQuery<'a, 'b, 'c, 'd> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, 'b, 'c, 'd> Query for ModernQuery<'a, 'b, 'c, 'd> {
    fn populate_args(&self, cmd: &mut Command) {
        if self.all {
            let _ = cmd.arg("-all");
        }
        if self.prerelease {
            let _ = cmd.arg("-prerelease");
        }
        if !self.requires_any {
            let _ = cmd.arg("-requiresAny");
        }
        if !self.products.is_empty() {
            let _ = cmd.arg("-products");
            let _ = cmd.args(self.products);
        }
        if !self.requires.is_empty() {
            let _ = cmd.arg("-requires");
            let _ = cmd.args(self.requires);
        }
        self.version.populate_args(cmd);
    }
}

#[doc(hidden)]
impl<'a, 'b, 'c, 'd> Sealed for ModernQuery<'a, 'b, 'c, 'd> {}

#[derive(Clone, Copy, Debug)]
struct VersionRange {
    lower: Option<Version>,
    upper: Option<Version>,
}

impl VersionRange {
    const fn new() -> Self {
        Self {
            lower: None,
            upper: None,
        }
    }
}

impl Default for VersionRange {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for VersionRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match (self.lower, self.upper) {
            (Some(lower), Some(upper)) => f.write_fmt(format_args!("{},{}", lower, upper)),
            (Some(lower), None) => f.write_fmt(format_args!("{},", lower)),
            (None, Some(upper)) => f.write_fmt(format_args!(",{}", upper)),
            (None, None) => Ok(()),
        }
    }
}

impl Query for VersionRange {
    fn populate_args(&self, cmd: &mut Command) {
        let mut buffer = [0; 47];
        write!(&mut buffer[..], "{}", self).unwrap();
        let last = buffer
            .iter()
            .position(|&e| e == b'\0')
            .map_or(buffer.len(), usize::from);
        if last > 0 {
            // SAFETY: if `<VersionRange as Display>::fmt` doesn't output a UTF-8 string, we have a
            // very big problem.
            let s = unsafe { str::from_utf8_unchecked(&buffer[..last]) };
            let _ = cmd.args(&["-version", s]);
        }
    }
}

impl Sealed for VersionRange {}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// A version number, in the format `[major].[minor].[revision].[build]`.
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub revision: u16,
    pub build: u16,
}

impl Default for Version {
    fn default() -> Self {
        Self {
            major: 0,
            minor: 0,
            revision: 0,
            build: 0,
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}.{}.{}.{}",
            self.major, self.minor, self.revision, self.build
        ))
    }
}

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

mod args;
mod sealed {
    pub trait Sealed {}
}

use args::{All, Prerelease, Products, Requires, RequiresAny, VersionRange};
pub use args::{ArgCollector, PopulateArgs};
use sealed::Sealed;
use std::{
    fmt::{self, Display, Formatter},
    str,
};

#[derive(Clone, Debug)]
/// Constructs a vswhere query for modern products.
pub struct ModernQuery<'a, 'b, 'c, 'd> {
    all: All,
    prerelease: Prerelease,
    products: Products<'a, 'b>,
    requires: Requires<'c, 'd>,
    requires_any: RequiresAny,
    version: VersionRange,
}

impl<'a, 'b, 'c, 'd> ModernQuery<'a, 'b, 'c, 'd> {
    /// Creates a new invocation builder with default parameters.
    pub const fn new() -> Self {
        Self {
            all: All::new(),
            prerelease: Prerelease::new(),
            products: Products::new(),
            requires: Requires::new(),
            requires_any: RequiresAny::new(),
            version: VersionRange::new(),
        }
    }

    /// If `true`, vswhere will include incomplete and/or non-functional instances in its results.
    ///
    /// The default value for this setting is `false`.
    pub fn all(&mut self, value: bool) -> &mut Self {
        self.all.0 = value;
        self
    }

    /// If `true`, vswhere will include prelease instances in its results.
    ///
    /// The default value for this setting is `false`.
    pub fn prerelease(&mut self, value: bool) -> &mut Self {
        self.prerelease.0 = value;
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
        self.products.0 = value;
        self
    }

    /// Specifies the component/workload ID allowlist that vswhere should use.
    ///
    /// The default value for this setting is an empty slice (`&[]`). In this case, vswhere will
    /// not use a component/workload ID allowlist.
    pub fn requires(&mut self, value: &'c [&'d str]) -> &mut Self {
        self.requires.0 = value;
        self
    }

    /// If `true`, and a component/workload ID allowlist is specified via `ModernQuery::requires`,
    /// vswhere will instead return instances that have at least one of the component or workload
    /// IDs in the list.
    ///
    /// The default value for this setting is `false`.
    pub fn requires_any(&mut self, value: bool) -> &mut Self {
        self.requires_any.0 = value;
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

impl<'a, 'b, 'c, 'd> PopulateArgs for ModernQuery<'a, 'b, 'c, 'd> {
    fn populate_args<C: ArgCollector>(&self, mut cmd: C) {
        self.all.populate_args(&mut cmd);
        self.prerelease.populate_args(&mut cmd);
        self.requires_any.populate_args(&mut cmd);
        self.products.populate_args(&mut cmd);
        self.requires.populate_args(&mut cmd);
        self.version.populate_args(cmd);
    }
}

#[doc(hidden)]
impl<'a, 'b, 'c, 'd> Sealed for ModernQuery<'a, 'b, 'c, 'd> {}

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

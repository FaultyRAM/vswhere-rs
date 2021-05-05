// Copyright (c) 2021 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

//! Selection parameter groups.

use crate::{
    args::{
        All, ArgCollector, PopulateArgs, Prerelease, Products, Requires, RequiresAny, VersionRange,
    },
    Version,
};
use std::path::Path;

#[derive(Clone, Debug)]
/// Selection parameters for modern (side-by-side installable) instances.
pub struct Modern<'a, 'b, 'c, 'd> {
    all: All,
    prerelease: Prerelease,
    products: Products<'a, 'b>,
    requires: Requires<'c, 'd>,
    requires_any: RequiresAny,
    version: VersionRange,
}

impl<'a, 'b, 'c, 'd> Modern<'a, 'b, 'c, 'd> {
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

impl<'a, 'b, 'c, 'd> Default for Modern<'a, 'b, 'c, 'd> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, 'b, 'c, 'd> PopulateArgs for Modern<'a, 'b, 'c, 'd> {
    #[doc(hidden)]
    fn populate_args<C: ArgCollector>(&self, mut cmd: C) {
        self.all.populate_args(&mut cmd);
        self.prerelease.populate_args(&mut cmd);
        self.requires_any.populate_args(&mut cmd);
        self.products.populate_args(&mut cmd);
        self.requires.populate_args(&mut cmd);
        self.version.populate_args(cmd);
    }
}

#[derive(Clone, Debug)]
/// Selection parameters for legacy instances.
pub struct Legacy {
    all: All,
    prerelease: Prerelease,
    version: VersionRange,
}

impl Legacy {
    /// Creates a new invocation builder with default parameters.
    pub const fn new() -> Self {
        Self {
            all: All::new(),
            prerelease: Prerelease::new(),
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

impl Default for Legacy {
    fn default() -> Self {
        Self::new()
    }
}

impl PopulateArgs for Legacy {
    #[doc(hidden)]
    fn populate_args<C: ArgCollector>(&self, mut cmd: C) {
        cmd.arg("-legacy");
        self.all.populate_args(&mut cmd);
        self.prerelease.populate_args(&mut cmd);
        self.version.populate_args(cmd);
    }
}

impl<P: AsRef<Path>> PopulateArgs for P {
    #[doc(hidden)]
    fn populate_args<C: ArgCollector>(&self, mut cmd: C) {
        cmd.arg("-path");
        cmd.arg(self.as_ref());
    }
}

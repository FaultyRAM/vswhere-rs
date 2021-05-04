// Copyright (c) 2018, 2021 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

//! Provides support for invoking and capturing the output of the vswhere utility.

#![forbid(unsafe_code)]
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

#[derive(Clone, Debug)]
/// Constructs a vswhere query for modern products.
pub struct ModernQuery<'a, 'b, 'c, 'd> {
    all: bool,
    prerelease: bool,
    products: Option<&'a [&'b str]>,
    requires: Option<Requires<'c, 'd>>,
}

impl<'a, 'b, 'c, 'd> ModernQuery<'a, 'b, 'c, 'd> {
    /// Creates a new invocation builder with default parameters.
    pub const fn new() -> Self {
        Self {
            all: false,
            prerelease: false,
            products: None,
            requires: None,
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

    /// If `Some`, replaces the default product ID allowlist with a user-provided list.
    ///
    /// To include all product IDs, use `Some(&["*"])`.
    ///
    /// The default allowlist that vswhere uses includes product IDs that correspond to the
    /// Community, Professional, and Enterprise editions of Visual Studio.
    pub fn products(&mut self, value: Option<&'a [&'b str]>) -> &mut Self {
        self.products = value;
        self
    }

    /// If `Some`, vswhere will use the given component/workload ID allowlist to limit the returned
    /// instances.
    ///
    /// The default value for this setting is `None`.
    pub fn requires(&mut self, value: Option<Requires<'c, 'd>>) -> &mut Self {
        self.requires = value;
        self
    }
}

impl<'a, 'b, 'c, 'd> Default for ModernQuery<'a, 'b, 'c, 'd> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
/// A component/workload ID allowlist.
pub enum Requires<'a, 'b> {
    /// An allowlist that includes only instances with at least one of the given component or
    /// workload IDs.
    Any(&'a [&'b str]),
    /// An allowlist that includes only instances with all of the given component or workload IDs.
    All(&'a [&'b str]),
}

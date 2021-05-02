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
/// Controls vswhere invocation via builder-style configuration.
pub struct Config<'a, 'b> {
    all: bool,
    prerelease: bool,
    products: Option<&'a [&'b str]>,
}

impl<'a, 'b> Config<'a, 'b> {
    /// Creates a new invocation builder with default parameters.
    pub const fn new() -> Self {
        Self {
            all: false,
            prerelease: false,
            products: None,
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
}

impl<'a, 'b> Default for Config<'a, 'b> {
    fn default() -> Self {
        Self::new()
    }
}

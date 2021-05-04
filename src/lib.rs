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

mod sealed {
    pub trait Sealed {}
}

use sealed::Sealed;
use std::process::Command;

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
    }
}

#[doc(hidden)]
impl<'a, 'b, 'c, 'd> Sealed for ModernQuery<'a, 'b, 'c, 'd> {}

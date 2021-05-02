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
pub struct Config {
    all: bool,
}

impl Config {
    /// Creates a new invocation builder with default parameters.
    pub const fn new() -> Self {
        Self {
            all: false,
        }
    }

    /// If `true`, vswhere will include incomplete and/or non-functional instances in its results.
    ///
    /// The default value for this setting is `false`.
    pub fn all(&mut self, value: bool) -> &mut Self {
        self.all = value;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

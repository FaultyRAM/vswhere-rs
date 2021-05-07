// Copyright (c) 2021 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use crate::Version;
use std::{ffi::OsStr, io::Write, process::Command, str};

/// A trait shared by types that collect command line arguments.
///
/// This is an implementation detail. Do not implement it outside of this crate.
pub trait ArgCollector {
    #[doc(hidden)]
    fn arg<S: AsRef<OsStr>>(&mut self, arg: S);

    #[doc(hidden)]
    fn args<I, S>(&mut self, args: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;
}

impl<C: ArgCollector> ArgCollector for &mut C {
    #[doc(hidden)]
    fn arg<S: AsRef<OsStr>>(&mut self, arg: S) {
        (*self).arg(arg)
    }

    #[doc(hidden)]
    fn args<I, S>(&mut self, args: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        (*self).args(args)
    }
}

impl ArgCollector for Command {
    #[doc(hidden)]
    fn arg<S: AsRef<OsStr>>(&mut self, arg: S) {
        let _ = self.arg(arg);
    }

    #[doc(hidden)]
    fn args<I, S>(&mut self, args: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let _ = self.args(args);
    }
}

#[allow(clippy::module_name_repetitions)]
/// A trait shared by types that generate command line arguments.
///
/// This is an implementation detail. Do not implement it outside of this crate.
pub trait PopulateArgs {
    #[doc(hidden)]
    fn populate_args<C: ArgCollector>(&self, cmd: C);
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct All(pub(crate) bool);

impl All {
    pub(crate) const fn new() -> Self {
        Self(false)
    }
}

impl PopulateArgs for All {
    fn populate_args<C: ArgCollector>(&self, mut cmd: C) {
        if self.0 {
            cmd.arg("-all");
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct Prerelease(pub(crate) bool);

impl Prerelease {
    pub(crate) const fn new() -> Self {
        Self(false)
    }
}

impl PopulateArgs for Prerelease {
    fn populate_args<C: ArgCollector>(&self, mut cmd: C) {
        if self.0 {
            cmd.arg("-prerelease");
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct Products<'a, 'b>(pub(crate) &'a [&'b str]);

impl<'a, 'b> Products<'a, 'b> {
    pub(crate) const fn new() -> Self {
        Self(&[])
    }
}

impl<'a, 'b> PopulateArgs for Products<'a, 'b> {
    fn populate_args<C: ArgCollector>(&self, mut cmd: C) {
        if !self.0.is_empty() {
            cmd.arg("-products");
            cmd.args(self.0);
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct Requires<'a, 'b>(pub(crate) &'a [&'b str]);

impl<'a, 'b> Requires<'a, 'b> {
    pub(crate) const fn new() -> Self {
        Self(&[])
    }
}

impl<'a, 'b> PopulateArgs for Requires<'a, 'b> {
    fn populate_args<C: ArgCollector>(&self, mut cmd: C) {
        if !self.0.is_empty() {
            cmd.arg("-requires");
            cmd.args(self.0);
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct RequiresAny(pub(crate) bool);

impl RequiresAny {
    pub(crate) const fn new() -> Self {
        Self(false)
    }
}

impl PopulateArgs for RequiresAny {
    fn populate_args<C: ArgCollector>(&self, mut cmd: C) {
        if self.0 {
            cmd.arg("-requiresAny");
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct VersionRange {
    pub(crate) lower: Option<Version>,
    pub(crate) upper: Option<Version>,
}

impl VersionRange {
    pub(crate) const fn new() -> Self {
        Self {
            lower: None,
            upper: None,
        }
    }
}

impl PopulateArgs for VersionRange {
    fn populate_args<C: ArgCollector>(&self, mut cmd: C) {
        let mut buffer = [0; 47];
        match (self.lower, self.upper) {
            (Some(lower), Some(upper)) => write!(&mut buffer[..], "{},{}", lower, upper),
            (Some(lower), None) => write!(&mut buffer[..], "{},", lower),
            (None, Some(upper)) => write!(&mut buffer[..], ",{}", upper),
            (None, None) => Ok(()),
        }
        .unwrap();
        let last = buffer
            .iter()
            .position(|&e| e == b'\0')
            .map_or(buffer.len(), usize::from);
        if last > 0 {
            // SAFETY: if `<VersionRange as Display>::fmt` doesn't output a UTF-8 string, we have a
            // very big problem.
            let s = unsafe { str::from_utf8_unchecked(&buffer[..last]) };
            cmd.args(&["-version", s]);
        }
    }
}

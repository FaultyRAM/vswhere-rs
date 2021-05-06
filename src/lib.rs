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

pub mod args;
pub mod selection;

use args::PopulateArgs;
use serde_json::Value;
use std::{
    ffi::OsString,
    fmt::{self, Display, Formatter},
    io,
    os::windows::ffi::OsStringExt,
    path::Path,
    process::Command,
    ptr, slice,
};
use winapi::{
    shared::winerror::S_OK,
    um::{
        combaseapi::CoTaskMemFree,
        knownfolders::FOLDERID_ProgramFilesX86,
        shlobj::SHGetKnownFolderPath,
        shtypes::KNOWNFOLDERID,
        winnt::{PWSTR, WCHAR},
    },
};

/// Invokes vswhere with the given selection parameters.
///
/// This function attempts to run vswhere from the following locations:
///
/// 1. The current working directory;
/// 2. Each folder specified in the `PATH` environment variable;
/// 3. The path where Visual Studio Installer is located
///    (`%ProgramFiles(x86)%\Microsoft Visual Studio\Installer`).
///
/// # Errors
///
/// This function returns an `io::Error` if any of the following occurs:
///
/// * vswhere is not found in any of the supported locations;
/// * vswhere fails to run;
/// * vswhere runs, but exits unsuccessfully.
pub fn run<S: PopulateArgs>(selection: &S) -> io::Result<Value> {
    run_path(selection).or_else(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            run_vs_installer(selection)
        } else {
            Err(e)
        }
    })
}

/// Invokes vswhere either in the current working directory or on the current `PATH`, with the
/// given selection parameters.
///
/// # Errors
///
/// This function returns an `io::Error` if vswhere fails to run or does not successfully exit.
pub fn run_path<S: PopulateArgs>(selection: &S) -> io::Result<Value> {
    run_custom_location("vswhere.exe", selection)
}

/// Invokes vswhere bundled with Visual Studio Installer with the given selection parameters.
///
/// # Errors
///
/// This function returns an `io::Error` if vswhere fails to run or does not successfully exit.
fn run_vs_installer<S: PopulateArgs>(selection: &S) -> io::Result<Value> {
    #[repr(transparent)]
    #[derive(Debug)]
    struct KnownFolderPath(PWSTR);

    impl KnownFolderPath {
        fn new(id: &KNOWNFOLDERID) -> io::Result<Self> {
            let mut path = ptr::null_mut();
            if unsafe { SHGetKnownFolderPath(id, 0, ptr::null_mut(), &mut path) } == S_OK {
                Ok(Self(path))
            } else {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "could not locate the specified known folder",
                ))
            }
        }

        fn as_slice(&self) -> &[WCHAR] {
            unsafe {
                let mut len = 0;
                while *self.0.add(len) != 0 {
                    len += 1;
                }
                slice::from_raw_parts(self.0, len)
            }
        }
    }

    impl Drop for KnownFolderPath {
        fn drop(&mut self) {
            unsafe {
                CoTaskMemFree(self.0.cast());
            }
        }
    }

    KnownFolderPath::new(&FOLDERID_ProgramFilesX86).and_then(|kfp| {
        let path = OsString::from_wide(kfp.as_slice());
        run_custom_location(Path::new(&path).join("vswhere.exe"), selection)
    })
}

/// Invokes vswhere at the specified path with the given selection parameters.
///
/// The specified path must be a path to an executable file.
///
/// # Errors
///
/// This function returns an `io::Error` if vswhere fails to run or does not successfully exit.
pub fn run_custom_location<P: AsRef<Path>, S: PopulateArgs>(
    path: P,
    selection: &S,
) -> io::Result<Value> {
    let mut cmd = Command::new(path.as_ref());
    cmd.args(&["-utf8", "-format", "json"]);
    selection.populate_args(&mut cmd);
    cmd.output().and_then(|output| {
        if output.status.success() {
            Ok(serde_json::from_slice(&output.stdout).expect("vswhere produced invalid JSON"))
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("vswhere ran unsuccessfully (exit code: {})", output.status),
            ))
        }
    })
}

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

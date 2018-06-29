// Copyright (c) 2018 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides an idiomatic wrapper around invoking the vswhere utility.

#![cfg(target_os = "windows")]
#![forbid(warnings)]
#![forbid(future_incompatible)]
#![forbid(rust_2018_idioms)]
#![forbid(unused)]
#![forbid(box_pointers)]
#![forbid(missing_copy_implementations)]
#![forbid(missing_debug_implementations)]
#![forbid(missing_docs)]
#![forbid(trivial_casts)]
#![forbid(trivial_numeric_casts)]
#![forbid(unused_import_braces)]
#![deny(unused_qualifications)]
#![forbid(unused_results)]
#![forbid(variant_size_differences)]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_cargo))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_complexity))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_correctness))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_perf))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_style))]

extern crate serde_json;
extern crate winapi;

use serde_json::Value;
use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::ops::Range;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr;
use std::slice;
use std::str;
use winapi::ctypes::c_void;
use winapi::shared::ntdef::PWSTR;
use winapi::shared::winerror::S_OK;
use winapi::um::combaseapi::CoTaskMemFree;
use winapi::um::knownfolders::FOLDERID_ProgramFilesX86;
use winapi::um::shlobj::SHGetKnownFolderPath;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// The version number of a Visual Studio installation.
pub struct VSVersion {
    major: u16,
    minor: u16,
    revision: u16,
    build: u16,
}

#[derive(Clone, Debug)]
/// Builder-style configuration for a vswhere instance.
pub struct Config {
    prerelease: bool,
    products: Vec<String>,
    requires: Vec<String>,
    requires_any: bool,
    version: Option<Range<VSVersion>>,
    latest: bool,
    custom_path: Option<PathBuf>,
}

/// Safe wrapper around the string object returned from `SHGetKnownFolderPath`.
struct VSWherePath(PWSTR);

impl VSVersion {
    /// Creates a new version number using the given values.
    pub fn new(major: u16, minor: u16, revision: u16, build: u16) -> Self {
        Self {
            major,
            minor,
            revision,
            build,
        }
    }

    /// Returns the major version number.
    pub fn major(self) -> u16 {
        self.major
    }

    /// Returns the major version number.
    pub fn minor(self) -> u16 {
        self.minor
    }

    /// Returns the major version number.
    pub fn revision(self) -> u16 {
        self.revision
    }

    /// Returns the major version number.
    pub fn build(self) -> u16 {
        self.build
    }
}

impl Display for VSVersion {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.major, self.minor, self.revision, self.build
        )
    }
}

impl Config {
    /// Creates a new `Config` instance with default values.
    pub fn new() -> Self {
        Self {
            prerelease: false,
            products: Vec::new(),
            requires: Vec::new(),
            requires_any: false,
            version: None,
            latest: false,
            custom_path: None,
        }
    }

    /// Whether to include pre-release versions of Visual Studio in search results.
    ///
    /// By default this is `false`.
    pub fn find_prerelease_versions(&mut self, prerelease: bool) -> &mut Self {
        self.prerelease = prerelease;
        self
    }

    /// Adds a string to the product ID (Visual Studio edition) whitelist.
    ///
    /// By default the product ID whitelist is empty, in which case it is not used. If the product
    /// ID whitelist is non-empty, versions of Visual Studio without a matching product ID are
    /// excluded from search results.
    pub fn whitelist_product_id<T: ToString>(&mut self, product_id: &T) -> &mut Self {
        self.products.push(product_id.to_string());
        self
    }

    /// Adds a string to the component ID whitelist.
    ///
    /// By default the component ID whitelist is empty, in which case it is not used. If the
    /// component ID whitelist is non-empty, versions of Visual Studio are excluded from search
    /// results based on the filtering method in use (see `Config::require_any_component` for
    /// more information).
    pub fn whitelist_component_id<T: ToString>(&mut self, component_id: &T) -> &mut Self {
        self.requires.push(component_id.to_string());
        self
    }

    /// If `true`, exclude from search results Visual Studio versions that do not provide at least
    /// one whitelisted component. Otherwise if `false` (the default value), exclude from search
    /// results Visual Studio versions that do not provide every whitelisted component.
    ///
    /// This is only meaningful if the component ID whitelist is non-empty, as filtering by
    /// component ID is disabled otherwise.
    pub fn require_any_component(&mut self, require_any: bool) -> &mut Self {
        self.requires_any = require_any;
        self
    }

    /// Excludes Visual Studio installations whose version number falls outside of a given range.
    ///
    /// By default no installations are excluded based on version number.
    pub fn version_number_range(&mut self, range: Range<VSVersion>) -> &mut Self {
        self.version = Some(range);
        self
    }

    /// If `true`, include only the most current and most recently installed versions of Visual
    /// Studio in search results.
    ///
    /// By default this is `false`.
    pub fn only_latest_versions(&mut self, latest: bool) -> &mut Self {
        self.latest = latest;
        self
    }

    /// Specifies a custom path to the vswhere executable.
    ///
    /// The default path is `%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe`,
    /// where `%ProgramFiles(x86)%` is the path returned from `SHGetKnownFolderPath` specifying
    /// `FOLDERID_ProgramFilesX86` as the known folder ID to query. Calling this method overrides
    /// that behaviour and instead simply uses the specified path.
    pub fn vswhere_path<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.custom_path = Some(path.as_ref().to_owned());
        self
    }

    /// Invokes vswhere using the current configuration.
    pub fn run_vswhere(&self) -> io::Result<Value> {
        if let Some(custom_path) = self.custom_path.as_ref() {
            Ok(Command::new(custom_path))
        } else {
            VSWherePath::fetch().map(|p| Command::new(&p.into_pathbuf()))
        }.and_then(|mut cmd| {
            if self.prerelease {
                let _ = cmd.arg("-prerelease");
            }
            let _ = cmd.arg("-products");
            if self.products.is_empty() {
                let _ = cmd.arg("*");
            } else {
                let _ = cmd.args(&self.products);
            }
            if !self.requires.is_empty() {
                let _ = cmd.arg("-requires").args(&self.requires);
            }
            if self.requires_any {
                let _ = cmd.arg("-requiresAny");
            }
            if let Some(version_range) = self.version.as_ref() {
                let _ = cmd.args(&[
                    "-version",
                    &format!("[{},{})", version_range.start, version_range.end),
                ]);
            }
            if self.latest {
                let _ = cmd.arg("-latest");
            }
            cmd.args(&["-format", "json", "-utf8"])
                .output()
                .map(|output| {
                    assert!(output.status.success());
                    let json =
                        str::from_utf8(&output.stdout).expect("vswhere returned invalid UTF-8");
                    serde_json::from_str(json).expect("vswhere returned invalid JSON")
                })
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl VSWherePath {
    /// Returns an object representing the path to vswhere.
    pub(crate) fn fetch() -> io::Result<Self> {
        let mut path = VSWherePath(ptr::null_mut());
        let hres = unsafe {
            SHGetKnownFolderPath(&FOLDERID_ProgramFilesX86, 0, ptr::null_mut(), &mut path.0)
        };
        if hres == S_OK {
            Ok(path)
        } else {
            Err(io::Error::last_os_error())
        }
    }

    /// Consumes `self`, returning a `PathBuf` containing the path to vswhere.
    pub(crate) fn into_pathbuf(self) -> PathBuf {
        unsafe {
            let mut wide_string = self.0;
            let mut len = 0;
            while wide_string.read() != 0 {
                wide_string = wide_string.offset(1);
                len += 1;
            }
            let ws_slice = slice::from_raw_parts(self.0, len);
            let os_string = OsString::from_wide(ws_slice);
            Path::new(&os_string).join("Microsoft Visual Studio/Installer/vswhere.exe")
        }
    }
}

impl Drop for VSWherePath {
    fn drop(&mut self) {
        unsafe {
            CoTaskMemFree(self.0 as *mut c_void);
        }
    }
}

// Copyright (c) 2018 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the
// MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at
// your option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Provides support for invoking and capturing the output of the vswhere utility.

#![cfg(target_os = "windows")]
#![forbid(warnings)]
#![forbid(future_incompatible)]
#![deny(unused)]
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
#![cfg_attr(feature = "cargo-clippy", deny(clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_cargo))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_complexity))]
#![cfg_attr(feature = "cargo-clippy", deny(clippy_correctness))]
#![cfg_attr(feature = "cargo-clippy", deny(clippy_perf))]
#![cfg_attr(feature = "cargo-clippy", forbid(clippy_style))]

extern crate chrono;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;
extern crate url_serde;
extern crate winapi;

use chrono::offset::Utc;
use chrono::DateTime;
use semver::Version;
use serde::de::{Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::iter;
use std::ops::Range;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr;
use std::slice;
use std::str;
use url::Url;
use winapi::ctypes::c_void;
use winapi::shared::ntdef::PWSTR;
use winapi::shared::winerror::S_OK;
use winapi::um::combaseapi::CoTaskMemFree;
use winapi::um::knownfolders::FOLDERID_ProgramFilesX86;
use winapi::um::shlobj::SHGetKnownFolderPath;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// A version number that consists of four integers, widely used within the Windows world.
pub struct FourPointVersion {
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
    version: Option<Range<FourPointVersion>>,
    latest: bool,
    custom_path: Option<PathBuf>,
}

#[cfg_attr(feature = "cargo-clippy", allow(similar_names))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Information about a Visual Studio installation.
pub struct InstallInfo {
    instance_id: String,
    install_date: DateTime<Utc>,
    installation_name: String,
    installation_path: PathBuf,
    installation_version: FourPointVersion,
    product_id: String,
    product_path: PathBuf,
    is_prerelease: bool,
    display_name: String,
    description: String,
    channel_id: String,
    channel_path: PathBuf,
    #[serde(with = "url_serde")]
    channel_uri: Url,
    engine_path: PathBuf,
    #[serde(with = "url_serde")]
    release_notes: Url,
    #[serde(with = "url_serde")]
    third_party_notices: Url,
    update_date: DateTime<Utc>,
    catalog: InstallCatalog,
    properties: InstallProperties,
}

#[cfg_attr(feature = "cargo-clippy", allow(similar_names))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Catalog information for a Visual Studio installation.
pub struct InstallCatalog {
    build_branch: String,
    build_version: FourPointVersion,
    id: String,
    local_build: String,
    manifest_name: String,
    manifest_type: String,
    product_display_version: String,
    product_line: String,
    product_line_version: String,
    product_milestone: String,
    #[serde(deserialize_with = "deserialize_uppercase_bool")]
    #[serde(serialize_with = "serialize_uppercase_bool")]
    product_milestone_is_pre_release: bool,
    product_name: String,
    product_patch_version: String,
    product_pre_release_milestone_suffix: String,
    product_release: String,
    product_semantic_version: Version,
    required_engine_version: FourPointVersion,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Property information for a Visual Studio installation.
pub struct InstallProperties {
    campaign_id: String,
    channel_manifest_id: String,
    nickname: String,
    setup_engine_file_path: PathBuf,
}

/// Safe wrapper around the string object returned from `SHGetKnownFolderPath`.
struct VSWherePath(PWSTR);

fn deserialize_uppercase_bool<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<bool, D::Error> {
    struct UppercaseBoolVisitor;

    impl<'de> Visitor<'de> for UppercaseBoolVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            write!(formatter, r#"a string, either `"True"` or `"False"`"#)
        }

        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            let lower = v.to_lowercase();
            let lower_trim = lower.trim();
            if lower_trim == "true" {
                Ok(true)
            } else if lower_trim == "false" {
                Ok(false)
            } else {
                Err(E::invalid_value(Unexpected::Str(v), &self))
            }
        }
    }

    deserializer.deserialize_str(UppercaseBoolVisitor)
}

#[cfg_attr(feature = "cargo-clippy", allow(trivially_copy_pass_by_ref))]
fn serialize_uppercase_bool<S: Serializer>(
    boolean: &bool,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if *boolean {
        serializer.serialize_str("True")
    } else {
        serializer.serialize_str("False")
    }
}

impl FourPointVersion {
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

impl<'de> Deserialize<'de> for FourPointVersion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FourPointVersionVisitor;

        impl<'de> Visitor<'de> for FourPointVersionVisitor {
            type Value = FourPointVersion;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "one to four 16-bit unsigned integers separated by a period (`.`)"
                )
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                fn parse_number<E: serde::de::Error>(
                    visitor: &FourPointVersionVisitor,
                    chunk: &str,
                ) -> Result<u16, E> {
                    u16::from_str_radix(chunk, 10)
                        .map_err(|_| E::invalid_value(Unexpected::Str(chunk), visitor))
                }
                let iter = v.split('.');
                let len = iter.clone().count();
                if len < 1 || len > 4 {
                    Err(E::invalid_length(len, &self))
                } else {
                    let mut version_getter = iter.chain(iter::repeat("0"));
                    Ok(FourPointVersion::new(
                        parse_number(&self, version_getter.next().unwrap())?,
                        parse_number(&self, version_getter.next().unwrap())?,
                        parse_number(&self, version_getter.next().unwrap())?,
                        parse_number(&self, version_getter.next().unwrap())?,
                    ))
                }
            }
        }

        deserializer.deserialize_str(FourPointVersionVisitor)
    }
}

impl Display for FourPointVersion {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.major, self.minor, self.revision, self.build
        )
    }
}

impl Serialize for FourPointVersion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self)
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

    /// Specifies whether to include pre-release versions of Visual Studio in search results.
    ///
    /// By default this is `false`.
    pub fn find_prerelease_versions(&mut self, prerelease: bool) -> &mut Self {
        self.prerelease = prerelease;
        self
    }

    /// Adds a string to the product ID (Visual Studio edition) whitelist.
    ///
    /// By default the product ID whitelist is empty, which is equivalent to passing `-products *`
    /// to vswhere (retrieves information about every installed product, as opposed to just
    /// Community, Professional and Enterprise). If the product ID whitelist is non-empty, versions
    /// of Visual Studio without a matching product ID are excluded from search results.
    pub fn whitelist_product_id<T: ToString + ?Sized>(&mut self, product_id: &T) -> &mut Self {
        self.products.push(product_id.to_string());
        self
    }

    /// Adds a string to the component ID whitelist.
    ///
    /// By default the component ID whitelist is empty, in which case it is not used. If the
    /// component ID whitelist is non-empty, versions of Visual Studio are excluded from search
    /// results based on the filtering method in use (see `Config::require_any_component` for
    /// more information).
    pub fn whitelist_component_id<T: ToString + ?Sized>(&mut self, component_id: &T) -> &mut Self {
        self.requires.push(component_id.to_string());
        self
    }

    /// Specifies the method to use for component ID filtering.
    ///
    /// If `true`, Visual Studio versions are excluded from search results if they do not provide
    /// at least one whitelisted component. Otherwise if `false` (the default value), Visual Studio
    /// versions are excluded if they do not provide every whitelisted component.
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
    pub fn version_number_range(&mut self, range: Range<FourPointVersion>) -> &mut Self {
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

    /// Invokes vswhere, returning a list of detected Visual Studio installations according to the
    /// current configuration.
    pub fn run_vswhere(&self) -> io::Result<Vec<InstallInfo>> {
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

impl InstallInfo {
    /// Returns the string that uniquely identifies a Visual Studio instance.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Returns the date and time when a Visual Studio instance was installed.
    pub fn install_date(&self) -> &DateTime<Utc> {
        &self.install_date
    }

    /// Returns the internal name of a Visual Studio instance.
    pub fn installation_name(&self) -> &str {
        &self.installation_name
    }

    /// Returns the filesystem path to a Visual Studio instance.
    pub fn installation_path(&self) -> &Path {
        &self.installation_path
    }

    /// Returns the product version number for a Visual Studio instance.
    pub fn installation_version(&self) -> &FourPointVersion {
        &self.installation_version
    }

    /// Returns the product ID for a Visual Studio instance.
    pub fn product_id(&self) -> &str {
        &self.product_id
    }

    /// Returns the filesystem path to the main executable for a Visual Studio instance.
    pub fn product_path(&self) -> &Path {
        &self.product_path
    }

    /// Returns `true` if a Visual Studio instance is a prerelease version, or `false` otherwise.
    pub fn is_prerelease(&self) -> bool {
        self.is_prerelease
    }

    /// Returns the human-readable name of a Visual Studio instance.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Returns the human-readable description for a Visual Studio instance.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the ID of the release channel that a Visual Studio instance is associated with.
    pub fn channel_id(&self) -> &str {
        &self.channel_id
    }

    /// Returns the filesystem path to the catalog file for the release channel that a Visual
    /// Studio instance is associated with.
    pub fn channel_path(&self) -> &Path {
        &self.channel_path
    }

    /// Returns the URL from where release channel updates are fetched.
    pub fn channel_url(&self) -> &Url {
        &self.channel_uri
    }

    /// {TODO}
    pub fn engine_path(&self) -> &Path {
        &self.engine_path
    }

    /// Returns the URL to the release notes for a Visual Studio instance.
    pub fn release_notes(&self) -> &Url {
        &self.release_notes
    }

    /// Returns the URL to the third-party notices for a Visual Studio instance.
    pub fn third_party_notices(&self) -> &Url {
        &self.third_party_notices
    }

    /// Returns the date and time when a Visual Studio instance was last updated.
    pub fn update_date(&self) -> &DateTime<Utc> {
        &self.update_date
    }

    /// Returns the catalog information for a Visual Studio instance.
    pub fn catalog(&self) -> &InstallCatalog {
        &self.catalog
    }

    /// Returns the property information for a Visual Studio instance.
    pub fn properties(&self) -> &InstallProperties {
        &self.properties
    }
}

impl InstallCatalog {
    /// {TODO}
    pub fn build_branch(&self) -> &str {
        &self.build_branch
    }

    /// {TODO}
    pub fn build_version(&self) -> &FourPointVersion {
        &self.build_version
    }

    /// {TODO}
    pub fn id(&self) -> &str {
        &self.id
    }

    /// {TODO}
    pub fn local_build(&self) -> &str {
        &self.local_build
    }

    /// {TODO}
    pub fn manifest_name(&self) -> &str {
        &self.manifest_name
    }

    /// {TODO}
    pub fn manifest_type(&self) -> &str {
        &self.manifest_type
    }

    /// Returns the human-readable version number for a Visual Studio instance.
    pub fn product_display_version(&self) -> &str {
        &self.product_display_version
    }

    /// {TODO}
    pub fn product_line(&self) -> &str {
        &self.product_line
    }

    /// {TODO}
    pub fn product_line_version(&self) -> &str {
        &self.product_line_version
    }

    /// {TODO}
    pub fn product_milestone(&self) -> &str {
        &self.product_milestone
    }

    /// {TODO}
    pub fn product_milestone_is_pre_release(&self) -> bool {
        unimplemented!()
    }

    /// {TODO}
    pub fn product_name(&self) -> &str {
        &self.product_name
    }

    /// {TODO}
    pub fn product_patch_version(&self) -> &str {
        &self.product_patch_version
    }

    /// {TODO}
    pub fn product_pre_release_milestone_suffix(&self) -> &str {
        &self.product_pre_release_milestone_suffix
    }

    /// {TODO}
    pub fn product_release(&self) -> &str {
        &self.product_release
    }

    /// Returns the semver-compliant version number for a Visual Studio instance.
    pub fn product_semantic_version(&self) -> &Version {
        &self.product_semantic_version
    }

    /// {TODO}
    pub fn required_engine_version(&self) -> &FourPointVersion {
        &self.required_engine_version
    }
}

impl InstallProperties {
    /// {TODO}
    pub fn campaign_id(&self) -> &str {
        &self.campaign_id
    }

    /// {TODO}
    pub fn channel_manifest_id(&self) -> &str {
        &self.channel_manifest_id
    }

    /// Returns the user-assigned nickname for a Visual Studio instance.
    pub fn nickname(&self) -> &str {
        &self.nickname
    }

    /// {TODO}
    pub fn setup_engine_file_path(&self) -> &Path {
        &self.setup_engine_file_path
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

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::Path;
    use {Config, FourPointVersion};

    #[test]
    fn test_default() {
        let _ = Config::default()
            .run_vswhere()
            .expect("`Config::run_vswhere` failed");
    }

    #[test]
    fn test_everything() {
        let pfx86 = env::var_os("ProgramFiles(x86)")
            .expect("`ProgramFiles(x86)` environment variable not set");
        let _ = Config::new()
            .find_prerelease_versions(true)
            .whitelist_product_id("*")
            .whitelist_component_id("Microsoft.VisualStudio.Component.VC.Tools.x86.x64")
            .require_any_component(true)
            .version_number_range(
                FourPointVersion::new(15, 0, 0, 0)..FourPointVersion::new(16, 0, 0, 0),
            )
            .only_latest_versions(true)
            .vswhere_path(Path::new(&pfx86).join("Microsoft Visual Studio/Installer/vswhere.exe"))
            .run_vswhere();
    }
}

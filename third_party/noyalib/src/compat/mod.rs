// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! Compatibility shims for downstream crates migrating to `noyalib`.
//!
//! Each sub-module here mirrors the public API of an upstream YAML
//! library byte-for-byte, so an existing codebase can switch by
//! changing one line in `Cargo.toml` and one `use` statement. The
//! shims are intentionally thin — they delegate every operation to
//! the underlying `noyalib` engine without re-implementing any
//! parsing, serialisation, or schema logic.
//!
//! Each shim is gated behind its own feature flag so users who do
//! not need migration help do not pay for the wrapper code or the
//! re-exports.

#[cfg(feature = "compat-serde-yaml")]
#[cfg_attr(docsrs, doc(cfg(feature = "compat-serde-yaml")))]
pub mod serde_yaml;

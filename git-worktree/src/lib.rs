//! ## Feature Flags
#![cfg_attr(
    feature = "document-features",
    cfg_attr(doc, doc = ::document_features::document_features!())
)]
#![forbid(unsafe_code, rust_2018_idioms)]

/// file system related utilities
pub mod fs;

pub mod index;

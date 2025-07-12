//! DOM diffing module implementing a morphdom-like algorithm for HTML diffing
//! 
//! This module provides efficient DOM diffing capabilities that work with HTML strings/fragments
//! to generate minimal update instructions.

pub mod morphdom;
pub mod patch;
pub mod parser;
pub mod optimizer;

pub use morphdom::{diff_html, DiffOptions};
pub use patch::{Patch, PatchOp};

#[cfg(test)]
mod tests;
//! Client configuration: add or remove Vexen's MCP entry in agent config files.
//!
//! Rules: preview first, never touch unrelated keys, never overwrite a
//! different existing `vexen` entry without `--force`, refuse (and print a
//! snippet) when a file cannot be edited losslessly, back up once, write
//! atomically, and remove only an entry that matches what install writes.

mod config;
mod registry;

pub use config::{Outcome, Request, run, snippet};
pub use registry::{Adapter, Scope, adapters, find};

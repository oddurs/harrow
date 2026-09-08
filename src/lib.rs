//! harrow — work a cairn backlog from the terminal.
//!
//! The crate is a library first so that reading a project, the state machine and
//! the rendering can all be tested without a terminal and without a repository
//! underneath them. The binary is a thin shell over it.
//!
//! Reads go straight to the item files; writes go through `cairn`, which owns
//! the locking, the hooks and the rules about what a valid item is. Nothing in
//! the core performs a side effect — [`app::App`] returns actions for the shell
//! to carry out, which is what makes the tests possible.

#![deny(unsafe_code)]
#![warn(clippy::all)]

pub mod app;
pub mod config;
pub mod diag;
pub mod doctor;
pub mod engine;
pub mod exec;
pub mod filter;
pub mod item;
pub mod keys;
pub mod runtime;
pub mod schema;
pub mod term;
pub mod testkit;
pub mod theme;
pub mod ui;

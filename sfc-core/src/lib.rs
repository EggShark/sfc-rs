//! # SFC-Core
//! This library provides shared types and utilties for controlling Sensirions Mass Flow Controllers. Currently it is used by Sfc6xxx-rs and Sfc5xxx-rs
//! ## Features
//! - Translating to and from SHDLC in the [shdlc] module
//! - Handling Shared Device Errors in the [error] module
//! - Handling common units across devices in the [gasunit] module
pub mod gasunit;
pub mod shdlc;
pub mod error;

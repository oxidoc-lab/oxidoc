//! oxidoc-openapi — Interactive API Playground island for Oxidoc documentation sites.
//!
//! This crate provides a Wasm-based island component that renders interactive API playgrounds
//! within Oxidoc documentation. It uses Shadow DOM for CSS isolation and implements the
//! OxidocIsland trait for integration with oxidoc-registry.

pub mod auth;
pub mod codegen;
pub mod playground;
pub mod request;
pub mod response;
pub mod styles;

pub use playground::{ApiPlayground, ApiPlaygroundProps, ParameterDef};

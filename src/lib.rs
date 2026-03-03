#![no_std]
#![deny(warnings)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! `os_service_fabric` is a dependency-free, `no_std`-first framework for
//! modeling and orchestrating operating system services.
//!
//! It defines:
//!
//! - explicit service lifecycles (`state` module)
//! - static, allocation-free dependency graphs (`graph` module)
//! - health reporting (`health` module)
//! - recovery policies (`policy` module)
//! - a deterministic runtime that drives services (`runtime` module)
//!
//! The crate does **not** perform I/O, allocation, or logging on its own.
//! It is intended to be integrated into kernels or bare-metal frameworks
//! that provide these capabilities externally.

extern crate core;

pub mod error;
pub mod graph;
pub mod health;
pub mod id;
pub mod policy;
pub mod runtime;
pub mod service;
pub mod state;

pub use crate::error::ServiceError;
pub use crate::graph::{GraphError, ServiceGraph, ServiceNode};
pub use crate::health::{HealthLevel, ServiceHealth};
pub use crate::id::ServiceId;
pub use crate::policy::{RecoveryAction, RecoveryPolicy};
pub use crate::runtime::{FabricRuntime, FabricStepOutcome};
pub use crate::service::Service;
pub use crate::state::ServiceState;

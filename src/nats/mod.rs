//! NATS Communication Layer for Document Domain
//!
//! This module provides NATS-first communication infrastructure for the Document domain,
//! implementing CIM principles for perfect domain isolation and event-driven architecture.

pub mod subjects;
pub mod message_identity;

pub use subjects::*;
pub use message_identity::*;
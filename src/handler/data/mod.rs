//! MID-specific data structures with friendly names
//!
//! Each module defines typed data structures for specific MID responses,
//! implementing the ResponseData trait for automatic serialization.

pub mod communication_start;
pub mod tightening_result;
pub mod command_accepted;

pub use communication_start::CommunicationStartAck;
pub use tightening_result::TighteningResult;

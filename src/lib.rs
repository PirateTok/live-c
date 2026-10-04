#![deny(unused_must_use)]
#![deny(for_loops_over_fallibles)]
#![deny(dead_code)]
#![deny(unused_variables)]
#![deny(unused_assignments)]

pub mod api;
pub mod client;
pub mod codes;
pub mod connect;
pub mod ffi_str;
pub mod last_error;
pub mod runtime;
pub mod serialize;

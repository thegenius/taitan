#![allow(async_fn_in_trait)]
#![allow(dead_code)]
#![allow(clippy::needless_return)]
#![forbid(unsafe_code)]
pub mod application;
pub mod config;
pub mod const_val;
pub mod error;
pub mod file;
pub mod logger;
pub mod response;
pub mod result;
pub mod spa;

pub mod middleware;
pub mod state;

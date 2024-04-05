//! Implementations of the syscall about file system
extern crate alloc;

mod ctl;
mod epoll;
mod io;
mod link;
mod mount;
mod poll;
mod stat;
mod fd_ops;
mod stdio;
pub use ctl::*;
pub use epoll::*;
pub use io::*;
pub use link::*;
pub use mount::*;
pub use poll::*;
pub use stat::*;

#[rustfmt::skip]
#[path = "./ctypes_gen.rs"]
#[allow(dead_code, non_snake_case, non_camel_case_types, non_upper_case_globals, clippy::upper_case_acronyms, missing_docs)]
pub mod ctypes;

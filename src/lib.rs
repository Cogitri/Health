// We don't want default methods for all GObject types
#![allow(clippy::new_without_default)]
#![warn(clippy::await_holding_refcell_ref)]
#![warn(clippy::cast_lossless)]
#![warn(clippy::comparison_to_empty)]
#![warn(clippy::find_map)]
#![warn(clippy::map_unwrap_or)]
#![warn(clippy::redundant_closure_for_method_calls)]
#![warn(clippy::struct_excessive_bools)]
#![warn(clippy::unnecessary_unwrap)]
#![warn(clippy::wildcard_imports)]

pub mod config;
pub mod core;
pub mod model;
pub mod sync;
pub mod views;
pub mod widgets;
pub mod windows;

pub use crate::core::*;
pub use crate::model::*;
pub use crate::views::*;
pub use crate::widgets::*;
pub use crate::windows::*;

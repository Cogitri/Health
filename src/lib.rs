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

#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod core;
#[doc(hidden)]
pub mod model;
#[doc(hidden)]
pub mod sync;
#[doc(hidden)]
pub mod views;
#[doc(hidden)]
pub mod widgets;
#[doc(hidden)]
pub mod windows;

#[doc(inline)]
pub use crate::core::*;
#[doc(inline)]
pub use crate::model::*;
#[doc(inline)]
pub use crate::views::*;
#[doc(inline)]
pub use crate::widgets::*;
#[doc(inline)]
pub use crate::windows::*;

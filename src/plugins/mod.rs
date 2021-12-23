// Hide and inline the docs for all generic plugin parts
pub mod activities;
pub mod calories;
#[doc(hidden)]
pub mod details;
#[doc(hidden)]
pub mod overview;
#[doc(hidden)]
pub mod plugin;
#[doc(hidden)]
pub mod plugin_list;
#[doc(hidden)]
pub mod plugin_name;
#[doc(hidden)]
pub mod plugin_object;
#[doc(hidden)]
pub mod registrar;
pub mod steps;
#[doc(hidden)]
pub mod summary;
pub mod weight;

pub use activities::*;
pub use calories::*;
#[doc(inline)]
pub use details::*;
#[doc(inline)]
pub use overview::*;
#[doc(inline)]
pub use plugin::*;
#[doc(inline)]
pub use plugin_list::*;
#[doc(inline)]
pub use plugin_name::*;
#[doc(inline)]
pub use plugin_object::*;
#[doc(inline)]
pub use registrar::*;
pub use steps::*;
#[doc(inline)]
pub use summary::*;
pub use weight::*;

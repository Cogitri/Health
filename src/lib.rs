//! # Health
//!
//! Welcome to Heath's docs! Health is a health tracking app for the GNOME desktop.
//!
//! This documentation should help you get started with Health's structs and functions. To make
//! the start a bit easier, let me explain some basics about Health:
//!
//! * Health's source is split into multiple directiories to make working with the many source files a bit easier:
//!     * core: This directory contains core modules of Health that are used throughout the application, like the
//!       [Settings](crate::core::Settings) struct that's used for retrieving `GSettings` values.
//!     * model: This directory contains the data types of Health. Things like [Activity](crate::model::Activity), which
//!       holds data about a single activity live here.
//!     * sync: The code for synching with third party providers (e.g. Google Fit) or exporting/importing data lies here.
//!     * views: This directory contains Views. These make up a single page in Health, e.g. the [ViewSteps](crate::views::ViewSteps)
//!       that draws a that draws a graph of the user's step count.
//!     * widgets: Widgets are smaller, reusable parts of Health's UI, e.g the [BMILevelBar](crate::widgets::BMILevelBar)
//!       is contained in this module.
//!     * windows: This directory contains actual windows, like the main [Window](crate::windows::Window) or the
//!       [PreferencesWindow](crate::windows::PreferencesWindow).
//! * Health has a rather strict code style to make sure working with the sourcecode is easy:
//!     * Please make sure `cargo fmt` and `cargo clippy` are happy with any changes you do.
//!       CI will also run these tests when creating new merge requests.
//!     * Please make sure your subclasses roughly follow this structure:
//!       ```
//!       mod my_struct {
//!           use glib::subclass::prelude::*;
//!
//!           // This is your struct to do GObject subclassing.
//!           // No additional logic should be implemented here.
//!           mod imp {
//!               use glib::subclass::{self, prelude::*};
//!
//!               pub struct MyStruct {}
//!               impl ObjectSubclass for MyStruct {
//!                   const NAME: &'static str = "MyStruct";
//!                   type ParentType = glib::Object;
//!                   type Instance = subclass::simple::InstanceStruct<Self>;
//!                   type Class = subclass::simple::ClassStruct<Self>;
//!                   type Type = super::MyStruct;
//!                   type Interfaces = ();
//!
//!                   glib::object_subclass!();
//!
//!                   fn new() -> Self {
//!                       Self { }
//!                   }
//!               }
//!
//!               impl ObjectImpl for MyStruct {}
//!           }
//!
//!           // This is your public struct, that can be used in other modules / ui XML etc.
//!           glib::wrapper! {
//!               pub struct MyStruct(ObjectSubclass<imp::MyStruct>);
//!           }
//!
//!           // Actual logic goes here.
//!           impl MyStruct {}
//!       }
//!       ```
//!     * Please sure to order functions in the following order to make it easy to find functions:
//!          * `pub` functions first, sorted alphabetically
//!          *  private functions afterwards, also sorted alphabetically
//!     * Try only having one (public) GObject class per file and name the file after the public class.
//!       That way it's easy to find classes in the folder structure.
//!     * Personally, I prefer to not `use` gtk and other GObject crates, but you do you.
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

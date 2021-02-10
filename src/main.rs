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

use gettextrs::{bindtextdomain, setlocale, textdomain, LocaleCategory};
use gtk::prelude::ApplicationExtManual;

mod config;
mod core;
mod model;
mod sync;
mod views;
mod widgets;
mod windows;

fn main() {
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain("dev.Cogitri.Health", config::LOCALEDIR);
    textdomain("dev.Cogitri.Health");

    glib::set_application_name(&core::i18n("Health"));
    glib::set_prgname(Some("dev.Cogitri.Health"));

    gtk::init().expect("Failed to initialize GTK.");
    adw::init();

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/dev.Cogitri.Health.gresource")
        .expect("Could not load resources");
    gio::resources_register(&res);

    let app = crate::core::Application::new();

    let ret = app.run(&std::env::args().collect::<Vec<_>>());
    std::process::exit(ret);
}

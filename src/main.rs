// We don't want default methods for all GObject types
#![allow(clippy::new_without_default)]

use gettextrs::*;
use gtk::prelude::ApplicationExtManual;

mod config;
mod core;
mod model;
mod sync;
mod views;
mod widgets;
mod windows;

use crate::core::application::HealthApplication;

fn main() {
    gtk::init().expect("Failed to initialize GTK.");
    adw::init();

    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain("dev.Cogitri.Health", config::LOCALEDIR);
    textdomain("dev.Cogitri.Health");

    let res = gio::Resource::load(config::PKGDATADIR.to_owned() + "/dev.Cogitri.Health.gresource")
        .expect("Could not load resources");
    gio::resources_register(&res);

    let app = HealthApplication::new();

    let ret = app.run(&std::env::args().collect::<Vec<_>>());
    std::process::exit(ret);
}

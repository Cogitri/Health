// We don't want default methods for all GObject types
#![allow(clippy::new_without_default)]
#![warn(clippy::await_holding_refcell_ref)]
#![warn(clippy::cast_lossless)]
#![warn(clippy::comparison_to_empty)]
#![warn(clippy::manual_find_map)]
#![warn(clippy::map_unwrap_or)]
#![warn(clippy::redundant_closure_for_method_calls)]
#![warn(clippy::struct_excessive_bools)]
#![warn(clippy::unnecessary_unwrap)]
#![warn(clippy::wildcard_imports)]
#![warn(clippy::trivially_copy_pass_by_ref)]
#![warn(clippy::option_if_let_else)]

use gettextrs::{bindtextdomain, setlocale, textdomain, LocaleCategory};
use gtk::{gio, glib, prelude::ApplicationExtManual};
use libhealth::{
    config,
    core::{i18n, utils, Application},
};

fn main() {
    setlocale(LocaleCategory::LcAll, "");
    if let Err(e) = bindtextdomain(config::GETTEXT_PACKAGE, config::LOCALEDIR) {
        glib::g_warning!(config::LOG_DOMAIN, "Couldn't bind textdomain: {}", e)
    }
    if let Err(e) = textdomain(config::GETTEXT_PACKAGE) {
        glib::g_warning!(config::LOG_DOMAIN, "Couldn't set textdomain: {}", e)
    }

    glib::set_application_name(&i18n("Health"));
    glib::set_prgname(Some("dev.Cogitri.Health"));

    gtk::init().expect("Failed to initialize GTK.");
    adw::init();

    let res = if config::APPLICATION_ID.ends_with("Devel") {
        utils::get_file_in_builddir("dev.Cogitri.Health.gresource").map_or_else(
            || {
                gio::Resource::load(config::PKGDATADIR.to_owned() + "/dev.Cogitri.Health.gresource")
                    .expect("Could not load resources")
            },
            |s| gio::Resource::load(s).expect("Could not load resources"),
        )
    } else {
        gio::Resource::load(config::PKGDATADIR.to_owned() + "/dev.Cogitri.Health.gresource")
            .expect("Could not load resources")
    };
    gio::resources_register(&res);

    let app = Application::new();

    let ret = app.run();
    std::process::exit(ret);
}

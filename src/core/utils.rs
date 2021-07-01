/* utils.rs
 *
 * Copyright 2020-2021 Rasmus Thomsen <oss@cogitri.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

#[cfg(test)]
use gtk::gio;

pub mod prelude {
    use gtk::{glib, prelude::*};
    use std::future::Future;

    #[easy_ext::ext(HealthEditableExt)]
    impl<B> B
    where
        B: IsA<gtk::Editable>,
    {
        /// Get the number-value of a [gtk::Editable], even if the user hasn't pressed `Enter` yet to commit the value.
        ///
        ///
        /// # Returns
        /// The value of the [gtk::Editable] or `T::default()`.
        pub fn raw_value<T>(&self) -> Option<T>
        where
            T: std::str::FromStr + Default,
            <T as std::str::FromStr>::Err: std::fmt::Debug,
        {
            self.text().as_str().parse::<T>().ok()
        }
    }

    #[easy_ext::ext(F32Ext)]
    impl f32 {
        /// Round a number to a certain amount of decimal places.
        ///
        /// # Arguments
        /// * `self` - The value to round.
        /// * `decimal_places` - The amount of decimal places to round this to.
        ///
        /// # Returns
        /// The rounded value.
        ///
        /// # Examples
        /// ```
        /// use libhealth::utils::prelude::*;
        ///
        /// let val: f32 = 13.54231;
        /// assert_eq!(val.round_decimal_places(1), 13.5);
        /// assert_eq!(val.round_decimal_places(2), 13.54);
        /// ```
        pub fn round_decimal_places(self, decimal_places: u32) -> f32 {
            let round_factor = (10_u32).pow(decimal_places) as f32;
            (self * round_factor).round() / round_factor
        }
    }

    #[easy_ext::ext(HealthFutureExt)]
    impl<F, T> F
    where
        F: Future<Output = T> + 'static,
        T: 'static,
    {
        /// Block on the provided future and return the result.
        ///
        /// # Arguments
        /// * `future` - The future to run.
        ///
        /// # Returns
        /// Returns the return value of the future.
        ///
        /// # Examples
        /// ```
        /// use libhealth::utils::prelude::*;
        ///
        /// assert_eq!(async { 25 }.block(), 25);
        /// ```
        pub fn block(self) -> T {
            let context = glib::MainContext::new();
            let ml = glib::MainLoop::new(Some(&context), false);
            let (sender, receiver) = std::sync::mpsc::channel();

            context.push_thread_default();
            let m = ml.clone();
            context.spawn_local(async move {
                sender.send(self.await).unwrap();
                m.quit();
            });

            ml.run();

            receiver.recv().unwrap()
        }
    }
}

#[cfg(test)]
pub fn get_file_in_builddir(filename: &str) -> Option<std::path::PathBuf> {
    glob::glob(&format!("{}/**/{}", env!("CARGO_MANIFEST_DIR"), filename))
        .ok()
        .and_then(|mut p| p.next())
        .and_then(|p| p.ok())
}

#[cfg(test)]
pub fn init_gtk() {
    let res = if let Some(gresource_path) = get_file_in_builddir("dev.Cogitri.Health.gresource") {
        gio::Resource::load(gresource_path)
    } else {
        use std::process::Command;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let mut gresource_path = dir.path().to_path_buf();
        gresource_path.push("out.gresource");

        Command::new("glib-compile-resources")
            .arg(&format!(
                "{}/data/dev.Cogitri.Health.gresource.xml",
                env!("CARGO_MANIFEST_DIR")
            ))
            .arg("--sourcedir")
            .arg(&format!("{}/data", env!("CARGO_MANIFEST_DIR")))
            .arg("--internal")
            .arg("--target")
            .arg(&gresource_path)
            .spawn()
            .expect("Failed to run glib-compile-resources!");

        std::thread::sleep(std::time::Duration::from_secs(1));

        gio::Resource::load(gresource_path)
    };

    gio::resources_register(&res.unwrap());

    gtk::init().unwrap();
}

/// Initialise some environment variables for testing GSchemas and compile the GSchema
/// if meson hasn't done so for us already.
///
/// # Returns
/// A [TempDir](tempfile::TempDir) if we had to compile the GSchema ourselves and put the
/// result in a temporary directory. You have to hold onto the return value for as long
/// as you need the GSchema (so probably your entire test function), since the temporary
/// directory on the disk will be cleaned once the return value is dropped.
#[cfg(test)]
#[must_use]
pub fn init_gschema() -> Option<tempfile::TempDir> {
    use std::env::set_var;

    set_var("GSETTINGS_BACKEND", "memory");
    if let Some(dir) = get_file_in_builddir("gschemas.compiled") {
        set_var("GSETTINGS_SCHEMA_DIR", dir.parent().unwrap());
        None
    } else {
        use std::process::Command;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let gschema_path = dir.path().to_path_buf();

        Command::new("glib-compile-schemas")
            .arg(&format!("{}/data", env!("CARGO_MANIFEST_DIR")))
            .arg("--targetdir")
            .arg(&gschema_path)
            .spawn()
            .expect("Failed to run glib-compile-schemas!");

        std::thread::sleep(std::time::Duration::from_secs(1));
        set_var("GSETTINGS_SCHEMA_DIR", gschema_path);
        Some(dir)
    }
}

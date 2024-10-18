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

#[macro_use]
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
            self.text()
                .split(' ')
                .next()
                .and_then(|s| s.parse::<T>().ok())
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

    #[easy_ext::ext(F64Ext)]
    impl f64 {
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
        /// let val: f64 = 13.54231;
        /// assert_eq!(val.round_decimal_places(1), 13.5);
        /// assert_eq!(val.round_decimal_places(2), 13.54);
        /// ```
        pub fn round_decimal_places(self, decimal_places: u32) -> f64 {
            let round_factor = f64::from((10_u32).pow(decimal_places));
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

            context
                .with_thread_default(|| {
                    let m = ml.clone();
                    context.spawn_local(async move {
                        sender.send(self.await).unwrap();
                        m.quit();
                    });

                    ml.run();
                })
                .unwrap();

            receiver.recv().unwrap()
        }
    }

    #[easy_ext::ext(OptionU32Ext)]
    impl Option<u32> {
        /// Return the inner value of `self` is Some, or `alternative`
        ///
        /// # Arguments
        /// * `self` - The value to check.
        /// * `alternative` - The alternative value to return.
        ///
        /// # Returns
        /// One of the two values.
        ///
        /// # Examples
        /// ```
        /// use libhealth::utils::prelude::*;
        ///
        /// let val: Option<u32> = Some(5);
        /// let none: Option<u32> = None;
        /// assert_eq!(val.unwrap_ori(-1), 5);
        /// assert_eq!(none.unwrap_ori(-1), -1);
        /// ```
        pub fn unwrap_ori(self, alternative: i64) -> i64 {
            if let Some(u) = self {
                u.into()
            } else {
                alternative
            }
        }
    }

    #[macro_export]
    macro_rules! stateful_action {
        ($actions_group:expr, $name:expr, $state:expr, $callback:expr) => {
            let simple_action = gio::SimpleAction::new_stateful($name, None, $state.to_variant());
            simple_action.connect_activate($callback);
            $actions_group.add_action(&simple_action);
        };
        ($actions_group:expr, $name:expr, $param_type:expr, $state:expr, $callback:expr) => {
            let simple_action =
                gio::SimpleAction::new_stateful($name, $param_type, &$state.to_variant());
            simple_action.connect_activate($callback);
            $actions_group.add_action(&simple_action);
        };
    }
}

pub fn get_file_in_builddir(filename: &str) -> Option<std::path::PathBuf> {
    glob::glob(&format!("{}/**/{}", env!("CARGO_MANIFEST_DIR"), filename))
        .ok()
        .and_then(|mut p| p.next())
        .and_then(std::result::Result::ok)
}

#[cfg(test)]
pub fn init_gresources() {
    gio::resources_register_include!("compiled.gresource").unwrap();
}

#[cfg(test)]
pub fn init_gtk() {
    init_gresources();

    gtk::init().unwrap();
    adw::init().unwrap();
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

        let output = Command::new("glib-compile-schemas")
            .arg(&format!("{}/data", env!("CARGO_MANIFEST_DIR")))
            .arg("--targetdir")
            .arg(&gschema_path)
            .output()
            .expect("Failed to run glib-compile-schemas!");

        if !output.status.success() {
            panic!(
                "Couldn't execute glib-compile-resources! Status: {} Stdout: {}, Stderr: {}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        set_var("GSETTINGS_SCHEMA_DIR", gschema_path);
        Some(dir)
    }
}

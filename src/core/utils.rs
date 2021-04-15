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

use gtk::EditableExt;
use std::future::Future;

/// Get the number-value of a [gtk::SpinButton].
///
/// # Arguments
/// * `spin_button` - The [gtk::SpinButton] to get the value of.
///
/// # Returns
/// The value of the [gtk::SpinButton] or `T::default()`.
pub fn spinbutton_value<T>(spin_button: &gtk::SpinButton) -> T
where
    T: std::str::FromStr + Default,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    spin_button.text().as_str().parse::<T>().unwrap_or_default()
}

/// Round a number to a certain amount of decimal places.
///
/// # Arguments
/// * `val` - The value to round.
/// * `decimal_places` - The amount of decimal places to round this to.
///
/// # Returns
/// The rounded value.
///
/// # Examples
/// ```
/// use libhealth::utils::round_decimal_places;
///
/// let val = 13.54231;
/// assert_eq!(round_decimal_places(val, 1), 13.5);
/// assert_eq!(round_decimal_places(val, 2), 13.54);
/// ```
pub fn round_decimal_places(val: f32, decimal_places: u32) -> f32 {
    let round_factor = (10_u32).pow(decimal_places) as f32;
    (val * round_factor).round() / round_factor
}

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
/// use libhealth::utils::run_gio_future_sync;
///
/// assert_eq!(run_gio_future_sync(async { 25 }), 25);
/// ```
pub fn run_gio_future_sync<T: 'static, F: 'static>(future: F) -> T
where
    F: Future<Output = T>,
{
    let context = glib::MainContext::new();
    let ml = glib::MainLoop::new(Some(&context), false);
    let (sender, receiver) = std::sync::mpsc::channel();

    context.push_thread_default();
    let m = ml.clone();
    context.spawn_local(async move {
        sender.send(future.await).unwrap();
        m.quit();
    });

    ml.run();

    receiver.recv().unwrap()
}

#[cfg(test)]
pub fn init_gtk() {
    let res = gio::Resource::load(
        glob::glob("./**/dev.Cogitri.Health.gresource")
            .ok()
            .and_then(|mut p| p.next())
            .and_then(|p| p.ok())
            .expect("Couldn't find GResource file, did you run meson? `meson build && ninja -C build data/dev.Cogitri.Health.gresource` should get you up to speed."),
    )
    .expect("Could not load resources");

    gio::resources_register(&res);

    gtk::init().unwrap();
}

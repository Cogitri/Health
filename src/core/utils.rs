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
#[cfg(test)]
use std::future::Future;

pub fn get_spinbutton_value<T>(spin_button: &gtk::SpinButton) -> T
where
    T: std::str::FromStr + Default,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    spin_button
        .get_text()
        .as_str()
        .parse::<T>()
        .unwrap_or_default()
}

#[cfg(test)]
pub fn run_async_test_fn<T: 'static, F: 'static>(future: F) -> T
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

/* database_receiver.rs
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

use crate::{
    core::{i18n_f, Database},
    model::{Steps, Weight},
};
use gtk::glib;
use gtk_macros::spawn;

pub enum DatabaseValue {
    Steps(Vec<Steps>),
    Weights(Vec<Weight>),
}

/// Create a [async_channel::Sender] which can be used in threaded scenarios (e.g. sync providers).
/// Values sent through the sender will automatically import it into the DB.
pub fn new_db_receiver() -> async_channel::Sender<DatabaseValue> {
    let (sender, receiver) = async_channel::unbounded();

    glib::spawn_future_local(async move {
        while let Ok(value) = receiver.recv().await {
            let db = Database::instance();
            match value {
                DatabaseValue::Steps(s) => {
                    spawn!(async move {
                        if let Err(e) = db.import_steps(&s).await {
                            glib::g_warning!(
                                crate::config::LOG_DOMAIN,
                                "{}",
                                i18n_f(
                                    "Couldn’t synchronize steps due to error {}",
                                    &[&e.to_string()]
                                )
                            );
                        }
                    });
                }
                DatabaseValue::Weights(w) => {
                    spawn!(async move {
                        if let Err(e) = db.import_weights(&w).await {
                            glib::g_warning!(
                                crate::config::LOG_DOMAIN,
                                "{}",
                                i18n_f(
                                    "Couldn’t synchronize weight measurements due to error {}",
                                    &[&e.to_string()]
                                )
                            );
                        }
                    });
                }
            }
        }
    });

    sender
}

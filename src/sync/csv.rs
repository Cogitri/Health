/* csv.rs
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

use crate::core::Database;
use gio::FileExt;
use gtk::prelude::*;
use std::convert::TryFrom;

/// [CSVHandler] is a struct which manages exporting data from the Tracker DB to a
/// CSV file or importing it from a CSV file into the Tracker DB.
pub struct CSVHandler {
    db: Database,
}

impl CSVHandler {
    pub fn new() -> Self {
        Self {
            db: Database::instance(),
        }
    }

    /// Export all [Activity](crate::model::Activity)s in the Tracker DB to a CSV file.
    ///
    /// # Arguments
    /// * `file` - The file to write the CSV data to.
    ///
    /// # Returns
    /// An error if writing to the file fails or reading from the DB.
    pub async fn export_activities_csv(&self, file: &gio::File) -> Result<(), glib::Error> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        for activity in self.db.activities(None).await? {
            match wtr.serialize(activity) {
                Ok(_) => {}
                Err(e) => {
                    return Err(glib::error::Error::new(
                        glib::FileError::Failed,
                        &e.to_string(),
                    ))
                }
            }
        }

        self.write_csv(file, wtr.into_inner().unwrap()).await?;

        Ok(())
    }

    /// Export all [Weight](crate::model::Weight) in the Tracker DB to a CSV file.
    ///
    /// # Arguments
    /// * `file` - The file to write the CSV data to.
    ///
    /// # Returns
    /// An error if writing to the file fails or reading from the DB.
    pub async fn export_weights_csv(&self, file: &gio::File) -> Result<(), glib::Error> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        for weight in self.db.weights(None).await? {
            match wtr.serialize(weight) {
                Ok(_) => {}
                Err(e) => {
                    return Err(glib::error::Error::new(
                        glib::FileError::Failed,
                        &e.to_string(),
                    ))
                }
            }
        }

        self.write_csv(file, wtr.into_inner().unwrap()).await?;

        Ok(())
    }

    /// Import all [Activity](crate::model::Activity)s from a CSV file to the Tracker DB.
    ///
    /// # Arguments
    /// * `file` - The file to read the CSV data from.
    ///
    /// # Returns
    /// An error if reading from the file fails or writing to the DB.
    pub async fn import_activities_csv(&self, file: &gio::File) -> Result<(), glib::Error> {
        let (data, _) = file.load_contents_async_future().await?;
        let mut rdr = csv::Reader::from_reader(&*data);

        for activity in rdr.deserialize() {
            match activity {
                Ok(a) => Ok(self.db.save_activity(a).await?),
                Err(e) => {
                    return Err(glib::error::Error::new(
                        glib::FileError::Failed,
                        &e.to_string(),
                    ))
                }
            }?;
        }

        Ok(())
    }

    /// Import all [Weight](crate::model::Weight)s from a CSV file to the Tracker DB.
    ///
    /// # Arguments
    /// * `file` - The file to read the CSV data from.
    ///
    /// # Returns
    /// An error if reading from the file fails or writing to the DB.
    pub async fn import_weights_csv(&self, file: &gio::File) -> Result<(), glib::Error> {
        let (data, _) = file.load_contents_async_future().await?;
        let mut rdr = csv::Reader::from_reader(&*data);

        for weight in rdr.deserialize() {
            match weight {
                Ok(a) => Ok(self.db.save_weight(a).await?),
                Err(e) => {
                    return Err(glib::error::Error::new(
                        glib::FileError::Failed,
                        &e.to_string(),
                    ))
                }
            }?;
        }

        Ok(())
    }

    /// Write (CSV) data to a `File`.
    async fn write_csv(&self, file: &gio::File, data: Vec<u8>) -> Result<(), glib::Error> {
        let stream = file
            .replace_async_future(
                None,
                false,
                gio::FileCreateFlags::REPLACE_DESTINATION,
                glib::PRIORITY_DEFAULT,
            )
            .await?;

        let mut written = 0;
        while written < data.len() {
            let w = stream.write(&data[written..data.len()], None::<&gio::Cancellable>)?;
            written += usize::try_from(w).unwrap();
        }

        Ok(())
    }
}

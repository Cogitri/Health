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

use crate::{core::i18n, core::Database};
use anyhow::Result;
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use gtk::{
    gio::{self, prelude::*},
    glib,
};
use ring::rand::{self, SecureRandom};
use sha2::{Digest, Sha256};
use std::convert::TryFrom;

#[derive(thiserror::Error, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum EncryptionError {
    #[error("{0}")]
    NonceGenerate(String),
    #[error("{0}")]
    Decrypt(String),
    #[error("{0}")]
    Encrypt(String),
    #[error("{0}")]
    UnencryptedAsEncrypted(String),
    #[error("{0}")]
    EncryptedAsUnencrypted(String),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct EncryptedValue {
    pub data: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// [CsvHandler] is a struct which manages exporting data from the Tracker DB to a
/// CSV file or importing it from a CSV file into the Tracker DB.
pub struct CsvHandler {
    db: Database,
}

impl CsvHandler {
    pub fn new() -> Self {
        Self {
            db: Database::instance(),
        }
    }

    #[cfg(test)]
    pub fn new_with_database(db: Database) -> Self {
        Self { db }
    }

    /// Export all [Activity](crate::model::Activity)s in the Tracker DB to a CSV file.
    ///
    /// # Arguments
    /// * `file` - The file to write the CSV data to.
    ///
    /// # Returns
    /// An error if writing to the file fails or reading from the DB.
    pub async fn export_activities_csv(&self, file: &gio::File, key: Option<&str>) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        let activities = self.db.activities(None).await?;

        if activities.is_empty() {
            anyhow::bail!(i18n("No activities added yet; can’t create empty export!"));
        }

        for activity in activities {
            wtr.serialize(activity)?;
        }

        let data = wtr.into_inner().unwrap();
        if let Some(k) = key {
            self.write_csv_encrypted(file, &data, k).await?;
        } else {
            self.write_csv(file, &data).await?;
        }

        Ok(())
    }

    /// Export all [Weight](crate::model::Weight) in the Tracker DB to a CSV file.
    ///
    /// # Arguments
    /// * `file` - The file to write the CSV data to.
    ///
    /// # Returns
    /// An error if writing to the file fails or reading from the DB.
    pub async fn export_weights_csv(&self, file: &gio::File, key: Option<&str>) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        let weights = self.db.weights(None).await?;

        if weights.is_empty() {
            anyhow::bail!(i18n(
                "No weight measurements added yet; can’t create empty export!"
            ));
        }

        for weight in weights {
            wtr.serialize(weight)?;
        }

        let data = wtr.into_inner().unwrap();
        if let Some(k) = key {
            self.write_csv_encrypted(file, &data, k).await?;
        } else {
            self.write_csv(file, &data).await?;
        }

        Ok(())
    }

    /// Import all [Activity](crate::model::Activity)s from a CSV file to the Tracker DB.
    ///
    /// # Arguments
    /// * `file` - The file to read the CSV data from.
    ///
    /// # Returns
    /// An error if reading from the file fails or writing to the DB.
    pub async fn import_activities_csv(&self, file: &gio::File, key: Option<&str>) -> Result<()> {
        let data = if let Some(k) = key {
            self.read_csv_encrypted(file, k).await?
        } else {
            self.read_csv(file).await?
        };
        let mut rdr = csv::Reader::from_reader(&*data);

        for activity in rdr.deserialize() {
            match activity {
                Ok(a) => Ok(self.db.save_activity(a).await?),
                Err(e) => Err(e),
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
    pub async fn import_weights_csv(&self, file: &gio::File, key: Option<&str>) -> Result<()> {
        let data = if let Some(k) = key {
            self.read_csv_encrypted(file, k).await?
        } else {
            self.read_csv(file).await?
        };
        let mut rdr = csv::Reader::from_reader(&*data);

        for weight in rdr.deserialize() {
            match weight {
                Ok(a) => Ok(self.db.save_weight(a).await?),
                Err(e) => Err(e),
            }?;
        }

        Ok(())
    }

    async fn read_csv(&self, file: &gio::File) -> Result<Vec<u8>> {
        let data = file.load_contents_future().await?.0;

        if serde_json::from_slice::<EncryptedValue>(&data).is_ok() {
            Err(EncryptionError::EncryptedAsUnencrypted(i18n(
                "Can’t parse encrypted backup without encryption key!",
            )))
            .map_err(anyhow::Error::msg)
        } else {
            Ok(data)
        }
    }

    async fn read_csv_encrypted(&self, file: &gio::File, key: &str) -> Result<Vec<u8>> {
        let raw_contents = file.load_contents_future().await?.0;
        let encrypted_value: EncryptedValue = serde_json::from_slice(&raw_contents)
            .map_err(|_| EncryptionError::UnencryptedAsEncrypted(i18n("Couldn’t parse CSV. Are you trying to read an unencrypted backup as an encrypted one?")))?;
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = hasher.finalize();
        let key = Key::from_slice(&hash);
        let cipher = XChaCha20Poly1305::new(key);

        let nonce = XNonce::from_slice(&encrypted_value.nonce);

        Ok(cipher
            .decrypt(nonce, encrypted_value.data.as_slice())
            .map_err(|_| {
                EncryptionError::Decrypt(i18n(
                    "Couldn’t decrypt data. Are you sure you’re using the right key?",
                ))
            })?)
    }

    async fn write_csv(&self, file: &gio::File, data: &[u8]) -> Result<()> {
        let stream = file
            .replace_future(
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

    /// Write (CSV) data to a `File`.
    async fn write_csv_encrypted(&self, file: &gio::File, data: &[u8], key: &str) -> Result<()> {
        let rng = rand::SystemRandom::new();

        let mut nonce = [0u8; 24];
        rng.fill(&mut nonce)
            .map_err(|e| EncryptionError::NonceGenerate(e.to_string()))?;

        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = hasher.finalize();
        let key = Key::from_slice(&hash);
        let aead = XChaCha20Poly1305::new(key);

        let nonce = XNonce::from_slice(&nonce);
        let ciphertext = aead
            .encrypt(nonce, data)
            .map_err(|e| EncryptionError::Encrypt(e.to_string()))?;

        let encrypted_value = EncryptedValue {
            data: ciphertext,
            nonce: nonce.to_vec(),
        };
        let json = serde_json::to_string_pretty(&encrypted_value)?;
        self.write_csv(file, json.as_bytes()).await
    }
}

#[cfg(test)]
mod test {
    use super::CsvHandler;
    use crate::{
        core::{i18n, Database},
        model::{Activity, Weight},
        sync::csv::EncryptionError,
        utils::init_gschema,
    };
    use gtk::{gio, glib};
    use tempfile::tempdir;
    use uom::si::{
        f32::{Length, Mass},
        length::kilometer,
        mass::kilogram,
    };

    #[test]
    fn simple_read_write() {
        let ctx = glib::MainContext::new();
        let file = gio::File::new_tmp(Some("Health-Test-XXXXXX")).unwrap().0;
        let data_dir = tempdir().unwrap();
        let csv_handler = CsvHandler::new_with_database(
            Database::new_with_store_path(data_dir.path().into()).unwrap(),
        );
        let data = b"test string";
        ctx.block_on(csv_handler.write_csv(&file, data)).unwrap();
        let data_readback = ctx.block_on(csv_handler.read_csv(&file)).unwrap();
        assert_eq!(
            std::str::from_utf8(data).unwrap(),
            std::str::from_utf8(&data_readback).unwrap()
        );
    }

    #[test]
    fn en_decrypt() {
        let ctx = glib::MainContext::new();
        let file = gio::File::new_tmp(Some("Health-Test-XXXXXX")).unwrap().0;
        let data_dir = tempdir().unwrap();
        let csv_handler = CsvHandler::new_with_database(
            Database::new_with_store_path(data_dir.path().into()).unwrap(),
        );
        let key = "super secret test key here";
        let data = b"test string";
        ctx.block_on(csv_handler.write_csv_encrypted(&file, data, key))
            .unwrap();
        let data_readback = ctx
            .block_on(csv_handler.read_csv_encrypted(&file, key))
            .unwrap();
        assert_eq!(
            std::str::from_utf8(data).unwrap(),
            std::str::from_utf8(&data_readback).unwrap()
        );
    }

    #[test]
    fn encrypted_write_try_unecrypted_read() {
        let ctx = glib::MainContext::new();
        let file = gio::File::new_tmp(Some("Health-Test-XXXXXX")).unwrap().0;
        let data_dir = tempdir().unwrap();
        let csv_handler = CsvHandler::new_with_database(
            Database::new_with_store_path(data_dir.path().into()).unwrap(),
        );
        let key = "super secret test key here";
        let data = b"test string";
        ctx.block_on(csv_handler.write_csv_encrypted(&file, data, key))
            .unwrap();
        let data_readback = ctx.block_on(csv_handler.read_csv(&file));

        assert_eq!(
            data_readback.err().and_then(|e| e.downcast().ok()),
            Some(EncryptionError::EncryptedAsUnencrypted(i18n(
                "Can’t parse encrypted backup without encryption key!"
            ))),
        );
    }

    #[test]
    fn unencrypt_write_try_unencrypted_read() {
        let ctx = glib::MainContext::new();
        let file = gio::File::new_tmp(Some("Health-Test-XXXXXX")).unwrap().0;
        let data_dir = tempdir().unwrap();
        let csv_handler = CsvHandler::new_with_database(
            Database::new_with_store_path(data_dir.path().into()).unwrap(),
        );
        let key = "super secret test key here";
        let data = b"test string";
        ctx.block_on(csv_handler.write_csv(&file, data)).unwrap();
        let data_readback = ctx.block_on(csv_handler.read_csv_encrypted(&file, key));

        assert_eq!(
            data_readback.err().and_then(|e| e.downcast().ok()),
            Some(EncryptionError::UnencryptedAsEncrypted(i18n("Couldn’t parse CSV. Are you trying to read an unencrypted backup as an encrypted one?"))),
        );
    }

    #[test]
    fn empty_activities_export() {
        let ctx = glib::MainContext::new();
        let file = gio::File::new_tmp(Some("Health-Test-XXXXXX")).unwrap().0;
        let data_dir = tempdir().unwrap();
        let csv_handler = CsvHandler::new_with_database(
            Database::new_with_store_path(data_dir.path().into()).unwrap(),
        );

        assert_eq!(
            ctx.block_on(csv_handler.export_activities_csv(&file, None))
                .err()
                .unwrap()
                .to_string(),
            i18n("No activities added yet; can’t create empty export!")
        );
    }

    #[test]
    fn empty_weights_export() {
        let ctx = glib::MainContext::new();
        let file = gio::File::new_tmp(Some("Health-Test-XXXXXX")).unwrap().0;
        let data_dir = tempdir().unwrap();
        let csv_handler = CsvHandler::new_with_database(
            Database::new_with_store_path(data_dir.path().into()).unwrap(),
        );

        assert_eq!(
            ctx.block_on(csv_handler.export_weights_csv(&file, None))
                .err()
                .unwrap()
                .to_string(),
            i18n("No weight measurements added yet; can’t create empty export!")
        );
    }

    #[test]
    fn activities_reimport() {
        let _dir = init_gschema();
        let ctx = glib::MainContext::new();
        let file = gio::File::new_tmp(Some("Health-Test-XXXXXX")).unwrap().0;
        let data_dir = tempdir().unwrap();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let csv_handler = CsvHandler::new_with_database(db.clone());
        let act = Activity::builder()
            .date(glib::DateTime::from_unix_utc(1_000_000_000).unwrap())
            .steps(2000)
            .calories_burned(2000)
            .duration(glib::TimeSpan::from_minutes(20))
            .distance(Length::new::<kilometer>(2.0))
            .build();
        ctx.block_on(async {
            db.save_activity(act.clone()).await.unwrap();
            csv_handler
                .export_activities_csv(&file, None)
                .await
                .unwrap();
            db.reset().await.unwrap();
            csv_handler
                .import_activities_csv(&file, None)
                .await
                .unwrap();
            let new_act = &db.activities(None).await.unwrap()[0];
            assert_eq!(new_act.date(), act.date());
            assert_eq!(new_act.steps(), act.steps());
            assert_eq!(new_act.calories_burned(), act.calories_burned());
            assert_eq!(new_act.duration(), act.duration());
            assert_eq!(new_act.distance(), act.distance());
        });
    }

    #[test]
    fn weights_reimport() {
        let _dir = init_gschema();
        let ctx = glib::MainContext::new();
        let file = gio::File::new_tmp(Some("Health-Test-XXXXXX")).unwrap().0;
        let data_dir = tempdir().unwrap();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let csv_handler = CsvHandler::new_with_database(db.clone());
        let weight = Weight::new(
            glib::DateTime::from_unix_utc(1_000_000_000).unwrap(),
            Mass::new::<kilogram>(70.0),
        );
        ctx.block_on(async {
            db.save_weight(weight.clone()).await.unwrap();
            csv_handler.export_weights_csv(&file, None).await.unwrap();
            db.reset().await.unwrap();
            csv_handler.import_weights_csv(&file, None).await.unwrap();
            let new_weight = &db.weights(None).await.unwrap()[0];
            assert_eq!(new_weight.date, weight.date);
            assert_eq!(new_weight.weight, weight.weight);
        });
    }
}

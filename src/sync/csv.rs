use crate::core::Database;
use gio::FileExt;
use gtk::prelude::*;
use std::convert::TryFrom;

pub struct CSVHandler {
    db: Database,
}

impl CSVHandler {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn export_activities_csv(&self, file: &gio::File) -> Result<(), glib::Error> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        for activity in self.db.get_activities(None).await? {
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

    pub async fn export_weights_csv(&self, file: &gio::File) -> Result<(), glib::Error> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        for weight in self.db.get_weights(None).await? {
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

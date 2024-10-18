/* export_dialog.rs
 *
 * Copyright 2021 Rasmus Thomsen <oss@cogitri.dev>
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

use crate::core::i18n;
use gtk::{glib, prelude::ToValue};

mod imp {
    use crate::{
        core::i18n, prelude::*, sync::csv::CsvHandler,
        windows::import_export_dialog_base::ImportExportDialogBase,
    };
    use gtk::{gio, glib, prelude::*, subclass::prelude::*};
    use gtk_macros::spawn;

    #[derive(Debug, Default)]
    pub struct ExportDialog {}

    #[glib::object_subclass]
    impl ObjectSubclass for ExportDialog {
        const NAME: &'static str = "HealthExportDialog";
        type ParentType = ImportExportDialogBase;
        type Type = super::ExportDialog;
    }

    impl ObjectImpl for ExportDialog {}
    impl WidgetImpl for ExportDialog {}
    impl WindowImpl for ExportDialog {}
    impl DialogImpl for ExportDialog {}
    impl ImportExportDialogBaseImpl for ExportDialog {
        fn on_activities(
            &self,
            obj: &ImportExportDialogBase,
            password: Option<String>,
        ) -> PinnedResultFuture<()> {
            let file_chooser = gtk::FileDialog::builder()
                .title(i18n("Save Activities"))
                .accept_label(i18n("_Save"))
                .modal(true)
                .initial_name(if password.is_some() {
                    // TRANSLATORS: Please keep the file extension (.csv.encrypted)
                    i18n("Activities.csv.encrypted")
                } else {
                    // TRANSLATORS: Please keep the file extension (.csv)
                    i18n("Activities.csv")
                })
                .build();

            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(
                    #[weak]
                    obj,
                    move |_, _, send| {
                        let obj = obj.clone().upcast::<gtk::Window>();
                        spawn!(async move {
                            let res = file_chooser.save_future(Some(&obj)).await;
                            if let Ok(file) = res {
                                let pass = password.clone();
                                let handler = CsvHandler::new();
                                send.resolve(
                                    handler.export_activities_csv(&file, pass.as_deref()).await,
                                )
                            } else {
                                send.resolve(Err(anyhow::anyhow!(i18n("No file selected."))))
                            }
                        });
                    }
                ),
            ))
        }

        fn on_weights(
            &self,
            obj: &ImportExportDialogBase,
            password: Option<String>,
        ) -> PinnedResultFuture<()> {
            let file_chooser = gtk::FileDialog::builder()
                .title(i18n("Save Weight Measurement"))
                .accept_label(i18n("_Save"))
                .modal(true)
                .initial_name(if password.is_some() {
                    // TRANSLATORS: Please keep the file extension (.csv.encrypted)
                    i18n("Weight Measurements.csv.encrypted")
                } else {
                    // TRANSLATORS: Please keep the file extension (.csv)
                    i18n("Weight Measurements.csv")
                })
                .build();

            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(
                    #[weak]
                    obj,
                    move |_, _, send| {
                        let obj = obj.clone().upcast::<gtk::Window>();
                        spawn!(async move {
                            let res = file_chooser.save_future(Some(&obj)).await;
                            if let Ok(file) = res {
                                let pass = password.clone();
                                let handler = CsvHandler::new();
                                send.resolve(
                                    handler.export_weights_csv(&file, pass.as_deref()).await,
                                )
                            } else {
                                send.resolve(Err(anyhow::anyhow!(i18n("No file selected."))))
                            }
                        });
                    }
                ),
            ))
        }
    }
}

glib::wrapper! {
    /// A dialog for exporting data
    pub struct ExportDialog(ObjectSubclass<imp::ExportDialog>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog, crate::windows::import_export_dialog_base::ImportExportDialogBase,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl ExportDialog {
    /// Create a new [ExportDialog]
    ///
    /// # Arguments
    /// * `parent` - The [GtkWindow](gtk::Window) which is the transient parent of this dialog.
    pub fn new(parent: Option<&gtk::Window>) -> Self {
        glib::Object::builder()
            .property("use-header-bar", 1.to_value())
            .property("title", i18n("Export data"))
            .property("is-import", false)
            .property("transient-for", parent)
            .build()
    }
}

#[cfg(test)]
mod test {
    use super::ExportDialog;
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();

        ExportDialog::new(None);
    }
}

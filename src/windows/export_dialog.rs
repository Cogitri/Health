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

use crate::i18n;
use gtk::prelude::*;

mod imp {
    use crate::{
        i18n::i18n,
        sync::csv::CsvHandler,
        windows::import_export_dialog_base::{
            ImportExportDialogBase, ImportExportDialogBaseImpl, PinnedResultFuture,
        },
    };
    use gtk::{prelude::*, subclass::prelude::*};
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
            password: Box<Option<String>>,
        ) -> PinnedResultFuture {
            let file_chooser = gtk::FileChooserNativeBuilder::new()
                .title(&i18n("Save Activities"))
                .accept_label(&i18n("_Save"))
                .cancel_label(&i18n("_Cancel"))
                .modal(true)
                .transient_for(obj)
                .action(gtk::FileChooserAction::Save)
                .build();
            if password.is_some() {
                file_chooser.set_current_name(&i18n("Activities.csv.encrypted"));
            } else {
                file_chooser.set_current_name(&i18n("Activities.csv"));
            }

            Box::pin(gio::GioFuture::new(obj, move |_, _, send| {
                spawn!(async move {
                    let res = file_chooser.run_future().await;
                    if res == gtk::ResponseType::Accept {
                        let file = file_chooser.file().unwrap();
                        let pass = password.clone();
                        let handler = CsvHandler::new();
                        send.resolve(handler.export_activities_csv(&file, pass.as_deref()).await)
                    } else {
                        send.resolve(Ok(()))
                    }
                });
            }))
        }

        fn on_weights(
            &self,
            obj: &ImportExportDialogBase,
            password: Box<Option<String>>,
        ) -> PinnedResultFuture {
            let file_chooser = gtk::FileChooserNativeBuilder::new()
                .title(&i18n("Save Weight Measurement"))
                .accept_label(&i18n("_Save"))
                .cancel_label(&i18n("_Cancel"))
                .modal(true)
                .transient_for(obj)
                .action(gtk::FileChooserAction::Save)
                .build();
            if password.is_some() {
                file_chooser.set_current_name(&i18n("Weight Measurements.csv.encrypted"));
            } else {
                file_chooser.set_current_name(&i18n("Weight Measurements.csv"));
            }

            Box::pin(gio::GioFuture::new(obj, move |_, _, send| {
                spawn!(async move {
                    let res = file_chooser.run_future().await;
                    if res == gtk::ResponseType::Accept {
                        let file = file_chooser.file().unwrap();
                        let pass = password.clone();
                        let handler = CsvHandler::new();
                        send.resolve(handler.export_weights_csv(&file, pass.as_deref()).await)
                    } else {
                        send.resolve(Ok(()))
                    }
                });
            }))
        }
    }
}

glib::wrapper! {
    /// A dialog for exporting data
    pub struct ExportDialog(ObjectSubclass<imp::ExportDialog>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog, crate::windows::import_export_dialog_base::ImportExportDialogBase;
}

impl ExportDialog {
    /// Create a new [ExportDialog]
    ///
    /// # Arguments
    /// * `parent` - The [GtkWindow](gtk::Window) which is the transient parent of this dialog.
    pub fn new(parent: Option<&gtk::Window>) -> Self {
        let o: Self = glib::Object::new(&[
            ("use-header-bar", &1),
            ("title", &i18n("Export data")),
            ("is-import", &false),
        ])
        .expect("Failed to create ExportDialog");

        o.set_transient_for(parent);

        o
    }
}

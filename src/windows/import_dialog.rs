/* import_dialog.rs
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
use gtk::glib;

mod imp {
    use crate::{
        core::i18n, prelude::*, sync::csv::CsvHandler,
        windows::import_export_dialog_base::ImportExportDialogBase,
    };
    use gtk::{gio, glib, prelude::*, subclass::prelude::*};
    use gtk_macros::spawn;

    #[derive(Debug, Default)]
    pub struct ImportDialog {}

    #[glib::object_subclass]
    impl ObjectSubclass for ImportDialog {
        const NAME: &'static str = "HealthImportDialog";
        type ParentType = ImportExportDialogBase;
        type Type = super::ImportDialog;
    }

    impl ObjectImpl for ImportDialog {}
    impl WidgetImpl for ImportDialog {}
    impl WindowImpl for ImportDialog {}
    impl DialogImpl for ImportDialog {}
    impl ImportExportDialogBaseImpl for ImportDialog {
        fn on_activities(
            &self,
            obj: &ImportExportDialogBase,
            password: Option<String>,
        ) -> PinnedResultFuture<()> {
            let file_chooser = gtk::FileChooserNative::builder()
                .title(&i18n("Open Activities"))
                .accept_label(&i18n("_Open"))
                .cancel_label(&i18n("_Cancel"))
                .modal(true)
                .transient_for(obj)
                .action(gtk::FileChooserAction::Open)
                .build();

            Box::pin(gio::GioFuture::new(obj, move |_, _, send| {
                spawn!(async move {
                    let res = file_chooser.run_future().await;
                    if res == gtk::ResponseType::Accept {
                        let file = file_chooser.file().unwrap();
                        let pass = password.clone();
                        let handler = CsvHandler::new();
                        send.resolve(handler.import_activities_csv(&file, pass.as_deref()).await)
                    } else {
                        send.resolve(Ok(()))
                    }
                });
            }))
        }

        fn on_weights(
            &self,
            obj: &ImportExportDialogBase,
            password: Option<String>,
        ) -> PinnedResultFuture<()> {
            let file_chooser = gtk::FileChooserNative::builder()
                .title(&i18n("Open Weight Measurement"))
                .accept_label(&i18n("_Open"))
                .cancel_label(&i18n("_Cancel"))
                .modal(true)
                .transient_for(obj)
                .action(gtk::FileChooserAction::Open)
                .build();

            Box::pin(gio::GioFuture::new(obj, move |_, _, send| {
                spawn!(async move {
                    let res = file_chooser.run_future().await;
                    if res == gtk::ResponseType::Accept {
                        let file = file_chooser.file().unwrap();
                        let pass = password.clone();
                        let handler = CsvHandler::new();
                        send.resolve(handler.import_weights_csv(&file, pass.as_deref()).await)
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
    pub struct ImportDialog(ObjectSubclass<imp::ImportDialog>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog, crate::windows::import_export_dialog_base::ImportExportDialogBase,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl ImportDialog {
    /// Create a new [ImportDialog]
    ///
    /// # Arguments
    /// * `parent` - The [GtkWindow](gtk::Window) which is the transient parent of this dialog.
    pub fn new(parent: Option<&gtk::Window>) -> Self {
        glib::Object::new(&[
            ("use-header-bar", &1),
            ("is-import", &true),
            ("title", &i18n("Import data")),
            ("transient-for", &parent),
        ])
        .expect("Failed to create ImportDialog")
    }
}

#[cfg(test)]
mod test {
    use super::ImportDialog;
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();

        ImportDialog::new(None);
    }
}

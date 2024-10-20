/* sync_list_box.rs
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

use crate::core::i18n;
use crate::sync::{
    google_fit::GoogleFitSyncProvider, new_db_receiver, sync_provider::SyncProvider,
};
use adw::prelude::*;
use anyhow::Result;
use gtk::glib::{self, clone, g_warning, subclass::prelude::*};
use gtk_macros::spawn;

mod imp {
    use crate::core::Settings;
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{glib, CompositeTemplate};
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/sync_list_box.ui")]
    pub struct SyncListBox {
        pub parent_window: RefCell<Option<gtk::Window>>,

        #[template_child]
        pub google_fit_selected_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub google_fit_start_sync_row: TemplateChild<gtk::ListBoxRow>,
        #[template_child]
        pub google_fit_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub google_fit_spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub sync_list_box: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SyncListBox {
        const NAME: &'static str = "HealthSyncListBox";
        type ParentType = adw::Bin;
        type Type = super::SyncListBox;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SyncListBox {
        fn constructed(&self) {
            self.parent_constructed();

            if Settings::instance().sync_provider_setup_google_fit() {
                self.google_fit_selected_image.set_visible(true);
                self.google_fit_stack
                    .set_visible_child(&self.google_fit_selected_image.get());
                self.google_fit_stack.set_visible(true);
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::builder::<gtk::Window>("parent-window")
                        .construct()
                        .readwrite()
                        .build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "parent-window" => {
                    self.parent_window.replace(value.get().unwrap());
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "parent-window" => self.parent_window.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for SyncListBox {}
    impl BinImpl for SyncListBox {}
}

glib::wrapper! {
    /// The [SyncListBox] is a [gtk::ListBox] where users can initialise synching with a third-party provider.
    pub struct SyncListBox(ObjectSubclass<imp::SyncListBox>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl SyncListBox {
    /// Create a new [SyncListBox]
    ///
    /// # Arguments
    /// * `parent_window` - The [GtkWindow](gtk::Window) that should be the transient parent for error dialogs (or none).
    pub fn new(parent_window: Option<gtk::Window>) -> Self {
        glib::Object::builder()
            .property("parent-window", &parent_window)
            .build()
    }

    fn handle_db_receiver_received(&self, res: Result<()>) -> glib::ControlFlow {
        let imp = self.imp();
        if let Err(e) = res {
            imp.google_fit_selected_image
                .set_icon_name(Some("network-error-symbolic"));
            imp.google_fit_selected_image.set_visible(true);
            imp.google_fit_spinner.set_spinning(false);
            imp.google_fit_stack
                .set_visible_child(&imp.google_fit_selected_image.get());

            self.open_sync_error(&e.to_string());
        } else {
            let obj = self.clone();
            spawn!(async move {
                let imp = obj.imp();
                imp.google_fit_selected_image.set_visible(true);
                imp.google_fit_spinner.set_spinning(false);
                imp.google_fit_stack
                    .set_visible_child(&imp.google_fit_selected_image.get());
            });
        }

        imp.google_fit_start_sync_row.set_activatable(false);
        glib::ControlFlow::Break
    }

    #[template_callback]
    fn handle_row_activated(&self, row: gtk::ListBoxRow) {
        let imp = self.imp();
        if row == imp.google_fit_start_sync_row.get() {
            imp.google_fit_stack.set_visible(true);
            imp.google_fit_spinner.set_visible(true);
            imp.google_fit_spinner.set_spinning(true);
            imp.google_fit_start_sync_row.set_activatable(false);
            imp.google_fit_stack
                .set_visible_child(&imp.google_fit_spinner.get());

            let (sender, receiver) = async_channel::unbounded();
            let db_sender = new_db_receiver();

            glib::spawn_future_local(clone!(
                #[weak(rename_to = obj)]
                self,
                #[upgrade_or_panic]
                async move {
                    while let Ok(res) = receiver.recv().await {
                        if obj.handle_db_receiver_received(res) == glib::ControlFlow::Break {
                            break;
                        }
                    }
                }
            ));

            std::thread::spawn(move || {
                let mut sync_provider = GoogleFitSyncProvider::new(db_sender);
                if let Err(e) = sync_provider.initial_authenticate() {
                    sender.send_blocking(Err(e)).unwrap();
                } else {
                    if let Err(e) = sync_provider.initial_import() {
                        sender.send_blocking(Err(e)).unwrap();
                    }

                    sender.send_blocking(Ok(())).unwrap();
                }
            });
        }
    }

    fn open_sync_error(&self, errmsg: &str) {
        g_warning!(crate::config::LOG_DOMAIN, "{errmsg}");

        let dialog = adw::AlertDialog::builder()
            .heading(i18n("Sync Error"))
            .body(errmsg)
            .build();
        dialog.add_response("close", &i18n("Close"));
        dialog.set_response_appearance("close", adw::ResponseAppearance::Destructive);
        dialog.present(self.imp().parent_window.borrow().as_ref());
    }
}

#[cfg(test)]
mod test {
    use super::SyncListBox;
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();
        SyncListBox::new(None);
    }
}

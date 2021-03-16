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

use crate::sync::{
    google_fit::GoogleFitSyncProvider,
    new_db_receiver,
    sync_provider::{SyncProvider, SyncProviderError},
};
use glib::{clone, g_warning, subclass::prelude::*};
use gtk::prelude::*;
use gtk_macros::spawn;

mod imp {
    use crate::core::Settings;
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate)]
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
        type ParentType = gtk::Widget;
        type Type = super::SyncListBox;

        fn new() -> Self {
            Self {
                parent_window: RefCell::new(None),
                google_fit_selected_image: TemplateChild::default(),
                google_fit_start_sync_row: TemplateChild::default(),
                google_fit_stack: TemplateChild::default(),
                google_fit_spinner: TemplateChild::default(),
                sync_list_box: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SyncListBox {
        fn constructed(&self, obj: &Self::Type) {
            if Settings::get_instance().get_sync_provider_setup_google_fit() {
                self.google_fit_selected_image.set_visible(true);
                self.google_fit_stack
                    .set_visible_child(&self.google_fit_selected_image.get());
                self.google_fit_stack.set_visible(true);
            }

            obj.connect_handlers();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::object(
                    "parent-window",
                    "parent-window",
                    "parent-window",
                    gtk::Window::static_type(),
                    glib::ParamFlags::READWRITE,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.get_name() {
                "parent-window" => {
                    self.parent_window.replace(value.get().unwrap());
                }
                _ => unimplemented!(),
            }
        }

        fn get_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            pspec: &glib::ParamSpec,
        ) -> glib::Value {
            match pspec.get_name() {
                "parent-window" => self.parent_window.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for SyncListBox {}
    impl ListBoxRowImpl for SyncListBox {}
}

glib::wrapper! {
    /// The [SyncListBox] is a [gtk::ListBox] where users can initialise synching with a third-party provider.
    pub struct SyncListBox(ObjectSubclass<imp::SyncListBox>) @extends gtk::Widget, gtk::ListBoxRow;
}

impl SyncListBox {
    pub fn new(parent_window: Option<gtk::Window>) -> Self {
        let o: Self = glib::Object::new(&[]).expect("Failed to create SyncListBox");

        o.get_priv().parent_window.replace(parent_window);

        o
    }

    fn connect_handlers(&self) {
        self.get_priv().sync_list_box.connect_row_activated(
            glib::clone!(@weak self as obj => move |_, row| {
                obj.handle_row_activated(row);
            }),
        );
    }

    fn get_priv(&self) -> &imp::SyncListBox {
        imp::SyncListBox::from_instance(self)
    }

    fn handle_row_activated(self, row: &gtk::ListBoxRow) {
        let self_ = self.get_priv();
        if row == &self_.google_fit_start_sync_row.get() {
            self_.google_fit_stack.set_visible(true);
            self_.google_fit_spinner.set_visible(true);
            self_.google_fit_spinner.set_spinning(true);
            self_.google_fit_start_sync_row.set_activatable(false);
            self_
                .google_fit_stack
                .set_visible_child(&self_.google_fit_spinner.get());

            let (sender, receiver) =
                glib::MainContext::channel::<Result<(), SyncProviderError>>(glib::PRIORITY_DEFAULT);
            let db_sender = new_db_receiver();

            receiver.attach(None, clone!(@weak self as obj => move |res| {
                let self_ = obj.get_priv();
                if let Err(e) = res {
                    self_.google_fit_selected_image.set_property_icon_name(Some("network-error-symbolic"));
                    self_.google_fit_selected_image.set_visible(true);
                    self_.google_fit_spinner.set_spinning(false);
                    self_.google_fit_stack.set_visible_child(&self_.google_fit_selected_image.get());

                    obj.open_sync_error(&e.to_string());
                } else {
                    let obj = obj.clone();
                    spawn!(async move {
                        let self_ = obj.get_priv();
                        self_.google_fit_selected_image.set_visible(true);
                        self_.google_fit_spinner.set_spinning(false);
                        self_.google_fit_stack.set_visible_child(&self_.google_fit_selected_image.get());
                    });
                }

                self_.google_fit_start_sync_row.set_activatable(false);
                glib::Continue(false)
            }));

            std::thread::spawn(move || {
                let mut sync_provider = GoogleFitSyncProvider::new(db_sender);
                if let Err(e) = sync_provider.initial_authenticate() {
                    sender.send(Err(e)).unwrap();
                } else {
                    if let Err(e) = sync_provider.initial_import() {
                        sender.send(Err(e)).unwrap();
                    }

                    sender.send(Ok(())).unwrap();
                }
            });
        }
    }

    fn open_sync_error(&self, errmsg: &str) {
        g_warning!(crate::config::LOG_DOMAIN, "{}", errmsg);

        let dialog = gtk::MessageDialog::new(
            self.get_priv().parent_window.borrow().as_ref(),
            gtk::DialogFlags::DESTROY_WITH_PARENT | gtk::DialogFlags::MODAL,
            gtk::MessageType::Error,
            gtk::ButtonsType::Close,
            errmsg,
        );
        dialog.connect_response(|d, _| {
            d.destroy();
        });
        dialog.show();
    }
}

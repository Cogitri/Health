/* data_add_dialog.rs
 *
 * Copyright 2021 Visvesh Subramanian <visveshs.blogspot.com>
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
    prelude::*,
    views::{ViewAdd, ViewAddActivity, ViewAddWeight},
};
use gtk::{
    glib::{self, subclass::prelude::*, translate::FromGlib},
    prelude::*,
};

mod imp {
    use crate::{
        prelude::*,
        views::{ViewAdd, ViewAddActivity, ViewAddWeight},
    };
    use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
    use once_cell::unsync::OnceCell;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/data_add_dialog.ui")]
    pub struct DataAddDialog {
        pub current_plugin: OnceCell<String>,
        #[template_child]
        pub stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub button_cancel: TemplateChild<gtk::Button>,
        #[template_child]
        pub button_ok: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DataAddDialog {
        const NAME: &'static str = "HealthDataAddDialog";
        type ParentType = gtk::Dialog;
        type Type = super::DataAddDialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DataAddDialog {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.add_action_widget(&*self.button_cancel, gtk::ResponseType::Cancel);
            obj.add_action_widget(&*self.button_ok, gtk::ResponseType::Ok);

            for stack_page in &[
                ViewAddActivity::new().upcast::<ViewAdd>(),
                ViewAddWeight::new().upcast::<ViewAdd>(),
            ] {
                self.stack
                    .add_titled(
                        stack_page,
                        Some(stack_page.widget_name().as_str()),
                        &stack_page.view_title(),
                    )
                    .set_icon_name(Some(&stack_page.icon_name()));
                stack_page.connect_is_responsive_notify(glib::clone!(@weak obj => move |v| {
                    obj.handle_is_responsive_notify(v);
                }));
            }
            if &obj.property::<String>("current-plugin") == "weight" {
                self.stack.set_visible_child_name("Add Weight Data");
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecString::builder("current-plugin")
                    .construct_only()
                    .readwrite()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "current-plugin" => self.current_plugin.get().unwrap().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "current-plugin" => self.current_plugin.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for DataAddDialog {}
    impl WindowImpl for DataAddDialog {}
    impl DialogImpl for DataAddDialog {}
}

glib::wrapper! {
    /// popup dialog box that adds activity/weight/water intake data .
    pub struct DataAddDialog(ObjectSubclass<imp::DataAddDialog>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl DataAddDialog {
    /// Create a new [DataAddDialog].
    ///
    /// # Arguments
    /// * `parent` - The [GtkWindow](gtk::Window) who is the transient parent of this dialog.
    pub fn new(parent: &gtk::Window, current_plugin: String) -> Self {
        glib::Object::new(&[
            ("use-header-bar", &1.to_value()),
            ("transient-for", &Some(parent)),
            ("current-plugin", &current_plugin),
        ])
    }

    fn handle_is_responsive_notify(&self, view: &ViewAdd) {
        let imp = self.imp();
        if imp.stack.visible_child_name() == Some(view.widget_name()) {
            self.set_response_sensitive(gtk::ResponseType::Ok, view.is_responsive());
        }
    }

    #[template_callback]
    fn handle_response(&self, id: i32) {
        let id = unsafe { gtk::ResponseType::from_glib(id) };
        let imp = self.imp();
        let active_stack_page_name = imp.stack.visible_child_name().unwrap().to_string();
        if let Some(active_stack_page) = imp.stack.visible_child() {
            match active_stack_page_name.as_ref() {
                "Add Activity Data" => {
                    if let Result::Ok(activity_add_page) =
                        active_stack_page.downcast::<ViewAddActivity>()
                    {
                        glib::MainContext::default().spawn_local(async move {
                            activity_add_page.handle_response(id).await
                        });
                    }
                }
                "Add Weight Data" => {
                    if let Result::Ok(weight_add_page) =
                        active_stack_page.downcast::<ViewAddWeight>()
                    {
                        glib::MainContext::default()
                            .spawn_local(async move { weight_add_page.handle_response(id).await });
                    }
                }
                _ => unimplemented!(),
            };
            self.destroy();
        }
    }

    #[template_callback]
    fn handle_stack_visible_child_notify(&self) {
        let imp = self.imp();
        if let Some(w) = imp.stack.visible_child() {
            self.set_response_sensitive(
                gtk::ResponseType::Ok,
                w.downcast_ref::<ViewAdd>().unwrap().is_responsive(),
            );
        }
    }
}

#[cfg(test)]
mod test {
    use super::DataAddDialog;
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();

        DataAddDialog::new(&gtk::Window::new(), String::new());
    }
}

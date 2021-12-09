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

use crate::views::{View, ViewAddActivity, ViewAddWeight};
use gtk::{
    glib::{self, subclass::prelude::*},
    prelude::*,
};

mod imp {
    use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/data_add_dialog.ui")]
    pub struct DataAddDialog {
        #[template_child]
        pub stack: TemplateChild<adw::ViewStack>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DataAddDialog {
        const NAME: &'static str = "HealthDataAddDialog";
        type ParentType = gtk::Dialog;
        type Type = super::DataAddDialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DataAddDialog {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.connect_handlers();
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

impl DataAddDialog {
    /// Create a new [DataAddDialog].
    ///
    /// # Arguments
    /// * `parent` - The [GtkWindow](gtk::Window) who is the transient parent of this dialog.
    pub fn new(parent: &gtk::Window, current_plugin: String) -> Self {
        let o: Self =
            glib::Object::new(&[("use-header-bar", &1)]).expect("Failed to create DataAddDialog");

        o.set_transient_for(Some(parent));

        let self_ = o.imp();
        for stack_page in &[
            ViewAddActivity::new().upcast::<View>(),
            ViewAddWeight::new().upcast::<View>(),
        ] {
            stack_page.stack().set_visible_child_name("add_data_page");
            self_
                .stack
                .add_titled(
                    stack_page,
                    Some(stack_page.widget_name().as_str()),
                    &stack_page.view_title().unwrap(),
                )
                .unwrap()
                .set_icon_name(stack_page.icon_name().as_deref());
        }
        if current_plugin == "weight" {
            self_.stack.set_visible_child_name("Add Weight Data");
        }

        o
    }

    fn connect_handlers(&self) {
        self.connect_response(Self::handle_response);
    }

    fn handle_response(&self, id: gtk::ResponseType) {
        let self_ = self.imp();
        let active_stack_page_name = self_.stack.visible_child_name().unwrap().to_string();
        if let Some(active_stack_page) = self_.stack.visible_child() {
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

    fn imp(&self) -> &imp::DataAddDialog {
        imp::DataAddDialog::from_instance(self)
    }
}

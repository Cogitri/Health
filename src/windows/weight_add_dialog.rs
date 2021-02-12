/* weight_add_dialog.rs
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

use crate::{
    core::{i18n, settings::Unitsystem, Database},
    model::Weight,
};
use glib::{clone, subclass::prelude::*};
use gtk::prelude::*;
use gtk_macros::spawn;
use uom::si::{
    f32::Mass,
    mass::{kilogram, pound},
};

mod imp {
    use crate::{
        core::{Database, Settings},
        widgets::DateSelector,
    };
    use glib::subclass;
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use once_cell::unsync::OnceCell;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/weight_add_dialog.ui")]
    pub struct WeightAddDialog {
        pub database: OnceCell<Database>,
        pub settings: Settings,

        #[template_child]
        pub date_selector: TemplateChild<DateSelector>,
        #[template_child]
        pub weight_spin_button: TemplateChild<gtk::SpinButton>,
    }

    impl ObjectSubclass for WeightAddDialog {
        const NAME: &'static str = "HealthWeightAddDialog";
        type ParentType = gtk::Dialog;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::WeightAddDialog;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                database: OnceCell::new(),
                settings: Settings::new(),
                date_selector: TemplateChild::default(),
                weight_spin_button: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WeightAddDialog {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.set_response_sensitive(gtk::ResponseType::Ok, false);
            obj.update_title();
            obj.connect_handlers();
        }
    }

    impl WidgetImpl for WeightAddDialog {}
    impl WindowImpl for WeightAddDialog {}
    impl DialogImpl for WeightAddDialog {}
}

glib::wrapper! {
    pub struct WeightAddDialog(ObjectSubclass<imp::WeightAddDialog>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog;
}

impl WeightAddDialog {
    pub fn new(database: Database, parent: &gtk::Window) -> Self {
        let o: Self =
            glib::Object::new(&[("use-header-bar", &1)]).expect("Failed to create WeightAddDialog");

        o.set_transient_for(Some(parent));
        o.get_priv().database.set(database).unwrap();

        o
    }

    fn connect_handlers(&self) {
        let self_ = self.get_priv();

        self_
            .weight_spin_button
            .connect_changed(clone!(@weak self as obj => move |e| {
                let text = e.get_text ().to_string();
                obj.set_response_sensitive(gtk::ResponseType::Ok, text != "0" && !text.is_empty());
            }));

        self.connect_response(|obj, id| match id {
            gtk::ResponseType::Ok => {
                let downgraded = obj.downgrade();
                spawn!(async move {
                    if let Some(obj) = downgraded.upgrade() {
                        let self_ = obj.get_priv();
                        let value = if self_.settings.get_unitsystem() == Unitsystem::Metric {
                            Mass::new::<kilogram>(self_.weight_spin_button.get_value() as f32)
                        } else {
                            Mass::new::<pound>(self_.weight_spin_button.get_value() as f32)
                        };
                        if let Err(e) = self_
                            .database
                            .get()
                            .unwrap()
                            .save_weight(Weight::new(
                                self_.date_selector.get_selected_date(),
                                value,
                            ))
                            .await
                        {
                            glib::g_warning!(
                                crate::config::LOG_DOMAIN,
                                "Failed to save new data due to error {}",
                                e.to_string()
                            )
                        }

                        obj.destroy();
                    }
                });
            }
            _ => {
                obj.destroy();
            }
        });
    }

    fn get_priv(&self) -> &imp::WeightAddDialog {
        imp::WeightAddDialog::from_instance(self)
    }

    fn update_title(&self) {
        let downgraded = self.downgrade();
        glib::MainContext::default().spawn_local(async move {
            if let Some(obj) = downgraded.upgrade() {
                let self_ = obj.get_priv();
                let res = self_
                    .database
                    .get()
                    .unwrap()
                    .get_weight_exists_on_date(self_.date_selector.get_selected_date())
                    .await;
                if let Ok(true) = res {
                    obj.set_title(Some(&i18n("Update Weight Measurement")));
                } else {
                    if let Err(e) = res {
                        glib::g_warning!(crate::config::LOG_DOMAIN, "{}", e.to_string());
                    }
                    obj.set_title(Some(&i18n("Add New Weight Measurement")));
                }
            }
        })
    }
}

/* view_add_weight.rs
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
    core::{i18n, UnitSystem},
    model::Weight,
    prelude::*,
    views::ViewAdd,
};
use gtk::glib::{self, subclass::prelude::*};
use uom::si::{
    f32::Mass,
    mass::{kilogram, pound},
};

mod imp {
    use crate::{
        core::{Database, Settings},
        views::ViewAdd,
        widgets::{DateSelector, UnitSpinButton},
    };
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{glib, CompositeTemplate};

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/view_add_weight.ui")]
    pub struct ViewAddWeight {
        pub database: Database,
        pub settings: Settings,

        #[template_child]
        pub date_selector: TemplateChild<DateSelector>,
        #[template_child]
        pub weight_spin_button: TemplateChild<UnitSpinButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ViewAddWeight {
        const NAME: &'static str = "HealthViewAddWeight";
        type ParentType = ViewAdd;
        type Type = super::ViewAddWeight;

        fn class_init(klass: &mut Self::Class) {
            UnitSpinButton::static_type();
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ViewAddWeight {}
    impl WidgetImpl for ViewAddWeight {}
    impl BinImpl for ViewAddWeight {}
}

glib::wrapper! {
    /// A few widgets for adding a new weight record.
    pub struct ViewAddWeight(ObjectSubclass<imp::ViewAddWeight>)
        @extends gtk::Widget, adw::Bin, ViewAdd,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl ViewAddWeight {
    /// Create a new [ViewAddWeight]
    pub fn new() -> Self {
        let ret: Self = glib::Object::builder()
            .property("icon-name", "weight-scale-symbolic")
            .property("view-title", i18n("Weight"))
            .build();

        // Restore spin buttton to latest weight
        let db = crate::core::Database::default();
        if let Some(latest) = db.weight_latest() {
            ret.imp()
                .weight_spin_button
                .set_value(f64::from(latest.weight.get::<kilogram>()));
        }
        ret
    }

    pub async fn handle_response(&self, id: gtk::ResponseType) {
        if id == gtk::ResponseType::Ok {
            let imp = self.imp();
            let value = if imp.settings.unit_system() == UnitSystem::Metric {
                Mass::new::<kilogram>(imp.weight_spin_button.value() as f32)
            } else {
                Mass::new::<pound>(imp.weight_spin_button.value() as f32)
            };
            if let Err(e) = imp
                .database
                .save_weight(Weight::new(imp.date_selector.selected_date(), value))
                .await
            {
                glib::g_warning!(
                    crate::config::LOG_DOMAIN,
                    "Failed to save new data due to error {e}",
                )
            }
        }
    }

    #[template_callback]
    fn handle_weight_spin_button_changed(&self) {
        let imp = self.imp();
        self.set_is_responsive(imp.weight_spin_button.raw_value().unwrap_or(0.0) != 0.0);
    }
}

#[cfg(test)]
mod test {
    use super::ViewAddWeight;
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();
        ViewAddWeight::new();
    }
}

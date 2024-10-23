/* date_selector.rs
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

use gtk::{
    glib::{self, SignalHandlerId},
    prelude::*,
};

mod imp {
    use crate::prelude::*;
    use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/date_selector.ui")]
    pub struct DateSelector {
        #[template_child]
        pub day_adjustment: TemplateChild<gtk::Adjustment>,
        #[template_child]
        pub day_spinner: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub month_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub year_spinner: TemplateChild<gtk::SpinButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DateSelector {
        const NAME: &'static str = "HealthDateSelector";
        type ParentType = gtk::Box;
        type Type = super::DateSelector;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DateSelector {
        fn constructed(&self) {
            self.parent_constructed();

            let now = glib::DateTime::local();
            self.obj().set_selected_date(now.clone());
            self.day_adjustment
                .set_upper(glib::DateTime::days_of_month(now.year(), now.month()) as f64);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecBoxed::builder::<glib::DateTime>("selected-date").build()]
            });
            &PROPERTIES
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let obj = self.obj();

            match pspec.name() {
                "selected-date" => {
                    let year = self.year_spinner.value_as_int();
                    // The dropdown starts counting from 0, not 1.
                    let month: i32 = (self.month_dropdown.selected() + 1)
                        .clamp(1, 12)
                        .try_into()
                        .unwrap();
                    let mut day = self.day_spinner.raw_value().unwrap_or(1).clamp(1, 31);
                    match glib::DateTime::from_local(year, month, day, 0, 0, 0.0) {
                        Ok(o) => o.to_value(),
                        Err(_) => {
                            if day > 28 {
                                while glib::DateTime::from_local(year, month, day, 0, 0, 0.0)
                                    .is_err()
                                {
                                    day -= 1;
                                    if day <= 28 {
                                        break;
                                    }
                                }
                                if let Ok(d) =
                                    glib::DateTime::from_local(year, month, day, 0, 0, 0.0)
                                {
                                    obj.set_property("selected-date", d.clone());
                                    return d.to_value();
                                }
                            }
                            unimplemented!();
                        }
                    }
                }
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "selected-date" => {
                    let datetime = value.get::<glib::DateTime>().unwrap();

                    let now = glib::DateTime::local();
                    let date = if datetime.reset_hms() > now.reset_hms() {
                        now
                    } else {
                        datetime
                    };

                    self.day_adjustment
                        .set_upper(glib::DateTime::days_of_month(date.year(), date.month()) as f64);
                    self.day_spinner.set_value(date.day_of_month().into());
                    self.month_dropdown
                        .set_selected((date.month() - 1).try_into().unwrap());
                    self.year_spinner.set_value(date.year().into());
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for DateSelector {}
    impl BoxImpl for DateSelector {}
}

glib::wrapper! {
    /// A widget to select a date via a [gtk::Calendar] or by entering a date into a [gtk::Entry].
    pub struct DateSelector(ObjectSubclass<imp::DateSelector>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

#[gtk::template_callbacks]
impl DateSelector {
    /// Connect to a new date being selected.
    ///
    /// # Arguments
    /// * `callback` - The callback to call once the ::notify signal is emitted.
    ///
    /// # Returns
    /// The [glib::SignalHandlerId] to disconnect the signal later on.
    pub fn connect_selected_date_notify<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId {
        self.connect_notify_local(Some("selected-date"), move |s, _| f(s))
    }

    /// Get the currently selected date
    pub fn selected_date(&self) -> glib::DateTime {
        self.property::<glib::DateTime>("selected-date")
    }

    #[template_callback]
    pub fn handle_date_widget_changed(&self) {
        self.notify("selected-date");
    }

    /// Create a new [DateSelector]
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Set the currently selected date.
    pub fn set_selected_date(&self, value: glib::DateTime) {
        self.set_property("selected-date", value);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{core::i18n, prelude::*, utils::init_gtk};
    use gtk::subclass::prelude::*;

    #[gtk::test]
    fn new() {
        init_gtk();

        DateSelector::new();
    }

    #[gtk::test]
    fn selected_date() {
        init_gtk();

        let selector = DateSelector::new();
        let selector_ = selector.imp();
        selector_.day_spinner.set_value(17.0);
        selector_.month_dropdown.set_selected(1);
        assert_eq!(
            selector_
                .month_dropdown
                .model()
                .unwrap()
                .downcast_ref::<gtk::StringList>()
                .unwrap()
                .string(1)
                .unwrap(),
            i18n("February")
        );
        selector_.year_spinner.set_value(2007.0);
        assert_eq!(
            selector.selected_date().format("%Y-%m-%d").unwrap(),
            "2007-02-17"
        );
    }

    #[gtk::test]
    fn set_selected_date() {
        init_gtk();
        let selector = DateSelector::new();
        let selector_ = selector.imp();

        let now = Date::new(2007, 2, 17)
            .unwrap()
            .and_time_utc(Time::new(12, 0, 0).unwrap());
        selector.set_selected_date(now.clone());
        assert_eq!(selector_.day_spinner.value() as i32, now.day_of_month());
        assert_eq!(selector_.month_dropdown.selected() as i32, now.month() - 1);
        assert_eq!(selector_.year_spinner.value() as i32, now.year());
        assert_eq!(selector_.day_adjustment.upper(), 28.0);
    }

    #[gtk::test]
    fn set_invalid_day() {
        init_gtk();
        let selector = DateSelector::new();
        let selector_ = selector.imp();

        selector_.day_spinner.set_value(30.0);
        selector_.month_dropdown.set_selected(1);
        selector_.year_spinner.set_value(2000.0);
        assert_eq!(
            selector.selected_date().format("%Y-%m-%d").unwrap(),
            "2000-02-29"
        );

        selector_.day_spinner.set_value(31.0);
        selector_.year_spinner.set_value(2001.0);
        assert_eq!(
            selector.selected_date().format("%Y-%m-%d").unwrap(),
            "2001-02-28"
        );
    }
}

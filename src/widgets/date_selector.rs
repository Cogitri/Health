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

use crate::core::date::prelude::*;
use chrono::{DateTime, FixedOffset, Local, NaiveDate};
use gtk::{
    glib::{self, SignalHandlerId},
    prelude::*,
    subclass::prelude::*,
};

mod imp {
    use crate::{date::DateTimeBoxed, utils::prelude::*};
    use chrono::{Datelike, Local, LocalResult, NaiveDate, TimeZone};
    use gtk::{
        glib::{self, clone},
        prelude::*,
        subclass::prelude::*,
        CompositeTemplate,
    };

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/date_editor.ui")]
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
        type ParentType = gtk::Grid;
        type Type = super::DateSelector;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DateSelector {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.day_spinner
                .connect_changed(clone!(@weak obj => move |_| {
                    obj.handle_date_widget_changed();
                }));
            self.month_dropdown
                .connect_selected_notify(clone!(@weak obj => move |_| {
                    obj.handle_date_widget_changed();
                }));
            self.year_spinner
                .connect_changed(clone!(@weak obj => move |_| {
                    obj.handle_date_widget_changed();
                }));
            let now = Local::now();
            obj.set_selected_date(now.into());
            self.day_adjustment
                .set_upper(obj.get_days_from_month(now.date().year(), now.date().month()) as f64);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_boxed(
                    "selected-date",
                    "selected-date",
                    "selected-date",
                    DateTimeBoxed::static_type(),
                    glib::ParamFlags::READWRITE,
                )]
            });
            &PROPERTIES
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-date" => {
                    let naive_date = NaiveDate::from_ymd(
                        self.year_spinner.raw_value().unwrap_or(0),
                        // The dropdown starts counting from 0, not 1.
                        self.month_dropdown.selected() + 1,
                        self.day_spinner.raw_value().unwrap_or(1),
                    );
                    match Local.from_local_datetime(&naive_date.and_hms(12, 0, 0)) {
                        LocalResult::Single(d) | LocalResult::Ambiguous(d, _) => {
                            DateTimeBoxed(d.into()).to_value()
                        }
                        LocalResult::None => {
                            unimplemented!()
                        }
                    }
                }
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "selected-date" => {
                    let date = value.get::<DateTimeBoxed>().unwrap().0.date();
                    self.day_adjustment
                        .set_upper(obj.get_days_from_month(date.year(), date.month()) as f64);
                    self.day_spinner.set_value(date.day().into());
                    self.month_dropdown.set_selected(date.month() - 1);
                    self.year_spinner.set_value(date.year().into());
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for DateSelector {}
    impl GridImpl for DateSelector {}
}

glib::wrapper! {
    /// A widget to select a date via a [gtk::Calendar] or by entering a date into a [gtk::Entry].
    pub struct DateSelector(ObjectSubclass<imp::DateSelector>)
        @extends gtk::Widget, gtk::Grid,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

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
    pub fn selected_date(&self) -> DateTime<FixedOffset> {
        self.property("selected-date")
            .unwrap()
            .get::<DateTimeBoxed>()
            .unwrap()
            .0
    }

    pub fn handle_date_widget_changed(&self) {
        self.notify("selected-date");
    }

    /// Create a new [DateSelector]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create DateSelector")
    }

    /// Set the currently selected date.
    pub fn set_selected_date(&self, value: DateTime<FixedOffset>) {
        let now: DateTime<FixedOffset> = Local::now().into();
        let datetime = if value.date() > now.date() {
            now
        } else {
            value
        };

        self.set_property("selected-date", DateTimeBoxed(datetime))
            .unwrap();
    }

    fn get_days_from_month(&self, year: i32, month: u32) -> i64 {
        NaiveDate::from_ymd(
            match month {
                12 => year + 1,
                _ => year,
            },
            match month {
                12 => 1,
                _ => month + 1,
            },
            1,
        )
        .signed_duration_since(NaiveDate::from_ymd(year, month, 1))
        .num_days()
    }

    #[allow(dead_code)]
    fn imp(&self) -> &imp::DateSelector {
        imp::DateSelector::from_instance(self)
    }
}

#[cfg(test)]
mod test {
    use chrono::Datelike;

    use super::*;
    use crate::{i18n::i18n, utils::init_gtk};

    #[test]
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
            selector.selected_date().date().naive_local(),
            NaiveDate::from_ymd(2007, 2, 17)
        );
    }

    #[test]
    fn set_selected_date() {
        init_gtk();
        let selector = DateSelector::new();
        let selector_ = selector.imp();

        let now = DateTime::<chrono::Utc>::from_utc(
            NaiveDate::from_ymd(2007, 2, 17).and_hms(12, 0, 0),
            chrono::Utc,
        );
        selector.set_selected_date(now.into());
        assert_eq!(selector_.day_spinner.value() as u32, now.date().day());
        assert_eq!(selector_.month_dropdown.selected(), now.date().month() - 1);
        assert_eq!(selector_.year_spinner.value() as i32, now.date().year());
        assert_eq!(selector_.day_adjustment.upper(), 28.0);
    }
}

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
use chrono::{DateTime, FixedOffset, Local, LocalResult, TimeZone};
use gtk::glib::{self, subclass::prelude::*, SignalHandlerId};
use gtk::prelude::*;

mod imp {
    use crate::date::DateTimeBoxed;
    use chrono::{DateTime, FixedOffset, Local};
    use gtk::{
        glib::{self, clone},
        prelude::*,
        subclass::prelude::*,
        CompositeTemplate,
    };
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/date_editor.ui")]
    pub struct DateSelector {
        pub selected_date: RefCell<DateTime<FixedOffset>>,
        #[template_child]
        pub date_chooser: TemplateChild<gtk::Calendar>,
        #[template_child]
        pub date_selector_popover: TemplateChild<gtk::Popover>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DateSelector {
        const NAME: &'static str = "HealthDateSelector";
        type ParentType = gtk::Entry;
        type Type = super::DateSelector;

        fn new() -> Self {
            Self {
                selected_date: RefCell::new(chrono::Utc::now().into()),
                date_chooser: TemplateChild::default(),
                date_selector_popover: TemplateChild::default(),
            }
        }

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

            let controller = gtk::EventControllerFocus::new();
            obj.add_controller(&controller);

            controller.connect_enter(clone!(@weak obj => move |_| obj.parse_date()));
            controller.connect_leave(clone!(@weak obj => move |_| obj.parse_date()));
            obj.connect_activate(clone!(@weak obj => move |_| obj.parse_date()));
            obj.connect_icon_press(clone!(@weak obj => move |_, pos| {
                obj.handle_icon_press(pos)
            }));
            self.date_chooser
                .connect_day_selected(clone!(@weak obj => move |c| {
                    obj.handle_date_chooser_connect_day_selected(c)
                }));
            self.date_selector_popover.set_parent(obj);
            obj.set_selected_date(Local::now().into());
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.date_selector_popover.unparent();
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
                "selected-date" => DateTimeBoxed(*self.selected_date.borrow()).to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "selected-date" => {
                    self.selected_date
                        .replace(value.get::<DateTimeBoxed>().unwrap().0);
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for DateSelector {}
    impl EntryImpl for DateSelector {}
}

glib::wrapper! {
    /// A widget to select a date via a [gtk::Calendar] or by entering a date into a [gtk::Entry].
    pub struct DateSelector(ObjectSubclass<imp::DateSelector>)
        @extends gtk::Widget, gtk::Entry, @implements gtk::Editable;
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

    /// Create a new [DateSelector]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create DateSelector")
    }

    /// Set the currently selected date.
    pub fn set_selected_date(&self, value: DateTime<FixedOffset>) {
        self.set_text(&value.format_local());
        let now: DateTime<FixedOffset> = Local::now().into();
        if value.date() > now.date() {
            self.set_property("selected-date", DateTimeBoxed(now))
                .unwrap();
        } else {
            self.set_property("selected-date", DateTimeBoxed(value))
                .unwrap();
        }
    }

    fn imp(&self) -> &imp::DateSelector {
        imp::DateSelector::from_instance(self)
    }

    fn handle_date_chooser_connect_day_selected(&self, calendar: &gtk::Calendar) {
        let date = calendar.date().to_chrono();
        self.set_selected_date(date);
    }

    fn handle_icon_press(&self, pos: gtk::EntryIconPosition) {
        self.parse_date();
        let self_ = self.imp();
        self_
            .date_selector_popover
            .set_pointing_to(&self.icon_area(pos));
        self_.date_selector_popover.show();
    }

    fn parse_date(&self) {
        match dtparse::parse(self.text().as_str()) {
            Ok((d, timezone)) => {
                match timezone
                    .unwrap_or_else(|| *Local.timestamp(0, 0).offset())
                    .from_local_datetime(&d)
                {
                    LocalResult::Single(d) | LocalResult::Ambiguous(d, _) => {
                        self.set_selected_date(d);
                    }
                    LocalResult::None => {}
                }
            }
            Err(e) => {
                glib::g_warning!(
                    crate::config::LOG_DOMAIN,
                    "Couldn't parse date: {}",
                    e.to_string()
                )
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_parse_date() {
        crate::utils::init_gtk();

        let selector = DateSelector::new();
        let mut date = NaiveDate::from_ymd(2009, 2, 16);
        let mut assert_date = |s: &str| {
            selector.set_text(s);
            selector.parse_date();
            date += chrono::Duration::days(1);
            assert_eq!(selector.selected_date().date().naive_local(), date);
        };

        assert_date("02/17/2009");
        assert_date("18/02/2009");
        assert_date("2009/02/19");
        assert_date("February 20, 2009");
        assert_date("2/21/2009");
        assert_date(" 2/22/2009");
        assert_date("23/ 2/2009");
        assert_date("2009/ 2/24");
        assert_date("25Feb2009");
        assert_date("Feb 26, 2009");
        assert_date("27 Feb, 2009");
        assert_date("2009, Feb 28");
    }
}

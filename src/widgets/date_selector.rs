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
use chrono::{DateTime, FixedOffset, Local, LocalResult, NaiveDate, TimeZone};
use glib::subclass::prelude::*;
use gtk::prelude::*;

mod imp {
    use chrono::{DateTime, FixedOffset, Local};
    use glib::clone;
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
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
    /// Get the currently selected date
    pub fn selected_date(&self) -> DateTime<FixedOffset> {
        *self.imp().selected_date.borrow()
    }

    /// Create a new [DateSelector]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create DateSelector")
    }

    /// Set the currently selected date.
    pub fn set_selected_date(&self, value: DateTime<FixedOffset>) {
        self.set_text(&value.format_local());
        self.imp().selected_date.replace(value);
        self.disallow_future_dates();
    }

    fn disallow_future_dates(&self) {
        let date: DateTime<FixedOffset> = Local::now().into();
        if self.imp().selected_date.borrow().date() > date.date() {
            self.set_selected_date(date);
        }
    }

    fn imp(&self) -> &imp::DateSelector {
        imp::DateSelector::from_instance(self)
    }

    fn handle_date_chooser_connect_day_selected(&self, calendar: &gtk::Calendar) {
        let date = Local.timestamp(calendar.date().to_unix(), 0);
        self.set_text(&date.format_local());
        self.imp().selected_date.replace(date.into());
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
        if let Ok(date) = NaiveDate::parse_from_str(self.text().as_str(), "%x") {
            match Local.from_local_datetime(&date.and_hms(12, 0, 0)) {
                LocalResult::Single(d) | LocalResult::Ambiguous(d, _) => {
                    self.set_selected_date(d.into());
                }
                LocalResult::None => {}
            }
        } else {
            glib::g_warning!(crate::config::LOG_DOMAIN, "Couldn't parse date!");
        }
    }
}

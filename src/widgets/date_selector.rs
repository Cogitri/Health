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

use chrono::{DateTime, FixedOffset};
use glib::subclass::prelude::*;
use gtk::prelude::*;

mod imp {
    use chrono::{DateTime, FixedOffset, Local, LocalResult, NaiveDate, TimeZone};
    use glib::{clone, subclass};
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

    impl ObjectSubclass for DateSelector {
        const NAME: &'static str = "HealthDateSelector";
        type ParentType = gtk::Entry;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::DateSelector;
        type Interfaces = ();

        glib::object_subclass!();

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

        fn instance_init(obj: &glib::subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DateSelector {
        fn constructed(&self, obj: &Self::Type) {
            let controller = gtk::EventControllerFocus::new();
            obj.add_controller(&controller);

            let parse_date = clone!(@weak obj => move || {
                if let Ok(date) = NaiveDate::parse_from_str(obj.get_text().as_str(), "%x") {
                        match Local.from_local_datetime(&date.and_hms(12, 0, 0)) {
                            LocalResult::Single(d) | LocalResult::Ambiguous(d, _) => {
                                obj.set_selected_date (d.into());
                            }
                            LocalResult::None => {},
                        }
                } else {
                    glib::g_warning!(crate::config::LOG_DOMAIN, "Couldn't parse date!");
                }
            });
            controller.connect_enter(clone!(@strong parse_date => move |_| parse_date()));
            controller.connect_leave(clone!(@strong parse_date => move |_| parse_date()));
            obj.connect_activate(clone!(@strong parse_date => move |_| parse_date()));
            obj.connect_icon_press(clone!(@weak obj, @strong parse_date => move |_, pos| {
                parse_date();
                let self_ = DateSelector::from_instance(&obj);
                self_.date_selector_popover.set_pointing_to (&obj.get_icon_area(pos));
                self_.date_selector_popover.show();
            }));

            let set_text = clone!(@weak obj => move |c: &gtk::Calendar| {
                let date = Local.timestamp(c.get_date().to_unix(), 0);
                obj.set_text(&format!("{}", date.format("%x")));
                DateSelector::from_instance(&obj).selected_date.replace(date.into());
            });
            self.date_chooser.connect_day_selected(set_text);
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
    pub struct DateSelector(ObjectSubclass<imp::DateSelector>)
        @extends gtk::Widget, gtk::Entry, @implements gtk::Editable;
}

impl DateSelector {
    pub fn get_selected_date(&self) -> DateTime<FixedOffset> {
        *self.get_priv().selected_date.borrow()
    }

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create DateSelector")
    }

    pub fn set_selected_date(&self, value: DateTime<FixedOffset>) {
        self.set_text(&format!("{}", value.format("%x")));
        self.get_priv().selected_date.replace(value);
    }

    fn get_priv(&self) -> &imp::DateSelector {
        imp::DateSelector::from_instance(self)
    }
}

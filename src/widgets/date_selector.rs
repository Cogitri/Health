use chrono::{DateTime, FixedOffset};
use gdk::subclass::prelude::ObjectSubclass;

mod imp {
    use chrono::{Local, LocalResult, NaiveDate, TimeZone, DateTime, FixedOffset};
    use glib::{clone, subclass};
    use gtk::{subclass::prelude::*, prelude::*, CompositeTemplate};
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
                if let Ok(date) = NaiveDate::parse_from_str(obj.get_text().unwrap().as_str(), "%x") {
                        match Local.from_local_datetime(&date.and_hms(12, 0, 0)) {
                            LocalResult::Single(d) | LocalResult::Ambiguous(d, _) => {
                                DateSelector::from_instance(&obj).set_selected_date (&obj, d.into());
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
            self.set_selected_date(&obj, Local::now().into());
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.date_selector_popover.unparent();
        }
    }
    impl WidgetImpl for DateSelector {}
    impl EntryImpl for DateSelector {}

    impl DateSelector {
        pub fn get_selected_date(&self) -> DateTime<FixedOffset> {
            *self.selected_date.borrow()
        }

        pub fn set_selected_date(&self, obj: &super::DateSelector, value: DateTime<FixedOffset>) {
            obj.set_text(&format!("{}", value.format("%x")));
            self.selected_date.replace(value);
        }
    }
}

glib::wrapper! {
    pub struct DateSelector(ObjectSubclass<imp::DateSelector>)
        @extends gtk::Widget, gtk::Entry, @implements gtk::Editable;
}

impl DateSelector {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create DateSelector")
    }

    pub fn get_selected_date(&self) -> DateTime<FixedOffset> {
        imp::DateSelector::from_instance(self).get_selected_date()
    }

    pub fn set_selected_date(&self, value: DateTime<FixedOffset>) {
        imp::DateSelector::from_instance(self).set_selected_date(self, value);
    }
}

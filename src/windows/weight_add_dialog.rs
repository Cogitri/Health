use crate::core::HealthDatabase;
use gdk::subclass::prelude::ObjectSubclass;
use gtk::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;
    use crate::{
        core::{i18n, settings::Unitsystem, HealthSettings},
        model::Weight,
        widgets::HealthDateSelector,
    };
    use glib::{clone, subclass};
    use gtk::subclass::prelude::*;
    use once_cell::unsync::OnceCell;
    use uom::si::{
        f32::Mass,
        mass::{kilogram, pound},
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/weight_add_dialog.ui")]
    pub struct HealthWeightAddDialog {
        pub database: OnceCell<HealthDatabase>,
        pub settings: HealthSettings,

        #[template_child]
        pub date_selector: TemplateChild<HealthDateSelector>,
        #[template_child]
        pub weight_spin_button: TemplateChild<gtk::SpinButton>,
    }

    impl ObjectSubclass for HealthWeightAddDialog {
        const NAME: &'static str = "HealthWeightAddDialog";
        type ParentType = gtk::Dialog;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::HealthWeightAddDialog;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                database: OnceCell::new(),
                settings: HealthSettings::new(),
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

    impl ObjectImpl for HealthWeightAddDialog {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.set_response_sensitive(gtk::ResponseType::Ok, false);
            self.update_title(obj);
            self.connect_handlers(obj);
        }
    }

    impl WidgetImpl for HealthWeightAddDialog {}
    impl WindowImpl for HealthWeightAddDialog {}
    impl DialogImpl for HealthWeightAddDialog {}

    impl HealthWeightAddDialog {
        pub fn set_database(&self, database: HealthDatabase) {
            self.database.set(database).unwrap();
        }

        fn connect_handlers(&self, obj: &super::HealthWeightAddDialog) {
            self.weight_spin_button
                .connect_changed(clone!(@weak obj => move |e| {
                    let text = e.get_text ().unwrap().to_string();
                    obj.set_response_sensitive(gtk::ResponseType::Ok, text != "0" && text != "");
                }));

            obj.connect_response(|obj, id| match id {
                gtk::ResponseType::Ok => {
                    let downgraded = obj.downgrade();
                    glib::MainContext::default().spawn_local(async move {
                        if let Some(obj) = downgraded.upgrade() {
                            let self_ = imp::HealthWeightAddDialog::from_instance(&obj);
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

        fn update_title(&self, obj: &super::HealthWeightAddDialog) {
            let downgraded = obj.downgrade();
            glib::MainContext::default().spawn_local(async move {
                if let Some(obj) = downgraded.upgrade() {
                    let self_ = imp::HealthWeightAddDialog::from_instance(&obj);
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
}

glib::wrapper! {
    pub struct HealthWeightAddDialog(ObjectSubclass<imp::HealthWeightAddDialog>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog;
}

impl HealthWeightAddDialog {
    pub fn new(database: HealthDatabase, parent: &gtk::Window) -> Self {
        let o: HealthWeightAddDialog = glib::Object::new(&[("use-header-bar", &1)])
            .expect("Failed to create HealthWeightAddDialog");

        o.set_transient_for(Some(parent));
        imp::HealthWeightAddDialog::from_instance(&o).set_database(database);

        o
    }
}

/* import_export_dialog_base.rs
 *
 * Copyright 2021 Rasmus Thomsen <oss@cogitri.dev>
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

use crate::{core::i18n, prelude::*};
use gtk::{
    glib::{self, prelude::*, subclass::prelude::*, translate::FromGlib},
    prelude::*,
    subclass::prelude::*,
};

mod imp {
    use crate::{core::i18n, prelude::*, widgets::PasswordEntry};
    use adw::prelude::*;
    use gtk::{
        gio,
        glib::{self},
        {subclass::prelude::*, CompositeTemplate},
    };

    #[repr(C)]
    pub struct ImportExportDialogBaseClass {
        pub parent_class: gtk::ffi::GtkDialogClass,
        pub on_activities:
            fn(&super::ImportExportDialogBase, password: Option<String>) -> PinnedResultFuture<()>,
        pub on_weights:
            fn(&super::ImportExportDialogBase, password: Option<String>) -> PinnedResultFuture<()>,
    }

    unsafe impl ClassStruct for ImportExportDialogBaseClass {
        type Type = super::imp::ImportExportDialogBase;
    }

    impl std::ops::Deref for ImportExportDialogBaseClass {
        type Target = glib::Class<glib::Object>;

        fn deref(&self) -> &Self::Target {
            unsafe { &*(self as *const Self).cast::<Self::Target>() }
        }
    }

    impl std::ops::DerefMut for ImportExportDialogBaseClass {
        fn deref_mut(&mut self) -> &mut glib::Class<glib::Object> {
            unsafe { &mut *(self as *mut Self).cast::<glib::Class<glib::Object>>() }
        }
    }

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/import_export_dialog_base.ui")]
    pub struct ImportExportDialogBase {
        #[template_child]
        pub button_cancel: TemplateChild<gtk::Button>,
        #[template_child]
        pub button_ok: TemplateChild<gtk::Button>,
        #[template_child]
        pub encrypt_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub activities_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub weight_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub password_entry: TemplateChild<PasswordEntry>,
        #[template_child]
        pub activities_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub encrypt_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub weights_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub end_icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub end_title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub end_content_label: TemplateChild<gtk::Label>,
    }

    // Virtual method default implementation trampolines
    fn on_activities_default_trampoline(
        this: &super::ImportExportDialogBase,
        password: Option<String>,
    ) -> PinnedResultFuture<()> {
        ImportExportDialogBase::from_obj(this).on_activities(this, password)
    }

    fn on_weights_default_trampoline(
        this: &super::ImportExportDialogBase,
        password: Option<String>,
    ) -> PinnedResultFuture<()> {
        ImportExportDialogBase::from_obj(this).on_weights(this, password)
    }

    pub(super) fn import_export_dialog_base_on_activities(
        this: &super::ImportExportDialogBase,
        password: Option<String>,
    ) -> PinnedResultFuture<()> {
        let klass = this.class();

        (klass.as_ref().on_activities)(this, password)
    }

    pub(super) fn import_export_dialog_base_on_weights(
        this: &super::ImportExportDialogBase,
        password: Option<String>,
    ) -> PinnedResultFuture<()> {
        let klass = this.class();

        (klass.as_ref().on_weights)(this, password)
    }

    impl ImportExportDialogBase {
        fn on_activities(
            &self,
            obj: &super::ImportExportDialogBase,
            _password: Option<String>,
        ) -> PinnedResultFuture<()> {
            Box::pin(gio::GioFuture::new(obj, move |_, _, send| {
                send.resolve(Ok(()));
            }))
        }

        fn on_weights(
            &self,
            obj: &super::ImportExportDialogBase,
            _password: Option<String>,
        ) -> PinnedResultFuture<()> {
            Box::pin(gio::GioFuture::new(obj, move |_, _, send| {
                send.resolve(Ok(()));
            }))
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImportExportDialogBase {
        const NAME: &'static str = "HealthImportExportDialogBase";
        type ParentType = gtk::Dialog;
        type Type = super::ImportExportDialogBase;
        type Class = ImportExportDialogBaseClass;

        fn class_init(klass: &mut Self::Class) {
            klass.on_activities = on_activities_default_trampoline;
            klass.on_weights = on_weights_default_trampoline;

            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ImportExportDialogBase {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.add_action_widget(&*self.button_cancel, gtk::ResponseType::Cancel);
            obj.add_action_widget(&*self.button_ok, gtk::ResponseType::Ok);
            obj.set_response_sensitive(gtk::ResponseType::Ok, false);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecBoolean::builder("is-import")
                    .construct_only()
                    .readwrite()
                    .build()]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "is-import" => (!self.password_entry.show_password_repeat()).to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "is-import" => {
                    let val: bool = value.get().unwrap();
                    self.password_entry.set_show_password_repeat(!val);
                    self.password_entry.set_show_password_strength(!val);

                    if val {
                        self.activities_action_row
                            .set_title(&i18n("Import activities"));
                        self.encrypt_action_row
                            .set_title(&i18n("Import is encrypted"));
                        self.weights_action_row.set_title(&i18n("Import weights"));
                        self.button_ok.set_label(&i18n("Import"));
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ImportExportDialogBase {}
    impl WindowImpl for ImportExportDialogBase {}
    impl DialogImpl for ImportExportDialogBase {}
}

pub enum DialogState {
    ActivityFileChooserDone,
    WeightFileChooserDone,
}

glib::wrapper! {
    /// A dialog for exporting data
    pub struct ImportExportDialogBase(ObjectSubclass<imp::ImportExportDialogBase>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl ImportExportDialogBase {
    #[template_callback]
    fn check_activate_response(&self) {
        let imp = self.imp();
        let any_option_activated =
            imp.activities_switch.is_active() || imp.weight_switch.is_active();
        let password_set = if imp.encrypt_switch.is_active() {
            imp.password_entry.password().is_some()
        } else {
            true
        };

        self.set_response_sensitive(gtk::ResponseType::Ok, any_option_activated && password_set);
    }

    #[template_callback]
    fn handle_encrypt_switch_active_notify(&self, _pspec: glib::ParamSpec, btn: gtk::Switch) {
        let imp = self.imp();
        self.check_activate_response();
        imp.password_entry.set_sensitive(btn.is_active());
    }

    #[template_callback]
    fn handle_response(&self, id: i32) {
        let id = unsafe { gtk::ResponseType::from_glib(id) };
        gtk_macros::spawn!(glib::clone!(
            #[weak(rename_to = obj)]
            self,
            async move {
                obj.on_response(id).await;
            }
        ));
    }

    async fn on_response(&self, id: gtk::ResponseType) {
        let imp = self.imp();
        if id == gtk::ResponseType::Ok {
            if imp.stack.visible_child_name().unwrap() == "begin" {
                let password = imp.password_entry.password();
                let mut error_text = String::new();
                if imp.activities_switch.is_active() {
                    if let Err(e) = self.on_activities(password.clone()).await {
                        error_text.push_str(&e.to_string());
                    }
                }

                if imp.weight_switch.is_active() {
                    if let Err(e) = self.on_weights(password).await {
                        if !error_text.is_empty() {
                            error_text.push('\n');
                        }
                        error_text.push_str(&e.to_string());
                    }
                }

                if error_text.is_empty() {
                    imp.end_title_label.set_text(&i18n("Success!"));
                    imp.end_icon.set_icon_name(Some("emblem-ok-symbolic"));
                } else {
                    glib::g_warning!(crate::config::LOG_DOMAIN, "{error_text}");
                    imp.end_title_label.set_text(&i18n("An error occurred!"));
                    imp.end_content_label.set_text(&error_text);
                    imp.end_icon
                        .set_icon_name(Some("emblem-important-symbolic"));
                }
                imp.button_ok.set_label(&i18n("Close"));
                self.set_response_sensitive(gtk::ResponseType::Cancel, false);
                imp.stack.set_visible_child_name("end");
            } else {
                self.destroy();
            }
        } else {
            self.destroy();
        }
    }
}

pub trait ImportExportDialogBaseExt {
    fn on_activities(&self, password: Option<String>) -> PinnedResultFuture<()>;
    fn on_weights(&self, password: Option<String>) -> PinnedResultFuture<()>;
}

impl<O: IsA<ImportExportDialogBase>> ImportExportDialogBaseExt for O {
    fn on_activities(&self, password: Option<String>) -> PinnedResultFuture<()> {
        imp::import_export_dialog_base_on_activities(self.upcast_ref(), password)
    }

    fn on_weights(&self, password: Option<String>) -> PinnedResultFuture<()> {
        imp::import_export_dialog_base_on_weights(self.upcast_ref(), password)
    }
}

pub trait ImportExportDialogBaseImpl: DialogImpl + 'static {
    fn on_activities(
        &self,
        obj: &ImportExportDialogBase,
        password: Option<String>,
    ) -> PinnedResultFuture<()> {
        self.parent_on_activities(obj, password)
    }

    fn on_weights(
        &self,
        obj: &ImportExportDialogBase,
        password: Option<String>,
    ) -> PinnedResultFuture<()> {
        self.parent_on_weights(obj, password)
    }
}

pub trait ImportExportDialogBaseImplExt: ObjectSubclass {
    fn parent_on_activities(
        &self,
        obj: &ImportExportDialogBase,
        password: Option<String>,
    ) -> PinnedResultFuture<()>;
    fn parent_on_weights(
        &self,
        obj: &ImportExportDialogBase,
        password: Option<String>,
    ) -> PinnedResultFuture<()>;
}

impl<T: ImportExportDialogBaseImpl> ImportExportDialogBaseImplExt for T {
    fn parent_on_activities(
        &self,
        obj: &ImportExportDialogBase,
        password: Option<String>,
    ) -> PinnedResultFuture<()> {
        unsafe {
            let data = Self::type_data();
            let parent_class =
                &*(data.as_ref().parent_class() as *mut imp::ImportExportDialogBaseClass);
            (parent_class.on_activities)(obj, password)
        }
    }

    fn parent_on_weights(
        &self,
        obj: &ImportExportDialogBase,
        password: Option<String>,
    ) -> PinnedResultFuture<()> {
        unsafe {
            let data = Self::type_data();
            let parent_class =
                &*(data.as_ref().parent_class() as *mut imp::ImportExportDialogBaseClass);
            (parent_class.on_weights)(obj, password)
        }
    }
}

unsafe impl<T: ImportExportDialogBaseImpl> IsSubclassable<T> for ImportExportDialogBase {
    fn class_init(class: &mut glib::Class<Self>) {
        <gtk::Dialog as IsSubclassable<T>>::class_init(class.upcast_ref_mut());

        let klass = class.as_mut();
        klass.on_activities = on_activities_trampoline::<T>;
        klass.on_weights = on_weights_trampoline::<T>;
    }

    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <gtk::Dialog as IsSubclassable<T>>::instance_init(instance);
    }
}

// Virtual method default implementation trampolines
fn on_activities_trampoline<T>(
    this: &ImportExportDialogBase,
    password: Option<String>,
) -> PinnedResultFuture<()>
where
    T: ImportExportDialogBaseImpl + ObjectSubclass,
{
    let imp = T::from_obj(this.dynamic_cast_ref::<T::Type>().unwrap());
    imp.on_activities(this, password)
}

fn on_weights_trampoline<T>(
    this: &ImportExportDialogBase,
    password: Option<String>,
) -> PinnedResultFuture<()>
where
    T: ImportExportDialogBaseImpl + ObjectSubclass,
{
    let imp = T::from_obj(this.dynamic_cast_ref::<T::Type>().unwrap());
    imp.on_weights(this, password)
}

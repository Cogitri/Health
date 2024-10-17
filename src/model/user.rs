/* user.rs
 *
 * Copyright 2021-2022 Aman Kumar <amankrx@protonmail.com>
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

use crate::{core::date::GDateTimeExt, model::ActivityType, plugins::PluginName};
use gtk::glib::{self, prelude::*, subclass::prelude::*};
use uom::si::{
    f32::{Length, Mass},
    length::meter,
    mass::kilogram,
};

#[derive(Clone, glib::Boxed, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[boxed_type(name = "PluginNames")]
pub struct PluginNames(pub Vec<PluginName>);

#[derive(Clone, glib::Boxed, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[boxed_type(name = "ActivityTypes")]
pub struct ActivityTypes(pub Vec<ActivityType>);

/// A [User] is a particular user using the Health who is currently active.

mod imp {
    use super::{ActivityTypes, PluginNames};
    use crate::{model::ActivityType, plugins::PluginName, prelude::*, sync::serialize};
    use gtk::{glib, prelude::*, subclass::prelude::*};
    use std::cell::RefCell;
    use uom::si::{
        f32::{Length, Mass},
        length::meter,
        mass::kilogram,
    };

    #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
    pub struct UserMut {
        pub user_id: i64,
        pub user_name: Option<String>,
        #[serde(serialize_with = "serialize::serialize_datetime")]
        #[serde(deserialize_with = "serialize::deserialize_datetime")]
        pub user_birthday: Option<glib::DateTime>,
        #[serde(serialize_with = "serialize::serialize_distance")]
        #[serde(deserialize_with = "serialize::deserialize_distance")]
        pub user_height: Option<Length>,
        #[serde(serialize_with = "serialize::serialize_weight")]
        #[serde(deserialize_with = "serialize::deserialize_weight")]
        pub user_weightgoal: Option<Mass>,
        pub user_stepgoal: Option<i64>,
        pub enabled_plugins: Option<Vec<PluginName>>,
        pub recent_activity_types: Option<Vec<ActivityType>>,
        pub did_initial_setup: Option<bool>,
    }

    pub struct User {
        pub inner: RefCell<UserMut>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for User {
        const NAME: &'static str = "HealthUser";
        type ParentType = glib::Object;
        type Type = super::User;

        fn new() -> Self {
            Self {
                inner: RefCell::new(UserMut {
                    user_id: 0,
                    user_name: None,
                    user_birthday: Some(glib::DateTime::local()),
                    user_height: None,
                    user_weightgoal: None,
                    user_stepgoal: None,
                    enabled_plugins: Some(vec![]),
                    recent_activity_types: Some(vec![]),
                    did_initial_setup: Some(false),
                }),
            }
        }
    }

    impl ObjectImpl for User {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;

            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt64::builder("user-id")
                        .minimum(0)
                        .default_value(0)
                        .maximum(u32::MAX.into())
                        .readwrite()
                        .construct()
                        .build(),
                    glib::ParamSpecString::builder("user-name")
                        .default_value(Some("User"))
                        .readwrite()
                        .construct()
                        .build(),
                    glib::ParamSpecBoxed::builder::<glib::DateTime>("user-birthday")
                        .readwrite()
                        .construct()
                        .build(),
                    glib::ParamSpecFloat::builder("user-height")
                        .minimum(-1.0)
                        .maximum(f32::MAX)
                        .default_value(-1.0)
                        .readwrite()
                        .construct()
                        .build(),
                    glib::ParamSpecFloat::builder("user-weightgoal")
                        .minimum(-1.0)
                        .maximum(f32::MAX)
                        .default_value(-1.0)
                        .readwrite()
                        .construct()
                        .build(),
                    glib::ParamSpecInt64::builder("user-stepgoal")
                        .readwrite()
                        .construct()
                        .build(),
                    glib::ParamSpecBoxed::builder::<PluginNames>("enabled-plugins")
                        .readwrite()
                        .construct()
                        .build(),
                    glib::ParamSpecBoxed::builder::<ActivityTypes>("recent-activity-types")
                        .construct()
                        .readwrite()
                        .build(),
                    glib::ParamSpecBoolean::builder("did-initial-setup")
                        .construct()
                        .readwrite()
                        .build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "user-id" => {
                    self.inner.borrow_mut().user_id = value.get::<i64>().unwrap();
                }
                "user-name" => {
                    let value = value.get::<&str>().unwrap().to_string();
                    self.inner.borrow_mut().user_name = Some(value);
                }
                "user-birthday" => {
                    let value = value.get().unwrap();
                    self.inner.borrow_mut().user_birthday = Some(value);
                }
                "user-height" => {
                    let value = value.get::<f32>().unwrap();
                    self.inner.borrow_mut().user_height = Some(Length::new::<meter>(value));
                }
                "user-weightgoal" => {
                    let value = value.get::<f32>().unwrap();
                    self.inner.borrow_mut().user_weightgoal = Some(Mass::new::<kilogram>(value));
                }
                "user-stepgoal" => {
                    let value = value.get::<i64>().unwrap();
                    self.inner.borrow_mut().user_stepgoal = Some(value);
                }
                "enabled-plugins" => {
                    self.inner.borrow_mut().enabled_plugins =
                        Some(value.get::<PluginNames>().unwrap().0);
                }
                "recent-activity-types" => {
                    self.inner.borrow_mut().recent_activity_types =
                        Some(value.get::<ActivityTypes>().unwrap().0);
                }
                "did-initial-setup" => {
                    let value = value.get::<bool>().unwrap();
                    self.inner.borrow_mut().did_initial_setup = Some(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "user-id" => self.inner.borrow().user_id.to_value(),
                "user-name" => self
                    .inner
                    .borrow()
                    .user_name
                    .as_ref()
                    .unwrap_or(&"User".to_string())
                    .to_value(),
                "user-birthday" => self
                    .inner
                    .borrow()
                    .user_birthday
                    .as_ref()
                    .unwrap()
                    .to_value(),
                "user-height" => self
                    .inner
                    .borrow()
                    .user_height
                    .map_or(-1.0, |d| d.get::<meter>())
                    .to_value(),
                "user-weightgoal" => self
                    .inner
                    .borrow()
                    .user_weightgoal
                    .map_or(-1.0, |d| d.get::<kilogram>())
                    .to_value(),
                "user-stepgoal" => self.inner.borrow().user_stepgoal.unwrap_or(0).to_value(),
                "enabled-plugins" => PluginNames(
                    self.inner
                        .borrow()
                        .enabled_plugins
                        .as_ref()
                        .unwrap()
                        .to_vec(),
                )
                .to_value(),
                "recent-activity-types" => ActivityTypes(
                    self.inner
                        .borrow()
                        .recent_activity_types
                        .as_ref()
                        .unwrap()
                        .to_vec(),
                )
                .to_value(),
                "did-initial-setup" => self.inner.borrow().did_initial_setup.unwrap().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    /// A [User] represents a single user profile.
    pub struct User(ObjectSubclass<imp::User>);
}

impl Default for User {
    fn default() -> Self {
        Self::new()
    }
}

impl User {
    /// Creates a new [User].
    pub fn new() -> Self {
        glib::Object::builder()
            .property("user-birthday", glib::DateTime::local())
            .build()
    }

    pub fn builder() -> UserBuilder {
        UserBuilder::new()
    }

    pub fn user_id(&self) -> i64 {
        self.property::<i64>("user-id")
    }

    pub fn user_name(&self) -> Option<String> {
        self.property("user-name")
    }

    pub fn user_birthday(&self) -> Option<glib::DateTime> {
        let value = self.property("user-birthday");
        Some(value)
    }

    pub fn user_height(&self) -> Option<Length> {
        let value = self.property::<f32>("user-height");
        Some(Length::new::<meter>(value))
    }

    pub fn user_weightgoal(&self) -> Option<Mass> {
        let value = self.property::<f32>("user-weightgoal");
        Some(Mass::new::<kilogram>(value))
    }

    pub fn user_stepgoal(&self) -> Option<i64> {
        let value = self.property::<i64>("user-stepgoal");
        Some(value)
    }

    pub fn enabled_plugins(&self) -> Option<Vec<PluginName>> {
        let value = self.property::<PluginNames>("enabled-plugins").0;
        Some(value)
    }

    pub fn recent_activity_types(&self) -> Option<Vec<ActivityType>> {
        let value = self.property::<ActivityTypes>("recent-activity-types").0;
        Some(value)
    }

    pub fn did_initial_setup(&self) -> Option<bool> {
        let value = self.property::<bool>("did-initial-setup");
        Some(value)
    }

    pub fn set_user_id(&self, value: i64) -> &Self {
        self.set_property("user-id", value);
        self
    }

    pub fn set_user_name(&self, value: Option<&str>) -> &Self {
        self.set_property("user-name", value.unwrap_or("User"));
        self
    }

    pub fn set_user_birthday(&self, value: Option<glib::DateTime>) -> &Self {
        self.set_property("user-birthday", value.unwrap());
        self
    }

    pub fn set_user_height(&self, value: Option<Length>) -> &Self {
        self.set_property("user-height", value.map_or(-1.0, |v| v.get::<meter>()));
        self
    }

    pub fn set_user_weightgoal(&self, value: Option<Mass>) -> &Self {
        self.set_property(
            "user-weightgoal",
            value.map_or(-1.0, |v| v.get::<kilogram>()),
        );
        self
    }

    pub fn set_user_stepgoal(&self, value: Option<i64>) -> &Self {
        self.set_property("user-stepgoal", value.unwrap_or(0));
        self
    }

    pub fn set_enabled_plugins(&self, value: Option<Vec<PluginName>>) -> &Self {
        self.set_property("enabled-plugins", PluginNames(value.unwrap()));
        self
    }

    pub fn set_recent_activity_types(&self, value: Option<Vec<ActivityType>>) -> &Self {
        self.set_property("recent-activity-types", ActivityTypes(value.unwrap()));
        self
    }

    pub fn set_did_initial_setup(&self, value: Option<bool>) -> &Self {
        self.set_property("did-initial-setup", value.unwrap_or(false));
        self
    }
}

impl serde::Serialize for User {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        self.imp().inner.borrow().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for User {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = imp::UserMut::deserialize(deserializer)?;

        let a = Self::new();
        a.imp().inner.replace(inner);
        Ok(a)
    }
}

/// A [builder-pattern] type to construct [`User`] objects.
///
/// [builder-pattern]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
#[must_use = "The builder must be built to be used"]
pub struct UserBuilder {
    builder: glib::object::ObjectBuilder<'static, User>,
}

impl UserBuilder {
    /// Create a new [`UserBuilder`].
    pub fn new() -> Self {
        Self {
            builder: glib::Object::builder(),
        }
    }

    /// Build the [`UserBuilder`].
    #[must_use = "Building the object from the builder is usually expensive and is not expected to have side effects"]
    pub fn build(self) -> User {
        self.builder.build()
    }

    pub fn user_id(mut self, user_id: i64) -> Self {
        self.builder = self.builder.property("user-id", user_id);
        self
    }

    pub fn user_name(mut self, user_name: &str) -> Self {
        self.builder = self.builder.property("user-name", user_name);
        self
    }

    pub fn user_birthday(mut self, user_birthday: glib::DateTime) -> Self {
        self.builder = self.builder.property("user-birthday", &user_birthday);
        self
    }

    pub fn user_height(mut self, user_height: Length) -> Self {
        self.builder = self
            .builder
            .property("user-height", user_height.get::<meter>());
        self
    }

    pub fn user_weightgoal(mut self, user_weightgoal: Mass) -> Self {
        self.builder = self
            .builder
            .property("user-weightgoal", user_weightgoal.get::<kilogram>());
        self
    }

    pub fn user_stepgoal(mut self, user_stepgoal: i64) -> Self {
        self.builder = self.builder.property("user-stepgoal", user_stepgoal);
        self
    }

    pub fn enabled_plugins(mut self, enabled_plugins: Vec<PluginName>) -> Self {
        self.builder = self
            .builder
            .property("enabled-plugins", PluginNames(enabled_plugins));
        self
    }

    pub fn recent_activity_types(mut self, recent_activity_types: Vec<ActivityType>) -> Self {
        self.builder = self.builder.property(
            "recent-activity-types",
            ActivityTypes(recent_activity_types),
        );
        self
    }

    pub fn did_initial_setup(mut self, did_initial_setup: bool) -> Self {
        self.builder = self
            .builder
            .property("did-initial-setup", did_initial_setup);
        self
    }
}

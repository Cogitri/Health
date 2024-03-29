/* tab_button.rs
 *
 * Copyright 2021 Visvesh Subramanian <visveshs.blogspot.com>
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

use adw::subclass::prelude::*;
use gtk::glib;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::{glib, CompositeTemplate};

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/card.ui")]
    pub struct Card {}

    #[glib::object_subclass]
    impl ObjectSubclass for Card {
        const NAME: &'static str = "HealthCard";
        type ParentType = adw::Bin;
        type Type = super::Card;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl BinImpl for Card {}
    impl WidgetImpl for Card {}
    impl ObjectImpl for Card {}
}

glib::wrapper! {
    /// [Card] is a toplevel container that is implemented by all other views of Health.
    pub struct Card(ObjectSubclass<imp::Card>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Card {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

unsafe impl<T: BinImpl> IsSubclassable<T> for Card {
    fn class_init(class: &mut glib::Class<Self>) {
        <adw::Bin as IsSubclassable<T>>::class_init(class.upcast_ref_mut());
    }

    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <adw::Bin as IsSubclassable<T>>::instance_init(instance);
    }
}

#[cfg(test)]
mod test {
    use super::Card;
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();
        Card::new();
    }
}

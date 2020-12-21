/* date_editor.vala
 *
 * Copyright 2020 Rasmus Thomsen <oss@cogitri.dev>
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

namespace Health {
    [GtkTemplate (ui = "/dev/Cogitri/Health/date_editor.ui")]
    class DateSelector : Gtk.Entry {
        [GtkChild]
        private Gtk.Calendar date_chooser;
        [GtkChild]
        private Gtk.Popover date_selector_popover;
        private GLib.DateTime _selected_date;
        public GLib.DateTime selected_date {
            get {
                return this._selected_date;
            }
            set {
                this.date_chooser.select_day (value);
                this._selected_date = value;
                this.text = this._selected_date.format ("%x");
            }
        }

        construct {
            var controller = new Gtk.EventControllerFocus ();
            this.add_controller (controller);

            controller.enter.connect (() => {
                this.parse_date ();
            });
            controller.leave.connect (() => {
                this.parse_date ();
            });

            this.date_selector_popover.set_parent (this);
            this.selected_date = new GLib.DateTime.now ();
        }

        public override void dispose () {
            this.date_selector_popover.unparent ();
            base.dispose ();
        }

        public override void size_allocate (int width, int height, int baseline) {
            base.size_allocate (width, height, baseline);

            this.date_selector_popover.present ();
        }

        private void parse_date () {
            GLib.Date date = GLib.Date ();
            date.set_parse (this.text);

            if (!date.valid ()) {
                this.selected_date = this.date_chooser.get_date ();
            } else {
                this.date_chooser.select_day (new GLib.DateTime.local (date.get_year (), date.get_month (), date.get_day (), 0, 0, 0));
            }

        }

        [GtkCallback]
        private void on_activated (Gtk.Entry entry) {
            this.parse_date ();
        }

        [GtkCallback]
        private void on_icon_pressed (Gtk.Entry entry, Gtk.EntryIconPosition pos) {
            this.parse_date ();
            this.date_selector_popover.pointing_to = this.get_icon_area (pos);
            this.date_selector_popover.show ();
        }

        [GtkCallback]
        private void on_calendar_date_changed (Gtk.Calendar c) {
            this._selected_date = c.get_date ();
            this.text = this._selected_date.format ("%x");
        }
    }
}

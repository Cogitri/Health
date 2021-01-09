/* distance_row.vala
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

    [GtkTemplate (ui = "/dev/Cogitri/Health/ui/distance_action_row.ui")]
    public class DistanceActionRow : Hdy.ActionRow {
        /**
         * Forwards the {@link Gtk.SpinButton}'s {@link Gtk.SpinButton.input} Signal
         */
        public signal int input (out double new_value);

        /**
         * The current value of the spinner in meters
         */
        public double value {
            get {
                return this._value;
            }
            set {
                if (this.settings.unitsystem == Unitsystem.IMPERIAL) {
                    if (this.small_unit_togglebutton.active) {
                        this.distance_spin_button.value = Util.meters_to_yard (value);
                     } else {
                        var miles = Util.meters_to_miles (value);
                        if (miles > 1) {
                            this.distance_spin_button.value = miles;
                        } else {
                            this.distance_spin_button.value = Util.meters_to_yard (value);
                        }
                    }
                } else {
                    if (this.small_unit_togglebutton.active) {
                        this.distance_spin_button.value = value;
                    } else {
                        var km = Util.meters_to_km (value);
                        if (km > 1) {
                            this.distance_spin_button.value = km;
                        } else {
                            this.distance_spin_button.value = value;
                        }
                    }
                }
            }
        }

        [GtkChild]
        private Gtk.Adjustment distance_adjustment;
        [GtkChild]
        private Gtk.SpinButton distance_spin_button;
        [GtkChild]
        private Gtk.ToggleButton big_unit_togglebutton;
        [GtkChild]
        private Gtk.ToggleButton small_unit_togglebutton;

        private double _value;
        private Settings settings;

        construct {
            this.settings = Settings.get_instance ();
            this.set_togglebutton_text ();

            this.settings.changed[Settings.UNITSYSTEM_KEY].connect (() => {
                this.set_togglebutton_text ();
            });
        }

        private void set_togglebutton_text () {
            if (this.settings.unitsystem == Unitsystem.IMPERIAL) {
                this.big_unit_togglebutton.label = _ ("Miles");
                this.small_unit_togglebutton.label = _ ("Yards");
            } else {
                this.big_unit_togglebutton.label = _ ("KM");
                this.small_unit_togglebutton.label = _ ("Meters");
            }
        }

        [GtkCallback]
        private void on_small_unit_togglebutton_toggled (Gtk.ToggleButton btn) {
            // Do bigger increments if the smaller unit (meters/yard) is selected, do smaller increments otherwise
            if (btn.active) {
                this.distance_adjustment.step_increment = 100;
                this.distance_adjustment.page_increment = 1000;
            } else {
                this.distance_adjustment.step_increment = 1;
                this.distance_adjustment.page_increment = 5;
            }

            this.value = this.value;
        }

        [GtkCallback]
        private void on_distance_spin_button_changed (Gtk.SpinButton sb) {
            var value = sb.value;

            if (this.settings.unitsystem == Unitsystem.IMPERIAL) {
                if (small_unit_togglebutton.active) {
                    value = Util.yard_to_meters (value);
                } else {
                    value = Util.miles_to_meters (value);
                }
            } else {
                if (!small_unit_togglebutton.active) {
                    value = Util.km_to_meters (value);
                }
            }

            this._value = value;
            this.notify_property ("value");
        }

        [GtkCallback]
        private int on_distance_spin_button_input (Gtk.SpinButton sb, out double new_value) {
            return this.input (out new_value);
        }
    }

}

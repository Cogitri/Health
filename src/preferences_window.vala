/* preferences_window.vala
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
    /**
     * The PreferencesWindow is presented to the user to set certain settings
     * in the applcation.
     */
    [GtkTemplate (ui = "/dev/Cogitri/Health/ui/preferences_window.ui")]
    public class PreferencesWindow : Hdy.PreferencesWindow {
        [GtkChild]
        private Hdy.ActionRow height_actionrow;
        [GtkChild]
        private Gtk.SpinButton age_spinner;
        [GtkChild]
        private Gtk.SpinButton height_spinner;
        [GtkChild]
        private Gtk.SpinButton stepgoal_spinner;
        [GtkChild]
        private Gtk.SpinButton weightgoal_spinner;
        [GtkChild]
        private Hdy.ActionRow weightgoal_actionrow;
        [GtkChild]
        private SyncView sync_view;

        private Settings settings;
        private Gtk.Window? parent_window;

        public signal void import_done ();

        public PreferencesWindow (Settings settings, Gtk.Window? parent) {
            this.settings = settings;

            this.stepgoal_spinner.value = this.settings.user_stepgoal;
            this.weightgoal_spinner.value = this.settings.user_weightgoal.value;
            this.age_spinner.value = this.settings.user_age;
            this.sync_view.parent_window = parent;
            this.sync_view.settings = settings;

            var lower_weight = 18.5 * GLib.Math.pow ((float) settings.user_height / 100, 2);
            var upper_weight = 24.9 * GLib.Math.pow ((float) settings.user_height / 100, 2);
            if (this.settings.unitsystem == Unitsystem.METRIC) {
                this.height_actionrow.title = _ ("Height in centimeters");
                this.height_spinner.value = settings.user_height;
                this.weightgoal_actionrow.subtitle = _ ("Your recommended weight is %.2lf-%.2lf KG (18.5-25 BMI).").printf (lower_weight, upper_weight);
            } else {
                this.height_actionrow.title = _ ("Height in inch");
                this.height_spinner.value = cm_to_inch (settings.user_height);
                this.weightgoal_actionrow.subtitle = _ ("Your recommended weight is %.2lf-%.2lf PB (18.5-25 BMI).").printf (kg_to_pb (lower_weight), kg_to_pb (upper_weight));
            }

            this.parent_window = parent;
            this.set_transient_for (parent);
            this.show ();
        }

        [GtkCallback]
        private void age_spinner_changed (Gtk.Editable editable) {
            var value = uint.parse (editable.text);
            if (value != 0) {
                this.settings.user_age = value;
            }
        }

        [GtkCallback]
        private void stepgoal_spinner_changed (Gtk.Editable editable) {
            var value = uint.parse (editable.text);
            if (value != 0) {
                this.settings.user_stepgoal = value;
            }
        }

        [GtkCallback]
        private void weightgoal_spinner_changed (Gtk.Editable editable) {
            var value = double.parse (editable.text);
            if (value != 0) {
                this.settings.user_weightgoal = new WeightUnitContainer.from_user_value (value, settings);
            }
        }

        [GtkCallback]
        private void height_spinner_changed (Gtk.Editable editable) {
            var value = uint.parse (editable.text);
            if (value != 0) {
                if (this.settings.unitsystem == Unitsystem.METRIC) {
                    this.settings.user_height = value;
                } else {
                    this.settings.user_height = (uint) inch_to_cm (value);
                }
            }
        }

        [GtkCallback]
        private void unit_metric_togglebutton_toggled (Gtk.ToggleButton btn) {
            if (btn.active) {
                this.settings.unitsystem = Unitsystem.METRIC;
                this.height_actionrow.title = _ ("Height in centimeters");
            } else {
                this.settings.unitsystem = Unitsystem.IMPERIAL;
                this.height_actionrow.title = _ ("Height in inch");
            }
        }
    }
}

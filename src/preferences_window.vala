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
        private Gtk.SpinButton age_spin_button;
        [GtkChild]
        private Gtk.SpinButton height_spin_button;
        [GtkChild]
        private Gtk.SpinButton stepgoal_spin_button;
        [GtkChild]
        private Gtk.SpinButton weightgoal_spin_button;
        [GtkChild]
        private BMILevelBar bmi_levelbar;
        [GtkChild]
        private SyncView sync_view;

        private Settings settings;
        private Gtk.Window? parent_window;

        public signal void import_done ();

        public PreferencesWindow (Gtk.Window? parent) {
            this.settings = Settings.get_instance ();

            this.stepgoal_spin_button.value = this.settings.user_stepgoal;
            this.weightgoal_spin_button.value = this.settings.user_weightgoal.value;
            this.age_spin_button.value = this.settings.user_age;
            this.sync_view.parent_window = parent;

            if (this.settings.unitsystem == Unitsystem.METRIC) {
                this.height_actionrow.title = _ ("Height in centimeters");
                this.height_spin_button.value = settings.user_height;
            } else {
                this.height_actionrow.title = _ ("Height in inch");
                this.height_spin_button.value = Util.cm_to_inch (settings.user_height);
            }

            this.parent_window = parent;
            this.set_transient_for (parent);
            this.show ();
        }

        [GtkCallback]
        private void age_spin_button_changed (Gtk.Editable editable) {
            var value = uint.parse (editable.text);
            if (value != 0) {
                this.settings.user_age = value;
            }
        }

        [GtkCallback]
        private void stepgoal_spin_button_changed (Gtk.Editable editable) {
            var value = uint.parse (editable.text);
            if (value != 0) {
                this.settings.user_stepgoal = value;
            }
        }

        [GtkCallback]
        private void weightgoal_spin_button_changed (Gtk.Editable editable) {
            var value = double.parse (editable.text);
            if (value != 0) {
                this.settings.user_weightgoal = new WeightUnitContainer.from_user_value (value);
                this.bmi_levelbar.weight = value;
            }
        }

        [GtkCallback]
        private void height_spin_button_changed (Gtk.Editable editable) {
            var value = uint.parse (editable.text);
            if (value != 0) {
                if (this.settings.unitsystem == Unitsystem.METRIC) {
                    this.settings.user_height = value;
                } else {
                    this.settings.user_height = (uint) Util.inch_to_cm (value);
                }
                this.bmi_levelbar.height = value;
            }
        }

        [GtkCallback]
        private void unit_metric_togglebutton_toggled (Gtk.ToggleButton btn) {
            if (btn.active) {
                this.settings.unitsystem = this.bmi_levelbar.unitsystem = Unitsystem.METRIC;
                this.height_actionrow.title = _ ("Height in centimeters");
            } else {
                this.settings.unitsystem = this.bmi_levelbar.unitsystem = Unitsystem.IMPERIAL;
                this.height_actionrow.title = _ ("Height in inch");
            }
        }
    }
}

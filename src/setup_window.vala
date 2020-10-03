/* setup_window.vala
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
     * The {@link SetupWindow} is shown to the user on the first start of the applcation to fill in some data.
     */
    [GtkTemplate (ui = "/org/gnome/Health/setup_window.ui")]
    public class SetupWindow : Gtk.ApplicationWindow {
        [GtkChild]
        private Gtk.RadioButton unit_metric_radiobutton;
        [GtkChild]
        private Gtk.SpinButton age_spinner;
        [GtkChild]
        private Gtk.SpinButton height_spinner;
        [GtkChild]
        private Gtk.SpinButton stepgoal_spinner;
        [GtkChild]
        private Gtk.SpinButton weightgoal_spinner;
        [GtkChild]
        private Gtk.Button setup_finished_button;
        [GtkChild]
        private Gtk.Button setup_cancel_button;
        [GtkChild]
        private Hdy.ActionRow height_actionrow;

        /**
         * This signal is fired when the user presses the setup_finish_button and all input data has been saved to GSettings.
         */
        public signal void setup_done ();

        public SetupWindow (Gtk.Application application, Settings settings) {
            Object (application: application);

            this.stepgoal_spinner.value = 10000;
            this.unit_metric_radiobutton.active = true;
            this.height_actionrow.title = _ ("Height in centimeters");

            this.unit_metric_radiobutton.toggled.connect ((btn) => {
                if (btn.active) {
                    this.height_actionrow.title = _ ("Height in centimeters");
                } else {
                    this.height_actionrow.title = _ ("Height in inch");
                }
                this.set_optimal_weightgoal ();
            });
            this.height_spinner.value_changed.connect (() => {
                this.set_optimal_weightgoal ();
            });
            this.setup_finished_button.clicked.connect (() => {
                var height_in_cm = (uint) this.height_spinner.value;
                if (this.unit_metric_radiobutton.active) {
                    settings.unitsystem = Unitsystem.METRIC;
                } else {
                    settings.unitsystem = Unitsystem.IMPERIAL;
                    height_in_cm = (uint) GLib.Math.round (inch_to_cm (height_in_cm));
                }

                settings.user_age = (uint) this.age_spinner.value;
                settings.user_height = height_in_cm;
                settings.user_stepgoal = (uint) this.stepgoal_spinner.value;
                settings.user_weightgoal = new WeightUnitContainer.from_user_value (this.weightgoal_spinner.value, settings);
                this.setup_done ();
            });
            this.setup_cancel_button.clicked.connect (() => {
                this.destroy ();
            });
        }

        private void set_optimal_weightgoal () {
            const uint OPTIMAL_BMI = 20;
            var height_in_cm = this.height_spinner.value;
            if (!this.unit_metric_radiobutton.active) {
                height_in_cm = inch_to_cm (height_in_cm);
            }
            var optimal_value = OPTIMAL_BMI * GLib.Math.pow (height_in_cm / 100, 2);
            if (!this.unit_metric_radiobutton.active) {
                optimal_value = kg_to_pb (optimal_value);
            }
            this.weightgoal_spinner.value = optimal_value;
        }
    }
}

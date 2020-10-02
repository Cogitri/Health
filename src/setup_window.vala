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
     * The `SetupWindow` is shown to the user on the first start of the applcation to fill in some data.
     */
    [GtkTemplate (ui = "/org/gnome/Health/setup_window.ui")]
    public class SetupWindow : Gtk.ApplicationWindow {
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

        public signal void setup_done ();

        public SetupWindow (Gtk.Application application, Settings settings) {
            Object (application: application);

            this.stepgoal_spinner.value = 10000;

            this.height_spinner.value_changed.connect (() => {
                const uint OPTIMAL_BMI = 20;
                this.weightgoal_spinner.value = OPTIMAL_BMI * GLib.Math.pow (this.height_spinner.value / 100, 2);
            });
            this.setup_finished_button.clicked.connect (() => {
                settings.user_age = (uint) this.age_spinner.value;
                settings.user_height = (uint) this.height_spinner.value;
                settings.user_stepgoal = (uint) this.stepgoal_spinner.value;
                settings.user_weightgoal = new WeightUnitContainer.from_user_value (this.weightgoal_spinner.value, settings);
                this.setup_done ();
            });
            this.setup_cancel_button.clicked.connect (() => {
                this.destroy ();
            });
        }
    }
}

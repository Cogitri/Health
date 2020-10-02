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
    [GtkTemplate (ui = "/org/gnome/Health/preferences_window.ui")]
    public class PreferencesWindow : Hdy.PreferencesWindow {
        [GtkChild]
        private Gtk.SpinButton age_spinner;
        [GtkChild]
        private Gtk.SpinButton height_spinner;
        [GtkChild]
        private Gtk.SpinButton stepgoal_spinner;
        [GtkChild]
        private Gtk.SpinButton weightgoal_spinner;

        public PreferencesWindow (Settings settings, Gtk.Window? parent) {
            settings.bind (Settings.USER_AGE_KEY, this.age_spinner, "value", GLib.SettingsBindFlags.DEFAULT);
            settings.bind (Settings.USER_HEIGHT_KEY, this.height_spinner, "value", GLib.SettingsBindFlags.DEFAULT);
            settings.bind (Settings.USER_STEPGOAL_KEY, this.stepgoal_spinner, "value", GLib.SettingsBindFlags.DEFAULT);
            settings.bind (Settings.USER_WEIGHTGOAL_KEY, this.weightgoal_spinner, "value", GLib.SettingsBindFlags.DEFAULT);

            this.set_transient_for (parent);
            this.destroy_with_parent = true;
            this.show_all ();
        }
    }
}

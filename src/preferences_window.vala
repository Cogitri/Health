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
        private Gtk.ListBox provider_sync_start_listbox;
        [GtkChild]
        private Gtk.ListBoxRow google_fit_start_sync_row;
        [GtkChild]
        private Gtk.SpinButton age_spinner;
        [GtkChild]
        private Gtk.SpinButton height_spinner;
        [GtkChild]
        private Gtk.SpinButton stepgoal_spinner;
        [GtkChild]
        private Gtk.SpinButton weightgoal_spinner;
        [GtkChild]
        private Hdy.ActionRow height_actionrow;

        private Gtk.Window? parent_window;

        public signal void import_done ();

        public PreferencesWindow (Settings settings, Gtk.Window? parent) {
            settings.bind (Settings.USER_AGE_KEY, this.age_spinner, "value", GLib.SettingsBindFlags.DEFAULT);
            settings.bind (Settings.USER_STEPGOAL_KEY, this.stepgoal_spinner, "value", GLib.SettingsBindFlags.DEFAULT);

            this.weightgoal_spinner.value = settings.user_weightgoal.value;

            if (settings.unitsystem == Unitsystem.METRIC) {
                this.height_actionrow.title = _ ("Height in centimeters");
                this.height_spinner.value = settings.user_height;
            } else {
                this.height_actionrow.title = _ ("Height in inch");
                this.height_spinner.value = cm_to_inch (settings.user_height);
            }

            this.height_spinner.value_changed.connect ((btn) => {
                if (settings.unitsystem == Unitsystem.METRIC) {
                    settings.user_height = (uint) btn.value;
                } else {
                    settings.user_height = (uint) inch_to_cm (btn.value);
                }
            });

            this.weightgoal_spinner.value_changed.connect ((btn) => {
                settings.user_weightgoal = new WeightUnitContainer.from_user_value (btn.value, settings);
            });

            this.provider_sync_start_listbox.row_activated.connect ((row) => {
                if (row == this.google_fit_start_sync_row) {
                    var proxy = new GoogleFitOAuth2Proxy ();
                    proxy.open_authentication_url.begin ((obj, res) => {
                        try {
                            proxy.open_authentication_url.end (res);
                            proxy.import_data.begin (settings, (obj, res) => {
                                try {
                                    proxy.import_data.end (res);
                                    this.import_done ();
                                } catch (GLib.Error e) {
                                    this.open_sync_error (e.message);
                                }
                            });
                        } catch (GLib.Error e) {
                            this.open_sync_error (e.message);
                        }
                    });
                }
            });

            this.parent_window = parent;
            this.set_transient_for (parent);
            this.destroy_with_parent = true;
            this.show ();
        }

        private void open_sync_error (string errmsg) {
            var dialog = new Gtk.MessageDialog (this.parent_window, Gtk.DialogFlags.DESTROY_WITH_PARENT | Gtk.DialogFlags.MODAL, Gtk.MessageType.ERROR, Gtk.ButtonsType.CLOSE, errmsg);
            unowned var dialog_u = dialog;
            dialog.response.connect ((obj) => {
                dialog_u.destroy ();
            });
        }
    }
}

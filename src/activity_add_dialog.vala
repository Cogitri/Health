/* activity_add_dialog.vala
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
     * A dialog for adding a new activity record.
     */
    [GtkTemplate (ui = "/dev/Cogitri/Health/activity_add_dialog.ui")]
    public class ActivityAddDialog : Gtk.Dialog {
        [GtkChild]
        DateSelector date_selector;
        [GtkChild]
        private Gtk.ListBox activities_list_box;
        [GtkChild]
        private Gtk.SpinButton calories_burned_spinner;
        [GtkChild]
        private Gtk.SpinButton distance_spinner;
        [GtkChild]
        private Gtk.SpinButton duration_spinner;
        [GtkChild]
        private Gtk.SpinButton heart_rate_average_spinner;
        [GtkChild]
        private Gtk.SpinButton heart_rate_max_spinner;
        [GtkChild]
        private Gtk.SpinButton heart_rate_min_spinner;
        [GtkChild]
        private Gtk.SpinButton steps_spinner;
        [GtkChild]
        private Gtk.StringList activity_type_model;
        [GtkChild]
        private Hdy.ActionRow calories_burned_action_row;
        [GtkChild]
        private Hdy.ActionRow distance_action_row;
        [GtkChild]
        private Hdy.ActionRow duration_action_row;
        [GtkChild]
        private Hdy.ActionRow heart_rate_average_action_row;
        [GtkChild]
        private Hdy.ActionRow heart_rate_max_action_row;
        [GtkChild]
        private Hdy.ActionRow heart_rate_min_action_row;
        [GtkChild]
        private Hdy.ActionRow stepcount_action_row;
        [GtkChild]
        private Hdy.ComboRow activity_type_comborow;

        private Activities.ActivityInfo? previous_activity;
        private Settings settings;
        private TrackerDatabase db;

        public ActivityAddDialog (Gtk.Window? parent, TrackerDatabase db, Settings settings) {
            Object (use_header_bar: 1);
            this.set_transient_for (parent);
            this.db = db;
            this.settings = settings;

            // FIXME: Somehow the activity_type_model doesn't live long enough because it's
            // unrefed too often (off by one)
            this.activity_type_model.ref ();
            foreach (var x in Activities.get_values ()) {
                    this.activity_type_model.append (x.name);
            }
            this.activity_type_comborow.selected = Activities.Enum.WALKING;

            // FIXME: Also allow entering distance in KM/Miles
            if (this.settings.unitsystem == Unitsystem.METRIC) {
                this.distance_action_row.title = _ ("Distance in Metres");
            } else {
                this.distance_action_row.title = _ ("Distance in Yards");
            }
        }

        /**
         * Saves the data that has been entered into the dialog to the database.
         */
        public async void save () throws GLib.Error {
            var db = TrackerDatabase.get_instance ();
            var selected_activity = this.get_selected_activity ();
            var distance = this.get_spinner_value_if_datapoint (this.distance_spinner, selected_activity, ActivityDataPoints.DISTANCE);

            if (distance != 0 && settings.unitsystem == Unitsystem.IMPERIAL) {
                // Yard to Metres
                distance = (uint32) (distance * 0.9144);
            }

            yield db.save_activity (
                new Activity (
                    this.get_selected_activity ().type,
                    date_from_datetime (this.date_selector.selected_date),
                    this.get_spinner_value_if_datapoint (this.calories_burned_spinner, selected_activity, ActivityDataPoints.CALORIES_BURNED),
                    distance,
                    this.get_spinner_value_if_datapoint (this.heart_rate_average_spinner, selected_activity, ActivityDataPoints.HEART_RATE),
                    this.get_spinner_value_if_datapoint (this.heart_rate_max_spinner, selected_activity, ActivityDataPoints.HEART_RATE),
                    this.get_spinner_value_if_datapoint (this.heart_rate_min_spinner, selected_activity, ActivityDataPoints.HEART_RATE),
                    this.get_spinner_value_if_datapoint (this.duration_spinner, selected_activity, ActivityDataPoints.DURATION),
                    this.get_spinner_value_if_datapoint (this.steps_spinner, selected_activity, ActivityDataPoints.STEP_COUNT)
                )
            );
        }

        private uint32 get_spinner_value_if_datapoint (Gtk.SpinButton? b, Activities.ActivityInfo a, ActivityDataPoints d) {
            if (d in a.available_data_points && b.get_text () != "") {
                return (uint32) ((!) b).value;
            } else {
                return 0;
            }
        }

        private Activities.ActivityInfo get_selected_activity () {
            return Activities.get_values ()[this.activity_type_comborow.selected];
        }

        private void update_activity_entries () {
            var selected_activity = this.get_selected_activity ();
            unowned Gtk.Widget? w;

            while ((w = this.activities_list_box.get_last_child ()) != this.activity_type_comborow) {
                this.activities_list_box.remove (w);
            }

            if (ActivityDataPoints.CALORIES_BURNED in selected_activity.available_data_points) {
                this.activities_list_box.append (this.calories_burned_action_row);
            }
            if (ActivityDataPoints.DISTANCE in selected_activity.available_data_points) {
                this.activities_list_box.append (this.distance_action_row);
            }
            if (ActivityDataPoints.DURATION in selected_activity.available_data_points) {
                this.activities_list_box.append (this.duration_action_row);
            }
            if (ActivityDataPoints.HEART_RATE in selected_activity.available_data_points) {
                this.activities_list_box.append (this.heart_rate_min_action_row);
                this.activities_list_box.append (this.heart_rate_average_action_row);
                this.activities_list_box.append (this.heart_rate_max_action_row);
            }
            if (ActivityDataPoints.STEP_COUNT in selected_activity.available_data_points) {
                this.activities_list_box.append (this.stepcount_action_row);
            }

        }

        [GtkCallback]
        private void on_response (int response_id) {
            switch (response_id) {
                case Gtk.ResponseType.OK:
                    this.save.begin ((obj, res) => {
                        try {
                            this.save.end (res);
                        } catch (GLib.Error e) {
                            warning (_ ("Failed to save new data due to error %s"), e.message);
                        }
                    });
                    break;
            }
            this.destroy ();
        }


        [GtkCallback]
        private void on_activity_type_comborow_selected (GLib.Object o, GLib.ParamSpec p) {
            this.update_activity_entries ();
            this.previous_activity = this.get_selected_activity ();
        }
    }
}

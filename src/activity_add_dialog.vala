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
    [GtkTemplate (ui = "/dev/Cogitri/Health/ui/activity_add_dialog.ui")]
    public class ActivityAddDialog : Gtk.Dialog {
        [GtkChild]
        private ActivityTypeSelector activity_type_selector;
        [GtkChild]
        private DateSelector date_selector;
        [GtkChild]
        private Gtk.ListBox activities_list_box;
        [GtkChild]
        private Gtk.MenuButton activity_type_menu_button;
        [GtkChild]
        private Gtk.SpinButton calories_burned_spin_button;
        [GtkChild]
        private Gtk.SpinButton distance_spin_button;
        [GtkChild]
        private Gtk.SpinButton duration_spin_button;
        [GtkChild]
        private Gtk.SpinButton heart_rate_average_spin_button;
        [GtkChild]
        private Gtk.SpinButton heart_rate_max_spin_button;
        [GtkChild]
        private Gtk.SpinButton heart_rate_min_spin_button;
        [GtkChild]
        private Gtk.SpinButton steps_spin_button;
        [GtkChild]
        private Hdy.ActionRow activity_type_actionrow;
        [GtkChild]
        private Hdy.ActionRow calories_burned_action_row;
        [GtkChild]
        private Hdy.ActionRow date_selector_actionrow;
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

        private Activities.ActivityInfo? _selected_activity;
        public Activities.ActivityInfo? selected_activity {
            get {
                return this._selected_activity;
            }
            set {
                this._selected_activity = value;
                if (this._selected_activity == null) {
                    this.activity_type_menu_button.label = _ ("No activity selected");
                } else {
                    this.activity_type_menu_button.label = ((!) this._selected_activity).name;
                }
            }
        }

        private bool calories_burned_spin_button_user_changed;
        private bool distance_spin_button_user_changed;
        private bool duration_spin_button_user_changed;
        private bool steps_spin_button_user_changed;
        private uint set_counter;

        private Activity activity;
        private Gtk.FilterListModel? filter_model;
        private Settings settings;
        private TrackerDatabase db;

        public ActivityAddDialog (Gtk.Window? parent, TrackerDatabase db, Settings settings) {
            Object (use_header_bar: 1);
            this.set_transient_for (parent);
            this.db = db;
            this.settings = settings;
            this.activity = (Activity) Object.new (typeof (Activity));
            this.selected_activity = Activities.get_values ()[Activities.Enum.WALKING];

            var model = new GLib.ListStore (typeof (Gtk.Widget));
            model.splice (0, 0, {
                this.date_selector_actionrow,
                this.activity_type_actionrow,
                this.calories_burned_action_row,
                this.distance_action_row,
                this.duration_action_row,
                this.heart_rate_average_action_row,
                this.heart_rate_min_action_row,
                this.heart_rate_max_action_row,
                this.stepcount_action_row,
            });

            var filter = new Gtk.CustomFilter (filter_activity_entries);
            var filter_model = new Gtk.FilterListModel (model, filter);
            this.activities_list_box.bind_model (filter_model, (o) => {
                return (Gtk.Widget) o;
            });

            this.filter_model = filter_model;
            // FIXME: Also allow entering distance in KM/Miles
            if (this.settings.unitsystem == Unitsystem.IMPERIAL) {
                this.distance_action_row.title = _ ("Distance in Yards");
            }

            this.calories_burned_spin_button.input.connect ((out o) => {
                this.calories_burned_spin_button_user_changed = true;
                o = 0;
                return 0;
            });
            this.distance_spin_button.input.connect ((out o) => {
                this.distance_spin_button_user_changed = true;
                o = 0;
                return 0;
            });
            this.duration_spin_button.input.connect ((out o) => {
                this.duration_spin_button_user_changed = true;
                o = 0;
                return 0;
            });
            this.steps_spin_button.input.connect ((out o) => {
                this.steps_spin_button_user_changed = true;
                o = 0;
                return 0;
            });
        }

        /**
         * Saves the data that has been entered into the dialog to the database.
         */
        public async void save () throws GLib.Error {
            var db = TrackerDatabase.get_instance ();
            var selected_activity = this.activity_type_selector.selected_activity;
            var distance = this.get_spin_button_value_if_datapoint (this.distance_spin_button, selected_activity, ActivityDataPoints.DISTANCE);

            if (distance != 0 && settings.unitsystem == Unitsystem.IMPERIAL) {
                // FIXME: Allow inputting in things other than yards
                distance = (uint32) Util.yard_to_meters (distance);
            }

            yield db.save_activity (
                new Activity (
                    this.activity_type_selector.selected_activity.type,
                    Util.date_from_datetime (this.date_selector.selected_date),
                    this.get_spin_button_value_if_datapoint (this.calories_burned_spin_button, selected_activity, ActivityDataPoints.CALORIES_BURNED),
                    distance,
                    this.get_spin_button_value_if_datapoint (this.heart_rate_average_spin_button, selected_activity, ActivityDataPoints.HEART_RATE),
                    this.get_spin_button_value_if_datapoint (this.heart_rate_max_spin_button, selected_activity, ActivityDataPoints.HEART_RATE),
                    this.get_spin_button_value_if_datapoint (this.heart_rate_min_spin_button, selected_activity, ActivityDataPoints.HEART_RATE),
                    this.get_spin_button_value_if_datapoint (this.duration_spin_button, selected_activity, ActivityDataPoints.DURATION),
                    this.get_spin_button_value_if_datapoint (this.steps_spin_button, selected_activity, ActivityDataPoints.STEP_COUNT)
                )
            );
        }

        private uint32 get_spin_button_value_if_datapoint (Gtk.SpinButton b, Activities.ActivityInfo a, ActivityDataPoints d) {
            if (d in a.available_data_points && b.get_text () != "") {
                return (uint32) ((!) b).value;
            } else {
                return 0;
            }
        }

        private bool filter_activity_entries (Object row) {
            if (this.selected_activity == null && !(row == this.activity_type_actionrow || row == this.date_selector_actionrow)) {
                return false;
            }

            var selected_activity = (!) this.selected_activity;
            if ((row == this.activity_type_actionrow || row == this.date_selector_actionrow)
                || (row == this.calories_burned_action_row && ActivityDataPoints.CALORIES_BURNED in selected_activity.available_data_points)
                || (row == this.distance_action_row && ActivityDataPoints.DISTANCE in selected_activity.available_data_points)
                || (row == this.duration_action_row && ActivityDataPoints.DURATION in selected_activity.available_data_points)
                || (row == this.stepcount_action_row && ActivityDataPoints.STEP_COUNT in selected_activity.available_data_points)
                || ((row == this.heart_rate_average_action_row || row == this.heart_rate_max_action_row || row == this.heart_rate_min_action_row) && ActivityDataPoints.HEART_RATE in selected_activity.available_data_points)
            ) {
                return true;
            }

            return false;
        }

        private void save_recent_activity () {
            if (this.selected_activity == null) {
                return;
            }

            var recent_activities = this.settings.recent_activity_types;
            var already_recent = false;
            foreach (var activity in recent_activities) {
                if (((!) this.selected_activity).name == activity) {
                    already_recent = true;
                    break;
                }
            }

            if (!already_recent) {
                recent_activities += ((!) this.selected_activity).name;
                if (recent_activities.length > 4) {
                    this.settings.recent_activity_types = recent_activities[1:recent_activities.length - 1];
                } else {
                    this.settings.recent_activity_types = recent_activities;
                }
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
                    this.save_recent_activity ();

                    break;
            }
            this.activities_list_box.bind_model (null, null);
            if (this.filter_model != null) {
                ((!) this.filter_model).dispose ();
                this.filter_model = null;
            }
            this.destroy ();
        }


        [GtkCallback]
        private void on_activity_type_selector_selected_activity (GLib.Object o, GLib.ParamSpec p) {
            this.selected_activity = this.activity_type_selector.selected_activity;
            this.activity.activity_type = ((!) this.selected_activity).type;

            if (this.filter_model != null) {
                ((!) this.filter_model).filter.changed (Gtk.FilterChange.DIFFERENT);
            }
        }

        private void set_spin_buttons_from_activity (Gtk.SpinButton emitter) {
            if (this.activity.calories_burned != 0 && this.calories_burned_spin_button != emitter && !this.calories_burned_spin_button_user_changed) {
                this.set_counter++;
                this.calories_burned_spin_button.value = this.activity.calories_burned;
            }
            if (this.activity.distance != 0 && this.distance_spin_button != emitter && !this.distance_spin_button_user_changed) {
                this.set_counter++;
                this.distance_spin_button.value = this.activity.distance;
            }
            if (this.activity.minutes != 0 && this.duration_spin_button != emitter && !this.duration_spin_button_user_changed) {
                this.set_counter++;
                this.duration_spin_button.value = this.activity.minutes;
            }
            if (this.activity.steps != 0 && this.steps_spin_button != emitter && !this.steps_spin_button_user_changed) {
                this.set_counter++;
                this.steps_spin_button.value = this.activity.steps;
            }
        }

        [GtkCallback]
        private void on_calories_burned_spin_button_changed (Gtk.SpinButton e) {
            if (this.set_counter != 0) {
                this.set_counter--;
                return;
            }

            this.activity.calories_burned = (uint32) e.value;
            this.activity.autofill_from_calories ();
            this.set_spin_buttons_from_activity (e);
        }

        [GtkCallback]
        private void on_distance_spin_button_changed (Gtk.SpinButton e) {
            if (this.set_counter != 0) {
                this.set_counter--;
                return;
            }

            this.activity.distance = (uint32) e.value;
            this.activity.autofill_from_distance ();
            this.set_spin_buttons_from_activity (e);
        }

        [GtkCallback]
        private void on_duration_spin_button_changed (Gtk.SpinButton e) {
            if (this.set_counter != 0) {
                this.set_counter--;
                return;
            }

            this.activity.minutes = (uint32) e.value;
            this.activity.autofill_from_minutes ();
            this.set_spin_buttons_from_activity (e);
        }

        [GtkCallback]
        private void on_steps_spin_button_changed (Gtk.SpinButton e) {
            if (this.set_counter != 0) {
                this.set_counter--;
                return;
            }

            this.activity.steps = (uint32) e.value;
            this.activity.autofill_from_steps ();
            this.set_spin_buttons_from_activity (e);
        }
    }
}

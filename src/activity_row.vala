/* activity_row.vala
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
    * An implementation of {@link Gtk.ListBox} that displays infos about an {@link Activity}.
    */
    [GtkTemplate (ui = "/dev/Cogitri/Health/ui/activity_row.ui")]
    public class ActivityRow : Gtk.ListBoxRow {
        [GtkChild]
        private Gtk.Label active_minutes_label;
        [GtkChild]
        private Gtk.Label activity_date_label;
        [GtkChild]
        private Gtk.Label activity_type_label;
        [GtkChild]
        private Gtk.Label calories_burned_label;
        [GtkChild]
        private Gtk.Label distance_label;
        [GtkChild]
        private Gtk.Label average_heart_rate_label;
        [GtkChild]
        private Gtk.Label maximum_heart_rate_label;
        [GtkChild]
        private Gtk.Label minimum_heart_rate_label;
        [GtkChild]
        private Gtk.Label steps_label;
        [GtkChild]
        private Gtk.Revealer details_revealer;
        [GtkChild]
        Hdy.ActionRow calories_burned_row;
        [GtkChild]
        Hdy.ActionRow distance_row;
        [GtkChild]
        Hdy.ActionRow steps_row;
        [GtkChild]
        Hdy.ActionRow average_heart_rate_row;
        [GtkChild]
        Hdy.ActionRow minimum_heart_rate_row;
        [GtkChild]
        Hdy.ActionRow maximum_heart_rate_row;

        private Activity _activity;
        public Activity activity {
            get {
                return _activity;
            }
            construct {
                this._activity = value;

                /* TRANSLATORS: this is how many minutes the user was active in the activity view. */
                this.active_minutes_label.label = _ ("%u Minutes").printf (this._activity.minutes);
                /* TRANSLATORS: this is the date as displayed in the activity view, e.g. 30/9 for September 30 */
                this.activity_date_label.label = _ ("%d/%d/%d").printf (this._activity.date.get_day (), this._activity.date.get_month (), this._activity.date.get_year ());
                this.activity_type_label.label = ActivityType.get_values ()[this._activity.activity_type].name;

                if (this._activity.calories_burned != 0) {
                    this.calories_burned_row.visible = true;
                    /* TRANSLATORS: this refers to how many calories the user has burned during an activity */
                    this.calories_burned_label.label = _ ("%u Calories").printf (this._activity.calories_burned);
                }
                if (this._activity.hearth_rate_avg != 0) {
                    this.average_heart_rate_row.visible = true;
                    this.average_heart_rate_label.label = "%u".printf (this._activity.hearth_rate_avg);
                }
                if (this._activity.hearth_rate_min != 0) {
                    this.minimum_heart_rate_row.visible = true;
                    this.minimum_heart_rate_label.label = "%u".printf (this._activity.hearth_rate_min);
                }
                if (this._activity.hearth_rate_max != 0) {
                    this.maximum_heart_rate_row.visible = true;
                    this.maximum_heart_rate_label.label = "%u".printf (this._activity.hearth_rate_max);
                }
                if (this._activity.distance != 0) {
                    this.distance_row.visible = true;

                    var distance = this._activity.distance;
                    if (Settings.get_instance ().unitsystem == Unitsystem.IMPERIAL) {
                        distance = (uint) Util.meters_to_yard (distance);
                        this.distance_row.title = _ ("Distance in Yards");
                    }

                    this.distance_label.label = "%u".printf (distance);
                }
                if (this._activity.steps != 0) {
                    this.steps_row.visible = true;
                    this.steps_label.label = "%u".printf (this._activity.steps);
                }
            }
        }

        construct {
            var gesture_controller = new Gtk.GestureClick ();
            this.add_controller (gesture_controller);

            gesture_controller.pressed.connect (() => {
                this.details_revealer.reveal_child = !this.details_revealer.reveal_child;
            });
        }
    }
}

/* activity_type_selector.vala
 *
 * Copyright 2021 Rasmus Thomsen <oss@cogitri.dev>
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

    [GtkTemplate (ui = "/dev/Cogitri/Health/ui/activity_type_row.ui")]
    public class ActivityTypeRow : Gtk.ListBoxRow {
        [GtkChild]
        Gtk.Image selected_image;
        [GtkChild]
        private Gtk.Label activity_type_label;

        public bool selected {
            get {
                return this.selected_image.visible;
            }
            set {
                this.selected_image.visible = value;
            }
        }
        public string label {
            get {
                return this.activity_type_label.label;
            }
            set {
                this.activity_type_label.label = value;
            }
        }
    }

    public class ActivityTypeRowData : GLib.Object {
        public bool selected;
        public string activity_type_name;

        public ActivityTypeRowData (bool selected, string activity_type_name) {
            this.selected = selected;
            this.activity_type_name = activity_type_name;
        }
    }

    [GtkTemplate (ui = "/dev/Cogitri/Health/ui/activity_type_selector.ui")]
    public class ActivityTypeSelector : Gtk.Popover {
        [GtkChild]
        private Gtk.Box recents_box;
        [GtkChild]
        private Gtk.ListBox activity_types_list_box;
        [GtkChild]
        private Gtk.ListBox recent_activity_types_list_box;
        [GtkChild]
        private Gtk.StringList recent_activity_types_model;
        [GtkChild]
        private Gtk.StringList activity_types_model;

        public Activities.ActivityInfo selected_activity { get; set; }

        construct {
            var recent_activity_types = new Settings ().recent_activity_types;
            if (recent_activity_types.length != 0) {
                this.recents_box.visible = true;
                // The array is sorted in least-recent to most-recent, so add the most recent first by reversing the for-loop
                for (int i = recent_activity_types.length - 1 ; i >= 0; i--) {
                    this.recent_activity_types_model.append (recent_activity_types[i]);
                }

                var activity = Activities.get_info_by_name (recent_activity_types[recent_activity_types.length - 1]);
                if (activity == null) {
                    warning ("Unknown activity %s, falling back to walking", recent_activity_types[recent_activity_types.length - 1]);
                    this.selected_activity = Activities.get_values ()[Activities.Enum.WALKING];
                } else {
                    this.selected_activity = activity;
                }
            } else {
                this.selected_activity = Activities.get_values ()[Activities.Enum.WALKING];
            }

            foreach (var x in Activities.get_values ()) {
                var recent_activity = false;
                foreach (var activity in recent_activity_types) {
                    if (x.name == activity) {
                        recent_activity = true;
                        break;
                    }
                }

                if (!recent_activity) {
                    this.activity_types_model.append (x.name);
                }
            }

            this.activity_types_list_box.bind_model (this.activity_types_model, this.create_list_box_row);
            this.recent_activity_types_list_box.bind_model (this.recent_activity_types_model, this.create_list_box_row);
        }

        private Gtk.Widget create_list_box_row (GLib.Object o) {
            var name = ((Gtk.StringObject) o).string;
            return (Gtk.Widget) Object.new (typeof (ActivityTypeRow), label: name, selected: name == this.selected_activity.name);
        }

        private void refresh_selected_rows (Gtk.ListBox box) {
            var i = 0;
            unowned Gtk.ListBoxRow? row;

            while ((row = box.get_row_at_index (i++)) != null) {
                var cast = (ActivityTypeRow) row;
                cast.selected = cast.label == this.selected_activity.name;
            }
        }

        [GtkCallback]
        private void on_activity_type_row_activated (Gtk.ListBoxRow r) {
            var row = (ActivityTypeRow) r;
            var activity = Activities.get_info_by_name (row.label);

            if (activity != null) {
                this.selected_activity = activity;
                this.refresh_selected_rows (this.activity_types_list_box);
                this.refresh_selected_rows (this.recent_activity_types_list_box);
                this.popdown ();
            } else {
                warning ("Unknown activity %s", row.label);
            }
        }
    }
}

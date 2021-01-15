/* views.vala
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
     * View is a toplevel container, used for e.g. the {@link StepView} and {@link WeightView}.
     */
    [GtkTemplate (ui = "/dev/Cogitri/Health/ui/view.ui")]
    public abstract class View : Gtk.Widget {
        public string empty_subtitle {
            get {
                return this.subtitle_empty_view_label.label;
            }
            set {
                this.subtitle_empty_view_label.label = value;
            }
        }
        public string icon_name {
            owned get {
                return this.empty_icon.icon_name;
            }
            set {
                this.empty_icon.icon_name = value;
            }
        }
        public string title {
            get {
                return this.title_label.label;
            }
            set {
                this.title_label.label = value;
            }
        }
        public string view_title { get; set; }

        [GtkChild]
        protected Gtk.Image empty_icon;
        [GtkChild]
        protected Gtk.Label goal_label;
        [GtkChild]
        protected Gtk.Label subtitle_empty_view_label;
        [GtkChild]
        protected Gtk.Label title_label;
        [GtkChild]
        protected Gtk.Label title_empty_view_label;
        [GtkChild]
        protected Gtk.ScrolledWindow scrolled_window;
        [GtkChild]
        protected Gtk.Stack stack;

        static construct {
            set_layout_manager_type (typeof (Gtk.BinLayout));
        }

        ~View () {
            unowned Gtk.Widget? child;
            while ((child = get_first_child ()) != null) {
                ((!) child).unparent ();
            }
        }

        /**
         * Update the view when new data is available.
         *
         * This can query a DB and then refresh the view.
         */
        public abstract void update ();
    }
}

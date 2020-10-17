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
    public abstract class View : Gtk.Widget {
        public string title;
        public string icon_name;

        static construct {
            set_layout_manager_type (typeof (Gtk.BinLayout));
        }

        protected View () {
            this.visible = true;
        }

        /**
         * Update the view when new data is available.
         *
         * This can query a DB and then refresh the view.
         */
        public abstract void update ();

    }

    /**
     * A {@link GraphModel} is the dataclass for a {@link GraphView}.
     *
     * It can retrieve data from a DB and provides it to
     * a {@link GraphView} to display it to the user.
     */
    public abstract class GraphModel<T>: GLib.Object {
        protected Gee.ArrayList<T> arr;

        public bool is_empty {
            get {
                return this.arr.is_empty;
            }
        }

        protected void init () {
            this.arr = new Gee.ArrayList<T> ();
            this.reload ();
        }

        /**
         * Converts the {@link GraphModel}'s data to data points so it can be displayed in the {@link GraphView}.
         */
        public abstract Gee.ArrayList<Point> to_points ();

        /**
         * Reloads the {@link GraphModel}'s data, e.g. by loading it from the DB again.
         */
        public abstract bool reload ();

    }
}

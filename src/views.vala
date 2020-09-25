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
    public abstract class View : Gtk.Bin {
        public string title;
        public string icon_name;

        protected View () {
            this.visible = true;
        }

        public abstract void update ();

    }

    public class GraphModel<T>: GLib.Object {
        protected Gee.ArrayList<T> arr;

        public GraphModel () {
            this.arr = new Gee.ArrayList<T> ();
            this.reload ();
        }

        public void add (T w) {
            this.arr.add (w);
        }

        public virtual void to_arrays (out double[] days, out double[] values) {
            var reserve_size = this.arr.is_empty ? 1 : this.arr.size;
            days = new double[reserve_size];
            values = new double[reserve_size];

            if (this.arr.is_empty) {
                days[0] = 0;
                values[0] = 0;
                return;
            }
        }

        public virtual bool reload () {
            return false;
        }

    }
}

/* primary_menu.vala
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
    public class PrimaryMenu : Gtk.Popover {
        public PrimaryMenu () {
            Object ();

            var builder = new Gtk.Builder.from_resource ("/org/gnome/Health/primary_menu.ui");
            var menu = builder.get_object ("primary_menu") as GLib.Menu;
            this.bind_model (menu, null);
        }
    }
}

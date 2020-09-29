/* window.vala
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
    public enum ViewModes {
        STEPS,
        WEIGHT,
    }


    [GtkTemplate (ui = "/org/gnome/Health/window.ui")]
    public class Window : Hdy.ApplicationWindow {
        [GtkChild]
        private Gtk.Stack stack;
        [GtkChild]
        private Gtk.MenuButton primary_menu_button;
        [GtkChild]
        private Gtk.Button add_data_button;

        private int current_height;
        private int current_width;
        private Settings settings;
        private ViewModes current_view;
        private View[] views;

        public Window (Gtk.Application app, Settings settings) {
            Object (application: app);
            this.current_view = ViewModes.STEPS;
            var menu = new PrimaryMenu ();
            this.primary_menu_button.set_popover (menu);
            this.settings = settings;
            var weight_model = new WeightGraphModel (this.settings);
            var steps_model = new StepsGraphModel ();
            views = new View[] { new StepView (steps_model), new WeightView (weight_model, settings), };
            foreach (var view in views) {
                stack.add_titled (view, view.name, view.title);
                stack.child_set (view, "icon-name", view.icon_name, null);
            }
            add_data_button.clicked.connect (() => {
                AddDialog dialog = null;
                switch (this.current_view) {
                case STEPS:
                    dialog = new StepsAddDialog (this);
                    break;
                case WEIGHT:
                    dialog = new WeightAddDialog (this, this.settings);
                    break;
                }
                dialog.run ();
                this.views[this.current_view].update ();
            });

            this.current_height = this.settings.window_height;
            this.current_width = this.settings.window_width;
            this.resize (this.current_width, this.current_height);
            if (this.settings.window_is_maximized) {
                this.maximize ();
            }
        }

        public override void size_allocate (Gtk.Allocation alloc) {
            base.size_allocate (alloc);
            if (!this.is_maximized) {
                this.get_size (out this.current_width, out this.current_height);
            }
        }

        public override void destroy () {
            this.settings.window_is_maximized = this.is_maximized;
            this.settings.window_height = this.current_height;
            this.settings.window_width = this.current_width;
            base.destroy ();
        }

        [GtkCallback]
        private void on_visible_child_changed () {
            if (stack.visible_child_name == views[ViewModes.STEPS].name) {
                this.current_view = ViewModes.STEPS;
            } else if (stack.visible_child_name == views[ViewModes.WEIGHT].name) {
                this.current_view = ViewModes.WEIGHT;
            }
        }

    }
}

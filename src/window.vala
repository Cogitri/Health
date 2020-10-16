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

    /**
     * The toplevel application window that holds all other widgets.
     */
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
        private SqliteDatabase db;
        private ViewModes current_view;
        private View[] views;

        public Window (Gtk.Application app, Settings settings) {
            Object (application: app);
            this.current_view = ViewModes.STEPS;
            var menu = new PrimaryMenu ();
            this.settings = settings;
            this.db = new SqliteDatabase ();

            try {
                this.db.open ();
            } catch (DatabaseError e) {
                error (e.message);
            }

            var weight_model = new WeightGraphModel (this.settings, this.db);
            var steps_model = new StepsGraphModel (this.db);
            this.views = new View[] { new StepView (steps_model, this.settings), new WeightView (weight_model, settings), };

            this.primary_menu_button.set_popover (menu);
            foreach (var view in views) {
                stack.add_titled (view, view.name, view.title);
                stack.child_set (view, "icon-name", view.icon_name, null);
            }
            add_data_button.clicked.connect (() => {
                AddDialog dialog;
                switch (this.current_view) {
                case STEPS:
                    dialog = new StepsAddDialog (this, this.db);
                    break;
                case WEIGHT:
                    dialog = new WeightAddDialog (this, this.settings, this.db);
                    break;
                default:
                    error ("Can't create add dialog for unknown view type %d", this.current_view);
                }
                dialog.run ();
                this.views[this.current_view].update ();
            });

            this.current_height = this.settings.window_height;
            this.current_width = this.settings.window_width;
            if (this.current_width != -1 && this.current_height != -1) {
                this.resize (this.current_width, this.current_height);
            }
            if (this.settings.window_is_maximized) {
                this.maximize ();
            }
            var proxy = new GoogleFitOAuth2Proxy ();
            proxy.sync_data.begin (settings, (obj, res) => {
                try {
                    proxy.sync_data.end (res);
                    foreach (var view in this.views) {
                        view.update ();
                    }
                } catch (GLib.Error e) {
                    var dialog = new Gtk.MessageDialog (this, Gtk.DialogFlags.DESTROY_WITH_PARENT | Gtk.DialogFlags.MODAL, Gtk.MessageType.ERROR, Gtk.ButtonsType.CLOSE, _("Synching data from Google Fit failed due to error %s"), e.message);
                    dialog.run ();
                    dialog.destroy ();
                }
            });
        }

        public void update () {
            foreach (var view in views) {
                view.update ();
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

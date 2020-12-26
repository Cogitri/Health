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
    [GtkTemplate (ui = "/dev/Cogitri/Health/ui/window.ui")]
    public class Window : Hdy.ApplicationWindow {
        [GtkChild]
        private Gtk.Stack stack;

        private int current_height;
        private int current_width;
        private Settings settings;
        private TrackerDatabase db;
        private ViewModes current_view;
        private View[] views;
        private uint sync_source_id;

        public Window (Gtk.Application app, Settings settings) {
            Object (application: app);
            this.current_view = ViewModes.STEPS;
            this.settings = settings;

            if (Config.APPLICATION_ID.has_suffix ("Devel")) {
                this.get_style_context ().add_class ("devel");

                Gtk.IconTheme.get_for_display (this.get_display ()).add_resource_path ("/dev/Cogitri/Health/icons");
            }

            try {
                this.db = TrackerDatabase.get_instance ();
            } catch (DatabaseError e) {
                error (e.message);
            }

            var weight_model = new WeightGraphModel (this.settings, this.db);
            var steps_model = new StepsGraphModel (this.db);
            var activity_model = new ActivityModel (this.settings, this.db);
            this.views = new View[] { new StepView (steps_model, this.settings, this.db), new WeightView (weight_model, settings, this.db), new ActivityView (activity_model, this.settings, this.db)};

            foreach (var view in views) {
                var page = this.stack.add_titled (view, view.name, view.title);
                page.icon_name = view.icon_name;
            }
            this.stack.set_visible_child (this.views[0]);

            this.current_height = this.settings.window_height;
            this.current_width = this.settings.window_width;
            if (this.current_width != -1 && this.current_height != -1) {
                this.set_default_size (this.current_width, this.current_height);
            }
            if (this.settings.window_is_maximized) {
                this.maximize ();
            }
            if (this.settings.sync_provider_setup_google_fit) {
                GLib.Idle.add (() => {
                    sync_data (this, this.settings, this.views, 0);
                    return GLib.Source.REMOVE;
                });
                this.sync_source_id = GLib.Timeout.add_seconds (900, () => {
                    sync_data (this, this.settings, this.views, this.sync_source_id);
                    return GLib.Source.CONTINUE;
                });
            }

            this.update ();
        }

        public void update () {
            foreach (var view in views) {
                view.update ();
            }
        }

        public override void size_allocate (int width, int height, int baseline) {
            base.size_allocate (width, height, baseline);
            if (!this.maximized) {
                this.get_default_size (out this.current_width, out this.current_height);
            }
        }

        private static void sync_data (Gtk.Window? parent, Settings settings, View[] views, uint source_id) {
            var proxy = new GoogleFitOAuth2Proxy ();
            var parent_ref = GLib.WeakRef (parent);
            proxy.sync_data.begin (settings, (obj, res) => {
                try {
                    proxy.sync_data.end (res);
                    foreach (var view in views) {
                        view.update ();
                    }
                } catch (OAuth2Error.NO_LIBSECRET_PASSWORD e) {
                    warning (e.message);
                    if (source_id > 0) {
                        GLib.Source.remove (source_id);
                    }
                } catch (GLib.Error e) {
                    var weak_ref = parent_ref.get ();
                    if (weak_ref != null) {
                        var window = (Gtk.Window) weak_ref;
                        var dialog = new Gtk.MessageDialog (window, Gtk.DialogFlags.DESTROY_WITH_PARENT | Gtk.DialogFlags.MODAL, Gtk.MessageType.ERROR, Gtk.ButtonsType.CLOSE, _("Synching data from Google Fit failed due to error %s"), e.message);
                        unowned var dialog_u = dialog;
                        dialog.response.connect (() => {
                            dialog_u.destroy ();
                        });
                    }
                }
            });
        }

        [GtkCallback]
        private bool on_close_request (Gtk.Window window) {
            this.settings.window_is_maximized = this.maximized;
            this.settings.window_height = this.current_height;
            this.settings.window_width = this.current_width;

            if (this.sync_source_id > 0) {
                GLib.Source.remove (this.sync_source_id);
            }

            return false;
        }

        [GtkCallback]
        private void on_visible_child_changed () {
            if (stack.visible_child_name == views[ViewModes.STEPS].name) {
                this.current_view = ViewModes.STEPS;
            } else if (stack.visible_child_name == views[ViewModes.WEIGHT].name) {
                this.current_view = ViewModes.WEIGHT;
            }
        }

        [GtkCallback]
        private void add_data_button_clicked (Gtk.Button btn) {
            Gtk.Dialog dialog;
            switch (this.current_view) {
            case STEPS:
                dialog = new ActivityAddDialog (this, this.db, this.settings);
                break;
            case WEIGHT:
                dialog = new WeightAddDialog (this, this.settings, this.db);
                break;
            default:
                error ("Can't create add dialog for unknown view type %d", this.current_view);
            }
            dialog.present ();
            unowned var dialog_u = dialog;
            dialog.response.connect (() => {
                dialog_u.destroy ();
            });
        }

    }
}

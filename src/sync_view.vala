/* sync_view.vala
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
     * The {@link SyncView} is a view where users can initialise synching with a third-party provider.
     */
    [GtkTemplate (ui = "/dev/Cogitri/Health/sync_view.ui")]
    public class SyncView : Gtk.Widget {
        [GtkChild]
        private Gtk.Image google_fit_selected_image;
        [GtkChild]
        private Gtk.ListBoxRow google_fit_start_sync_row;
        [GtkChild]
        private Gtk.Stack google_fit_stack;
        [GtkChild]
        private Gtk.Spinner google_fit_spinner;

        public Settings? settings { get; set; }
        public weak Gtk.Window? parent_window { get; set; }

        static construct {
            set_layout_manager_type (typeof (Gtk.BinLayout));
        }

        [GtkCallback]
        private void sync_list_box_row_activated (Gtk.ListBoxRow row) {
            if (row == this.google_fit_start_sync_row) {
                this.google_fit_stack.visible = true;
                this.google_fit_spinner.visible = true;
                this.google_fit_stack.visible_child = this.google_fit_spinner;
                var proxy = new GoogleFitOAuth2Proxy ();
                proxy.open_authentication_url.begin ((obj, res) => {
                    try {
                        proxy.open_authentication_url.end (res);
                        proxy.import_data.begin (settings ?? new Health.Settings (), (obj, res) => {
                            try {
                                proxy.import_data.end (res);
                                this.google_fit_selected_image.visible = true;
                                this.google_fit_stack.visible_child = this.google_fit_selected_image;
                            } catch (GLib.Error e) {
                                this.open_sync_error (e.message);
                                this.google_fit_selected_image.visible = true;
                                this.google_fit_selected_image.icon_name = "network-error-symbolic";
                                this.google_fit_stack.visible_child = this.google_fit_selected_image;
                            }
                        });
                    } catch (GLib.Error e) {
                        this.open_sync_error (e.message);
                        this.google_fit_selected_image.visible = true;
                        this.google_fit_selected_image.icon_name = "network-error-symbolic";
                        this.google_fit_stack.visible_child = this.google_fit_selected_image;
                    }
                });
            }
        }

        private void open_sync_error (string errmsg) {
            var dialog = new Gtk.MessageDialog (this.parent_window, Gtk.DialogFlags.DESTROY_WITH_PARENT | Gtk.DialogFlags.MODAL, Gtk.MessageType.ERROR, Gtk.ButtonsType.CLOSE, errmsg);
            unowned var dialog_u = dialog;
            dialog.response.connect ((obj) => {
                dialog_u.destroy ();
            });
        }
    }
}

/* application.vala
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

    public class Application : Gtk.Application {
        private Settings settings;
        private weak Window? window;
        private const GLib.ActionEntry[] APP_ENTRIES = {
            { "about", on_about },
            { "fullscreen", on_fullscreen },
            { "hamburger-menu", on_hamburger_menu },
            { "help", on_help },
            { "preferences", on_preferences },
            { "quit", on_quit },
            { "shortcuts", on_shortcuts }
        };

        public Application () {
            Object (application_id: Config.APPLICATION_ID, flags : ApplicationFlags.FLAGS_NONE);
            this.settings = Settings.get_instance ();
        }

        public override void activate () {
            if (this.window != null) {
                return;
            } else if (this.settings.did_initial_setup) {
                var window = new Window (this);
                window.show ();
                this.window = window;
            } else {
                var setup_window = new SetupWindow (this);
                setup_window.setup_done.connect (() => {
                    this.settings.did_initial_setup = true;
                    var window = new Window (this);
                    window.show ();
                    this.window = window;
                });
                setup_window.show ();
            }

            this.set_accels_for_action ("app.fullscreen", { "F11" });
            this.set_accels_for_action ("app.hamburger-menu", { "F10" });
            this.set_accels_for_action ("app.help", { "F1" });
            this.set_accels_for_action ("app.quit", { "<Primary>q" });
            this.set_accels_for_action ("app.shortcuts", { "<Primary>question" });
        }

        public override void startup () {
            base.startup ();
            Adw.init ();

            if (((!) Gtk.Settings.get_default ()).gtk_theme_name.contains ("-dark")) {
                warning ("Using -dark themes (such as Adwaita-dark) is unsupported. Please use your theme in dark-mode instead (e.g. Adwaita:dark instead of Adwaita-dark)");
            }

            this.add_action_entries (APP_ENTRIES, this);
        }

        private void on_about (GLib.SimpleAction action, GLib.Variant? parameter) {
            string[] authors = {
                "Rasmus Thomsen <oss@cogitri.dev>"
            };

            Gtk.show_about_dialog (
                this.window,
                logo_icon_name: Config.APPLICATION_ID,
                program_name: _ ("Health"),
                comments: _ ("A health tracking app for the GNOME desktop."),
                authors: authors,
                translator_credits: _ ("translator-credits"),
                website: "https://gitlab.gnome.org/Cogitri/gnome-health",
                website_label: _ ("Websites"),
                version: Config.VERSION,
                license_type: Gtk.License.GPL_3_0
                );
        }

        private void on_fullscreen (GLib.SimpleAction action, GLib.Variant? parameter) {
            if (this.window != null) {
                unowned var window = (!) this.window;
                if (window.is_fullscreen ()) {
                    window.unfullscreen ();
                } else {
                    window.fullscreen ();
                }
            }
        }

        private void on_hamburger_menu (GLib.SimpleAction action, GLib.Variant? parameter) {
            if (this.window != null) {
                ((!) this.window).open_hamburger_menu ();
            }
        }

        private void on_help (GLib.SimpleAction action, GLib.Variant? parameter) {
        }

        private void on_preferences (GLib.SimpleAction action, GLib.Variant? parameter) {
            var pref_window = new PreferencesWindow (this.window);
            pref_window.import_done.connect (() => {
                if (this.window != null) {
                    ((!) this.window).update ();
                }
            });
        }

        private void on_quit (GLib.SimpleAction action, GLib.Variant? parameter) {
            if (this.window != null) {
                ((!) this.window).destroy ();
            }
        }

        private void on_shortcuts (GLib.SimpleAction action, GLib.Variant? parameter) {
            var builder = new Gtk.Builder.from_resource ("/dev/Cogitri/Health/ui/shortcuts_window.ui");
            var shortcuts_window = (Gtk.ShortcutsWindow) builder.get_object ("shortcuts_window");
            shortcuts_window.show ();
        }
    }
}

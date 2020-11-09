namespace Health {
    public class Application : Gtk.Application {
        private Settings settings;
        private weak Window? window;

        public Application () {
            Object (application_id: Config.APPLICATION_ID, flags : ApplicationFlags.FLAGS_NONE);
            this.settings = new Settings ();
        }

        private const GLib.ActionEntry APP_ENTRIES[] = {
            { "about", on_about },
            { "preferences", on_preferences },
        };

        public override void activate () {
            if (this.window != null) {
                return;
            } else if (this.settings.did_initial_setup) {
                var window = new Window (this, settings);
                window.show ();
                this.window = window;
            } else {
                var setup_window = new SetupWindow (this, this.settings);
                setup_window.setup_done.connect (() => {
                    this.settings.did_initial_setup = true;
                    var window = new Window (this, settings);
                    window.show ();
                    this.window = window;
                });
                setup_window.show ();
            }

            this.set_accels_for_action ("app.quit", { "<Primary>q" });

        }

        public override void startup () {
            base.startup ();
            Hdy.init ();

            if (Gtk.Settings.get_default ().gtk_theme_name.contains ("-dark")) {
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

        private void on_preferences (GLib.SimpleAction action, GLib.Variant? parameter) {
            var pref_window = new PreferencesWindow (this.settings, this.window);
            pref_window.import_done.connect (() => {
                if (this.window != null) {
                    this.window.update ();
                }
            });
        }

    }
}

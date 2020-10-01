namespace Health {
    public class Application : Gtk.Application {
        private Settings settings;
        private Window? window;

        public Application () {
            Object (application_id: Config.APPLICATION_ID, flags : ApplicationFlags.FLAGS_NONE);
            this.settings = new Settings ();
        }

        private const GLib.ActionEntry APP_ENTRIES[] = {
            { "about", on_about },
            { "preferences", on_preferences },
            { "quit", on_quit },
            { "units", on_units, "s" },
        };

        public override void activate () {
            if (window != null) {
                return;
            } else if (this.settings.did_initial_setup) {
                this.window = new Window (this, settings);
                ((!) this.window).show ();
            } else {
                var setup_window = new SetupWindow (this, this.settings);
                setup_window.setup_done.connect (() => {
                    setup_window.destroy ();
                    this.settings.did_initial_setup = true;
                    this.window = new Window (this, settings);
                    ((!) this.window).show ();
                });
                setup_window.show ();
            }

            this.set_accels_for_action ("app.quit", { "<Primary>q" });

        }

        public override void startup () {
            base.startup ();
            Hdy.init ();

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
            new PreferencesWindow (this.settings, this.window);
        }

        private void on_units (GLib.SimpleAction action, GLib.Variant? parameter) {
            if (parameter == null) {
                return;
            }
            var unitsystem = ((!) parameter).get_string ();
            switch ((!) unitsystem) {
                case "imperial":
                    this.settings.unitsystem = Unitsystem.IMPERIAL;
                    break;
                case "metric":
                    this.settings.unitsystem = Unitsystem.METRIC;
                    break;
                default:
                    warning (_ ("Unknown unitsystem %s"), unitsystem);
                    break;
            }
        }

        private void on_quit (GLib.SimpleAction action, GLib.Variant? parameter) {
            if (this.window != null) {
                ((!) this.window).destroy ();
            }
        }

    }
}

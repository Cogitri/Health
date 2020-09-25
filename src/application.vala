namespace Health {
    public class Application : Gtk.Application {
        private Window window;

        public Application () {
            Object (application_id: Config.APPLICATION_ID, flags : ApplicationFlags.FLAGS_NONE);
        }

        private const GLib.ActionEntry APP_ENTRIES[] = {
            { "about", on_about },
            { "quit", on_quit },
        };

        public override void activate () {
            if (window != null) {
                return;
            }

            this.window = new Window (this);
            this.set_accels_for_action ("app.quit", { "<Primary>q" });
            window.show ();
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
                comments: _ (""),
                authors: authors,
                translator_credits: _ ("translator-credits"),
                website: "https://gitlab.gnome.org/Cogitri/gnome-health",
                website_label: _ ("Websites"),
                version: Config.VERSION,
                license_type: Gtk.License.GPL_3_0
                );
        }

        private void on_quit (GLib.SimpleAction action, GLib.Variant? parameter) {
            this.window.destroy ();
        }

    }
}

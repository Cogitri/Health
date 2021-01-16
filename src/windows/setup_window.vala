/* setup_window.vala
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
     * The {@link SetupWindow} is shown to the user on the first start of the applcation to fill in some data.
     */
    [GtkTemplate (ui = "/dev/Cogitri/Health/ui/setup_window.ui")]
    public class SetupWindow : Adw.ApplicationWindow {
        [GtkChild]
        private BMILevelBar bmi_levelbar;
        [GtkChild]
        private Gtk.Box setup_first_page;
        [GtkChild]
        private Gtk.Box setup_second_page;
        [GtkChild]
        private Gtk.Box setup_third_page;
        [GtkChild]
        private Gtk.Box setup_fourth_page;
        [GtkChild]
        private Gtk.Button setup_done_button;
        [GtkChild]
        private Gtk.Button setup_quit_button;
        [GtkChild]
        private Gtk.Button setup_next_page_button;
        [GtkChild]
        private Gtk.Button setup_previous_page_button;
        [GtkChild]
        private Gtk.Stack setup_right_stack;
        [GtkChild]
        private Gtk.Stack setup_left_stack;
        [GtkChild]
        private Gtk.SpinButton age_spin_button;
        [GtkChild]
        private Gtk.SpinButton height_spin_button;
        [GtkChild]
        private Gtk.SpinButton stepgoal_spin_button;
        [GtkChild]
        private Gtk.SpinButton weightgoal_spin_button;
        [GtkChild]
        private Gtk.ToggleButton unit_metric_togglebutton;
        [GtkChild]
        private Adw.ActionRow height_actionrow;
        [GtkChild]
        private Adw.ActionRow weightgoal_actionrow;
        [GtkChild]
        private Adw.Carousel setup_carousel;

        private Settings settings;

        /**
         * This signal is fired when the user presses the setup_finish_button and all input data has been saved to GSettings.
         */
        public signal void setup_done ();

        static construct {
            typeof (SyncListBox).ensure ();
        }

        public SetupWindow (Gtk.Application application) {
            Object (application: application);
            this.settings = Settings.get_instance ();
            this.stepgoal_spin_button.value = 10000;

            var provider = new Gtk.CssProvider ();
            provider.load_from_resource ("/dev/Cogitri/Health/custom.css");
            Gtk.StyleContext.add_provider_for_display (this.get_display (), provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION);
        }

        private void try_enable_next_button () {
            unowned var age_text = this.age_spin_button.text;
            unowned var height_text = this.height_spin_button.text;
            var filled_in_data = age_text != "0" && age_text != "" && height_text != "0" && height_text != "";
            this.setup_next_page_button.sensitive = filled_in_data;
            this.setup_carousel.interactive = filled_in_data;
        }

        private void set_optimal_weightgoal () {
            const double OPTIMAL_BMI = 22.5;
            var height_in_cm = double.parse (this.height_spin_button.text);
            if (!this.unit_metric_togglebutton.active) {
                height_in_cm = Util.inch_to_cm (height_in_cm);
            }
            var optimal_value = OPTIMAL_BMI * GLib.Math.pow (height_in_cm / 100, 2);
            if (!this.unit_metric_togglebutton.active) {
                optimal_value = Util.kg_to_pb (optimal_value);
            }
            this.weightgoal_spin_button.value = optimal_value;
        }

        [GtkCallback]
        private void unit_metric_togglebutton_toggled (Gtk.ToggleButton btn) {
            if (btn.active) {
                this.height_actionrow.title = _ ("Height in centimeters");
                this.weightgoal_actionrow.title = _ ("Weightgoal in KG");
                this.bmi_levelbar.unitsystem = Unitsystem.METRIC;
                this.height_spin_button.value = Util.inch_to_cm (this.height_spin_button.value);
            } else {
                this.height_actionrow.title = _ ("Height in inch");
                this.weightgoal_actionrow.title = _ ("Weightgoal in pounds");
                this.bmi_levelbar.unitsystem = Unitsystem.IMPERIAL;
                this.height_spin_button.value = Util.cm_to_inch (this.height_spin_button.value);
            }
            this.set_optimal_weightgoal ();
        }

        [GtkCallback]
        private void height_spin_button_changed (Gtk.Editable editable) {
            this.set_optimal_weightgoal ();
            this.try_enable_next_button ();
            this.bmi_levelbar.height = double.parse (this.height_spin_button.text);
        }

        [GtkCallback]
        private void age_spin_button_changed (Gtk.Editable editable) {
            this.try_enable_next_button ();
        }

        [GtkCallback]
        private void weightgoal_spin_button_changed (Gtk.Editable editable) {
            this.bmi_levelbar.weight = double.parse (editable.text);
        }

        [GtkCallback]
        private void setup_done_button_clicked (Gtk.Button btn) {
            var height_in_cm = uint.parse (this.height_spin_button.text);
            if (this.unit_metric_togglebutton.active) {
                this.settings.unitsystem = Unitsystem.METRIC;
            } else {
                this.settings.unitsystem = Unitsystem.IMPERIAL;
                height_in_cm = (uint) GLib.Math.round (Util.inch_to_cm (height_in_cm));
            }

            this.settings.user_age = uint.parse (this.age_spin_button.text);
            this.settings.user_height = height_in_cm;
            this.settings.user_stepgoal = uint.parse (this.stepgoal_spin_button.text);
            this.settings.user_weightgoal = new WeightUnitContainer.from_user_value (double.parse (this.weightgoal_spin_button.text));
            this.setup_done ();
            this.destroy ();
        }

        [GtkCallback]
        private void setup_quit_button_clicked (Gtk.Button btn) {
            this.destroy ();
        }

        [GtkCallback]
        private void setup_carousel_page_changed (uint index) {
            if (this.setup_carousel.n_pages - 1 == index) {
                this.setup_done_button.visible = true;
                this.setup_right_stack.set_visible_child (this.setup_done_button);
            } else if (index == 0) {
                this.setup_quit_button.visible = true;
                this.setup_left_stack.set_visible_child (this.setup_quit_button);
            } else {
                this.setup_next_page_button.visible = true;
                this.setup_previous_page_button.visible = true;
                this.setup_right_stack.set_visible_child (this.setup_next_page_button);
                this.setup_left_stack.set_visible_child (this.setup_previous_page_button);
            }
        }

        [GtkCallback]
        private void setup_next_page_button_clicked () {
            var current_page = (uint) this.setup_carousel.position;
            switch (current_page) {
                case 0:
                    this.setup_carousel.scroll_to (this.setup_second_page);
                    break;
                case 1:
                    this.setup_carousel.scroll_to (this.setup_third_page);
                    break;
                case 2:
                    this.setup_carousel.scroll_to (this.setup_fourth_page);
                    break;
                default:
                    error ("Scrollled to unknown page %u", current_page);
            }
        }

        [GtkCallback]
        private void setup_previous_page_button_clicked () {
            var current_page = (uint) this.setup_carousel.position;
            switch (current_page) {
                case 0:
                    // FIXME: This happens when the user scrolls back quickly and presses the "Previous" button on the last page before the "Quit" button appears
                    this.destroy ();
                    break;
                case 1:
                    this.setup_carousel.scroll_to (this.setup_first_page);
                    break;
                case 2:
                    this.setup_carousel.scroll_to (this.setup_second_page);
                    break;
                case 3:
                    this.setup_carousel.scroll_to (this.setup_third_page);
                    break;
                default:
                    error ("Scrollled to unknown page %u", current_page);
            }
        }
    }
}
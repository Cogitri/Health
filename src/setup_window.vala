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
    [GtkTemplate (ui = "/dev/Cogitri/Health/setup_window.ui")]
    public class SetupWindow : Hdy.ApplicationWindow {
        [GtkChild]
        private SyncView sync_view;
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
        private Gtk.ToggleButton unit_metric_togglebutton;
        [GtkChild]
        private Gtk.Stack setup_right_stack;
        [GtkChild]
        private Gtk.Stack setup_left_stack;
        [GtkChild]
        private Gtk.SpinButton age_spinner;
        [GtkChild]
        private Gtk.SpinButton height_spinner;
        [GtkChild]
        private Gtk.SpinButton stepgoal_spinner;
        [GtkChild]
        private Gtk.SpinButton weightgoal_spinner;
        [GtkChild]
        private Hdy.ActionRow height_actionrow;
        [GtkChild]
        private Hdy.Carousel setup_carousel;

        /**
         * This signal is fired when the user presses the setup_finish_button and all input data has been saved to GSettings.
         */
        public signal void setup_done ();

        public SetupWindow (Gtk.Application application, Settings settings) {
            Object (application: application);

            this.stepgoal_spinner.value = 10000;
            this.unit_metric_togglebutton.active = true;
            this.height_actionrow.title = _ ("Height in centimeters");
            this.sync_view.parent_window = this;
            this.sync_view.settings = settings;

            this.unit_metric_togglebutton.toggled.connect ((btn) => {
                if (btn.active) {
                    this.height_actionrow.title = _ ("Height in centimeters");
                } else {
                    this.height_actionrow.title = _ ("Height in inch");
                }
                this.set_optimal_weightgoal ();
            });
            this.height_spinner.changed.connect (() => {
                this.set_optimal_weightgoal ();
                this.try_enable_next_button ();
            });
            this.age_spinner.changed.connect (() => {
                this.try_enable_next_button ();
            });
            this.setup_done_button.clicked.connect (() => {
                var height_in_cm = uint.parse (this.height_spinner.text);
                if (this.unit_metric_togglebutton.active) {
                    settings.unitsystem = Unitsystem.METRIC;
                } else {
                    settings.unitsystem = Unitsystem.IMPERIAL;
                    height_in_cm = (uint) GLib.Math.round (inch_to_cm (height_in_cm));
                }

                settings.user_age = uint.parse (this.age_spinner.text);
                settings.user_height = height_in_cm;
                settings.user_stepgoal = uint.parse (this.stepgoal_spinner.text);
                settings.user_weightgoal = new WeightUnitContainer.from_user_value (this.weightgoal_spinner.value, settings);
                this.setup_done ();
            });
            this.setup_quit_button.clicked.connect (() => {
                this.destroy ();
            });
            this.setup_right_stack.set_visible_child (this.setup_done_button);
            this.setup_carousel.page_changed.connect ((index) => {
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
            });
            this.setup_next_page_button.clicked.connect (() => {
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
            });
            this.setup_previous_page_button.clicked.connect (() => {
                var current_page = (uint) this.setup_carousel.position;
                switch (current_page) {
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
            });
        }

        private void try_enable_next_button () {
            this.setup_next_page_button.sensitive = this.height_spinner.get_text () != "0" && this.age_spinner.get_text () != "0";
        }

        private void set_optimal_weightgoal () {
            const uint OPTIMAL_BMI = 20;
            var height_in_cm = double.parse (this.height_spinner.text);
            if (!this.unit_metric_togglebutton.active) {
                height_in_cm = inch_to_cm (height_in_cm);
            }
            var optimal_value = OPTIMAL_BMI * GLib.Math.pow (height_in_cm / 100, 2);
            if (!this.unit_metric_togglebutton.active) {
                optimal_value = kg_to_pb (optimal_value);
            }
            this.weightgoal_spinner.value = optimal_value;
        }
    }
}

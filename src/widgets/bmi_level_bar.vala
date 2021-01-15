/* activity_add_dialog.vala
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
     * Visualise the BMI of the user
     */
    [GtkTemplate (ui = "/dev/Cogitri/Health/ui/bmi_level_bar.ui")]
    class BMILevelBar : Gtk.Widget {
        [GtkChild]
        private Gtk.Label bmi_label;
        [GtkChild]
        private Gtk.LevelBar level_bar;

        private double _height;
        private double _weight;
        private Unitsystem _unitsystem;
        private const double LEVEL_BAR_MIN = 13.5;
        private const double LEVEL_BAR_MAX = 30;

        public double height {
            get {
                return this._height;
            }
            set {
                this._height = value;
                this.recalculate_bmi ();
            }
        }
        public double weight {
            get {
                return this._weight;
            }
            set {
                this._weight = value;
                this.recalculate_bmi ();
            }
        }
        public Unitsystem unitsystem {
            get {
                return this._unitsystem;
            }
            set {
                this._unitsystem = value;
                this.recalculate_bmi ();
            }
        }

        static construct {
            set_layout_manager_type (typeof (Gtk.BoxLayout));
            set_accessible_role (Gtk.AccessibleRole.METER);
        }

        construct {
            this._height = 1;
            this._weight = 1;

            var settings = Settings.get_instance ();
            settings.changed[Settings.UNITSYSTEM_KEY].connect (() => {
                this.unitsystem = settings.unitsystem;
            });

            this._unitsystem = settings.unitsystem;

            ((Gtk.Orientable) this.get_layout_manager ()).set_orientation (Gtk.Orientation.VERTICAL);

            this.level_bar.remove_offset_value (Gtk.LEVEL_BAR_OFFSET_LOW);
            this.level_bar.remove_offset_value (Gtk.LEVEL_BAR_OFFSET_HIGH);
            this.level_bar.remove_offset_value (Gtk.LEVEL_BAR_OFFSET_FULL);

            this.level_bar.add_offset_value ("severly-underweight-bmi", (18.5 - LEVEL_BAR_MIN) / (LEVEL_BAR_MAX - LEVEL_BAR_MIN));
            this.level_bar.add_offset_value ("underweight-bmi", (20 - LEVEL_BAR_MIN) / (LEVEL_BAR_MAX - LEVEL_BAR_MIN));
            this.level_bar.add_offset_value ("optimal-bmi", (25 - LEVEL_BAR_MIN) / (LEVEL_BAR_MAX - LEVEL_BAR_MIN));
            this.level_bar.add_offset_value ("overweight-bmi", (29.9 - LEVEL_BAR_MIN) / (LEVEL_BAR_MAX - LEVEL_BAR_MIN));
            this.level_bar.add_offset_value ("obese-bmi", 1);
        }

        ~BMILevelBar () {
            unowned Gtk.Widget? child;
            while ((child = get_first_child ()) != null) {
                ((!) child).unparent ();
            }
        }

        private void recalculate_bmi () {
            var current_bmi = 0.0;
            var fraction = 0.0;
            var height = this.height;
            var weight = this.weight;

            if (height != 0 && weight != 0) {
                if (this.unitsystem == Unitsystem.IMPERIAL) {
                    height = Util.inch_to_cm (height);
                    weight = Util.pb_to_kg (weight);
                }
                current_bmi = weight / GLib.Math.pow (height / 100, 2);
                // The BMI should be in the range of 18.5 to 24.9 and we want 5 as margin on both sides, so LEVEL_BAR_MIN is 0% and LEVEL_BAR_MAX 100%
                fraction = (current_bmi - LEVEL_BAR_MIN) / (LEVEL_BAR_MAX - LEVEL_BAR_MIN);
                if (fraction < 0) {
                    fraction = 0;
                } else if (fraction > 1) {
                    fraction = 1;
                }
            }

            this.level_bar.value = fraction;
            this.bmi_label.label = _ ("<small>Current BMI: %.2lf</small>").printf (current_bmi);
        }
    }
}

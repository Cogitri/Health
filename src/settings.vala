/* settings.vala
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
    public enum Unitsystem {
        IMPERIAL,
        METRIC,
    }

    /**
     * Settings utilizes GSettings to save the user's preferences.
     */
    public class Settings : GLib.Settings {
        public const string DID_INITIAL_SETUP_KEY = "did-initial-setup";
        public const string UNITSYSTEM_KEY = "unitsystem";
        public const string USER_AGE_KEY = "user-age";
        public const string USER_HEIGHT_KEY = "user-height";
        public const string USER_STEPGOAL_KEY = "user-stepgoal";
        public const string USER_WEIGHTGOAL_KEY = "user-weightgoal";
        public const string WINDOW_HEIGHT_KEY = "window-height";
        public const string WINDOW_IS_MAXIMIZED_KEY = "window-is-maximized";
        public const string WINDOW_WIDTH_KEY = "window-width";

        public bool did_initial_setup {
            get {
                return this.get_boolean (DID_INITIAL_SETUP_KEY);
            }
            set {
                this.set_boolean (DID_INITIAL_SETUP_KEY, value);
            }
        }

        public Unitsystem unitsystem {
            get {
                return (Unitsystem) this.get_enum (UNITSYSTEM_KEY);
            }
            set {
                this.set_enum (UNITSYSTEM_KEY, value);
            }
        }

        public uint user_age {
            get {
                return this.get_uint (USER_AGE_KEY);
            }
            set {
                this.set_uint (USER_AGE_KEY, value);
            }
        }

        public uint user_height {
            get {
                return this.get_uint (USER_HEIGHT_KEY);
            }
            set {
                this.set_uint (USER_HEIGHT_KEY, value);
            }
        }

        public uint user_stepgoal {
            get {
                return this.get_uint (USER_STEPGOAL_KEY);
            }
            set {
                this.set_uint (USER_STEPGOAL_KEY, value);
            }
        }

        public WeightUnitContainer user_weightgoal {
            owned get {
                return new WeightUnitContainer.from_database_value (this.get_double (USER_WEIGHTGOAL_KEY), this);
            }
            set {
                this.set_double (USER_WEIGHTGOAL_KEY, value.get_in_kg ());
            }
        }

        public int window_height {
            get {
                return this.get_int (WINDOW_HEIGHT_KEY);
            }
            set {
                this.set_int (WINDOW_HEIGHT_KEY, value);
            }
        }

        public bool window_is_maximized {
            get {
                return this.get_boolean (WINDOW_IS_MAXIMIZED_KEY);
            }
            set {
                this.set_boolean (WINDOW_IS_MAXIMIZED_KEY, value);
            }
        }

        public int window_width {
            get {
                return this.get_int (WINDOW_WIDTH_KEY);
            }
            set {
                this.set_int (WINDOW_WIDTH_KEY, value);
            }
        }

        public Settings () {
            Object (schema_id: Config.APPLICATION_ID);
        }
    }
 }

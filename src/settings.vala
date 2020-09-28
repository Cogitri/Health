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
    public class Settings : GLib.Settings {
        private const string DID_INITIAL_SETUP_KEY = "did-initial-setup";
        private const string USER_AGE_KEY = "user-age";
        private const string USER_HEIGHT_KEY = "user-height";

        public bool did_initial_setup {
            get {
                return this.get_boolean (DID_INITIAL_SETUP_KEY);
            }
            set {
                this.set_boolean (DID_INITIAL_SETUP_KEY, value);
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

        public Settings () {
            Object (schema_id: Config.APPLICATION_ID);
        }
    }
 }

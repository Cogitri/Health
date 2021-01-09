/* units.vala
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
     * A Container type which always converts the value to the right unitsystem (imperial or metric).
     */
    public class WeightUnitContainer : GLib.Object {
        private Settings settings;

        protected double _value;
        public double value {
            get {
                if (settings.unitsystem == Unitsystem.IMPERIAL) {
                    return Util.kg_to_pb (this._value);
                }
                return this._value;
            }
            set {
                if (settings.unitsystem == Unitsystem.IMPERIAL) {
                    this._value = Util.pb_to_kg (value);
                } else {
                    this._value = value;
                }
            }
        }

        /**
        * Construct from a database value (in KG!)
        */
        public WeightUnitContainer.from_database_value (double weight_in_kg) {
            this.settings = Settings.get_instance ();
            this._value = weight_in_kg;
        }

        /**
         * Construct from a value from the user (which can be PB or KG).
         *
         * If the unitsystem property of Settings is set to Imperial, we'll convert to KG here,
         * if not it is assumed that the weight parameter is in KG.
         */
        public WeightUnitContainer.from_user_value (double weight) {
            this.settings = Settings.get_instance ();
            this.value = weight;
        }

        /**
         * Get the weight value in KG (e.g. for BMI calculations).
         */
        public double get_in_kg () {
            return this._value;
        }

    }

}

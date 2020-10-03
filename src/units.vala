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


    public class WeightUnitContainer : GLib.Object {
        private Settings settings;

        protected double _value;
        public double value {
            get {
                if (settings.unitsystem == Unitsystem.IMPERIAL) {
                    return kg_to_pb(this._value);
                }
                return this._value;
            }
            set {
                if (settings.unitsystem == Unitsystem.IMPERIAL) {
                    this._value = pb_to_kg(value);
                } else {
                    this._value = value;
                }
            }
        }


        public WeightUnitContainer.from_database_value (double weight_in_kg, Settings settings) {
            this.settings = settings;
            this._value = weight_in_kg;
        }

        public WeightUnitContainer.from_user_value (double weight, Settings settings) {
            this.settings = settings;
            this.value = weight;
        }

        public double get_in_kg () {
            return this._value;
        }

    }

}

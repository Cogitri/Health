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

    public abstract class UnitContainer<T> : GLib.Object {
        protected T _value;
        public T value {
            owned get {
                return this.get_func ();
            }
            set {
                this.set_func (value);
            }
        }

        protected abstract void set_func (T value);

        protected abstract T get_func ();
    }

    public class WeightUnitContainer : UnitContainer<double?> {
        private Settings settings;

        public WeightUnitContainer.from_database_value (double weight_in_kg, Settings settings) {
            this.settings = settings;
            this._value = weight_in_kg;
        }

        public WeightUnitContainer.from_user_value (double weight_in_kg, Settings settings) {
            this.settings = settings;
            this.value = weight_in_kg;
        }

        public double get_in_kg () {
            return this._value;
        }

        protected override double? get_func () {
            if (settings.unitsystem == Unitsystem.IMPERIAL) {
                return kg_to_pb(this._value);
            }
            return this._value;
        }

        protected override void set_func (double? new_value) {
            if (settings.unitsystem == Unitsystem.IMPERIAL) {
                this._value = pb_to_kg(new_value);
            } else {
                this._value = new_value;
            }
        }
    }

}

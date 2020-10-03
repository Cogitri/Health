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
    void add_weight_tests () {
        Test.add_func ("/units/weight/from_user", () => {
            var settings = new Settings ();
            settings.unitsystem = Unitsystem.IMPERIAL;
            var val = new WeightUnitContainer.from_user_value (100, settings);
            assert (val.value == 100);
            assert (val.get_in_kg () == 45.359237);
        });
    }

    void main (string[] args) {
        Test.init (ref args);
        add_weight_tests ();
        Test.run ();
    }
}

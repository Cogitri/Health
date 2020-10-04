/* test_main.vala
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

    int main (string[] args) {
        Test.init (ref args);

        var suite = new TestSuite ("main");
        suite.add_suite (new WeightUnitContainerTest ().suite);
        suite.add_suite (new SqliteDatabaseTest ().suite);

        TestSuite root = TestSuite.get_root ();
        root.add_suite (suite);

        MainLoop loop = new MainLoop ();

        int ret = -1;
        Idle.add (() => {
                ret = Test.run ();
                loop.quit ();
                return false;
            });

        loop.run ();
        return ret;
    }

}

/* activity.vala
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

    public class Activity : GLib.Object {
        public Activities.Enum activity_type;
        public GLib.Date date { get; private set; }
        public uint32 minutes { get; private set; }
        public uint32 steps { get; private set; }

        public Activity (Activities.Enum activity_type, GLib.Date date, uint32 minutes, uint32 steps = 0) {
            this.activity_type = activity_type;
            this.date = date;
            this.minutes = minutes;
            this.steps = steps;
        }

        public uint32 get_calories_burned () {
            return this.minutes * Activities.get_values ()[this.activity_type].average_calories_burned_per_minute;
        }
    }

    public class Activities : GLib.Object {
        public enum Enum {
            BASKETBALL,
            BICYCLING,
            BOXING,
            DANCING,
            FOOTBALL,
            GOLF,
            HIKING,
            HOCKEY,
            HORSE_RIDING,
            OTHER_SPORTS,
            ROLLERBLADING,
            RUNNING,
            SKIING,
            SOCCER,
            SOFTBALL,
            SWIMMING,
            TENNIS,
            TRACK_AND_FIELD,
            VOLLEYBAL,
            WALKING,
        }

        public struct ActivityInfo {
            Activities.Enum type;
            string name;
            bool has_steps;
            // As per https://keisan.casio.com/menu/system/000000000140
            uint32 average_calories_burned_per_minute;
            // As per https://www.arhs-nc.org/live-healthy/data/StepConversionChart.pdf
            // uint32 steps_per_minute;
        }

        public static ActivityInfo? get_info_by_name (string name) {
            foreach (var x in Activities.get_values ()) {
                if (x.name == name) {
                    return x;
                }
            }

            return null;
        }

        public static ActivityInfo[] get_values () {
            return {
                {Enum.BASKETBALL, _ ("Basketball"), false, 6},
                {Enum.BICYCLING, _ ("Bicycling"), false, 10},
                {Enum.BOXING, _ ("Boxing"), false, 7},
                {Enum.DANCING, _ ("Dancing"), false, 8},
                {Enum.FOOTBALL, _ ("Football"), false, 3},
                {Enum.GOLF, _ ("Golf"), false, 4},
                {Enum.HIKING, _ ("Hiking"), true, 8},
                {Enum.HOCKEY, _ ("Hockey"), false, 10},
                {Enum.HORSE_RIDING, _ ("Horse Riding"), false, 5},
                {Enum.OTHER_SPORTS, _ ("Other Sports"), false, 9},
                {Enum.ROLLERBLADING, _ ("Rollerblading"), false, 10},
                {Enum.RUNNING, _ ("Running"), true, 15},
                {Enum.SKIING, _ ("Skiing"), false, 12},
                {Enum.SOCCER, _ ("Soccer"), false, 8},
                {Enum.SOFTBALL, _ ("Softball"), false, 5},
                {Enum.SWIMMING, _ ("Swimming"), false, 12},
                {Enum.TENNIS, _ ("Tennis"), false, 6},
                {Enum.TRACK_AND_FIELD, _ ("Track And Field"), false, 5},
                {Enum.VOLLEYBAL, _ ("Volleyball"), false, 4},
                {Enum.WALKING, _ ("Walking"), true, 5},
            };
        }
    }
}

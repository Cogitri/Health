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

    // TODO: Additional data points which might (!) make sense down the road:
    // * Oxygen saturation & VO2
    // * Body temperature
    // * Blood pressure
    [Flags]
    public enum ActivityDataPoints {
        CALORIES_BURNED,
        DISTANCE,
        DURATION,
        HEART_RATE,
        STEP_COUNT,
    }

    public class Activity : GLib.Object {

        public Activities.Enum activity_type { get; set; }
        public GLib.Date date { get; set; }
        public uint32 calories_burned { get; set; }
        public uint32 distance { get; set; }
        public uint32 hearth_rate_avg { get; set; }
        public uint32 hearth_rate_max { get; set; }
        public uint32 hearth_rate_min { get; set; }
        public uint32 minutes { get; set; }
        public uint32 steps { get; set; }

        public Activity (
            Activities.Enum activity_type,
            GLib.Date date,
            uint32 calories_burned,
            uint32 distance,
            uint32 hearth_rate_avg,
            uint32 hearth_rate_max,
            uint32 hearth_rate_min,
            uint32 minutes,
            uint32 steps
        ) {
            Object (
                activity_type: activity_type,
                date: date,
                distance: distance,
                hearth_rate_avg: hearth_rate_avg,
                hearth_rate_max: hearth_rate_max,
                hearth_rate_min: hearth_rate_min,
                minutes: minutes,
                steps: steps
            );
        }


        public uint32? get_estimated_minutes (bool steps_changed) {
            if (this.steps != 0 && steps_changed) {
                switch (this.activity_type) {
                    case Activities.Enum.WALKING:
                    case Activities.Enum.HIKING:
                        return this.steps / 100;
                    case Activities.Enum.RUNNING:
                        return this.steps / 150;
                }
            }

            if (this.calories_burned != 0 && !steps_changed) {
                return this.calories_burned / Activities.get_values ()[this.activity_type].average_calories_burned_per_minute;
            }

            return null;
        }

        public uint32? get_estimated_calories_burned (bool steps_changed) {
            if (steps_changed) {
                if (this.steps != 0) {
                    switch (this.activity_type) {
                    case Activities.Enum.WALKING:
                        return this.steps / 100;
                    case Activities.Enum.HIKING:
                        return this.steps / 120;
                    case Activities.Enum.RUNNING:
                        return this.steps / 150;
                    }
                }
            } else {
                if (this.minutes == 0) {
                    return null;
                }

                return Activities.get_values ()[this.activity_type].average_calories_burned_per_minute * this.minutes;
            }

            return null;
        }

        public uint32? get_estimated_distance () {
            if (this.steps == 0) {
                return null;
            }

            switch (this.activity_type) {
                case Activities.Enum.WALKING:
                case Activities.Enum.HIKING:
                case Activities.Enum.RUNNING:
                    return (uint32) (this.steps / 1.4);
                default:
                    return null;
            }
        }

        public uint32? get_estimated_steps (bool distance_changed) {
            if (this.distance != 0 && distance_changed) {
                switch (this.activity_type) {
                    case Activities.Enum.WALKING:
                    case Activities.Enum.HIKING:
                    case Activities.Enum.RUNNING:
                        return (uint32) (this.distance * 1.4);
                }
            }

            if (this.minutes != 0 && !distance_changed) {
                switch (this.activity_type) {
                    case Activities.Enum.WALKING:
                    case Activities.Enum.HIKING:
                        return this.minutes * 100;
                    case Activities.Enum.RUNNING:
                        return this.minutes * 150;
                }
            }

            return null;
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
            ActivityDataPoints available_data_points;
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
                {Enum.BASKETBALL, _ ("Basketball"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 6},
                {Enum.BICYCLING, _ ("Bicycling"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DISTANCE, 10},
                {Enum.BOXING, _ ("Boxing"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 7},
                {Enum.DANCING, _ ("Dancing"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 8},
                {Enum.FOOTBALL, _ ("Football"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 3},
                {Enum.GOLF, _ ("Golf"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION, 4},
                {Enum.HIKING, _ ("Hiking"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.STEP_COUNT | ActivityDataPoints.DISTANCE | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DURATION, 8},
                {Enum.HOCKEY, _ ("Hockey"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 10},
                {Enum.HORSE_RIDING, _ ("Horse Riding"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DISTANCE, 5},
                {Enum.OTHER_SPORTS, _ ("Other Sports"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 9},
                {Enum.ROLLERBLADING, _ ("Rollerblading"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DISTANCE, 10},
                {Enum.RUNNING, _ ("Running"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.STEP_COUNT | ActivityDataPoints.DISTANCE | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DURATION, 15},
                {Enum.SKIING, _ ("Skiing"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DISTANCE, 12},
                {Enum.SOCCER, _ ("Soccer"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 8},
                {Enum.SOFTBALL, _ ("Softball"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 5},
                {Enum.SWIMMING, _ ("Swimming"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DISTANCE, 12},
                {Enum.TENNIS, _ ("Tennis"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 6},
                {Enum.TRACK_AND_FIELD, _ ("Track And Field"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DISTANCE | ActivityDataPoints.STEP_COUNT, 5},
                {Enum.VOLLEYBAL, _ ("Volleyball"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 4},
                {Enum.WALKING, _ ("Walking"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.STEP_COUNT | ActivityDataPoints.DISTANCE | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DURATION, 5},
            };
        }
    }
}

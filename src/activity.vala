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

    /**
     * A single Activity the user did. 
     */
    public class Activity : GLib.Object {

        public ActivityType activity_type {
            get {
                return this.activity_info.type;
            }
            set {
                this.activity_info = ActivityType.get_values ()[value];
            }
        }
        public GLib.Date date { get; set; }
        public uint32 calories_burned { get; set; }
        public uint32 distance { get; set; }
        public uint32 hearth_rate_avg { get; set; }
        public uint32 hearth_rate_max { get; set; }
        public uint32 hearth_rate_min { get; set; }
        public uint32 minutes { get; set; }
        public uint32 steps { get; set; }

        private ActivityInfo activity_info;
        private const uint BICYCLING_METERS_PER_MINUTE = 300;
        private const uint HORSE_RIDING_METERS_PER_MINUTE = 260;
        private const uint ROLLER_BLADING_METERS_PER_MINUTE = 240;
        private const uint RUNNING_METERS_PER_MINUTE = 200;
        private const uint SKIING_METERS_PER_MINUTE = 400;
        private const uint SWIMMING_METERS_PER_MINUTE = 160;
        private const uint WALKING_METERS_PER_MINUTE = 90;

        public Activity (
            ActivityType activity_type,
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

        /**
         * If {@link calories_burned} is set, try to autofill the other fields like
         * minutes, steps & distance of the Activity by estimating their values. 
         */
        public void autofill_from_calories () {
            if (this.calories_burned != 0 && ActivityDataPoints.CALORIES_BURNED in this.activity_info.available_data_points) {
                this.minutes = this.calories_burned / ActivityType.get_values ()[this.activity_type].average_calories_burned_per_minute;

                this.autofill_from_minutes ();
            }
        }

        /**
         * If {@link minutes} is set, try to autofill the other fields like
         * calories_burned, steps & distance of the Activity by estimating their values. 
         */
        public void autofill_from_minutes () {
            if (this.minutes != 0 && ActivityDataPoints.DURATION in this.activity_info.available_data_points) {
                this.calories_burned = this.activity_info.average_calories_burned_per_minute * this.minutes;

                switch (this.activity_type) {
                    case ActivityType.BICYCLING:
                        this.distance = BICYCLING_METERS_PER_MINUTE * this.minutes;
                        break;
                    case ActivityType.HORSE_RIDING:
                        this.distance = HORSE_RIDING_METERS_PER_MINUTE * this.minutes;
                        break;
                    case ActivityType.HIKING:
                    case ActivityType.WALKING:
                        this.distance = WALKING_METERS_PER_MINUTE * this.minutes;
                        break;
                    case ActivityType.ROLLERBLADING:
                        this.distance = ROLLER_BLADING_METERS_PER_MINUTE * this.minutes;
                        break;
                    case ActivityType.RUNNING:
                    case ActivityType.TRACK_AND_FIELD:
                        this.distance = RUNNING_METERS_PER_MINUTE * this.minutes;
                        break;
                    case ActivityType.SKIING:
                        this.distance = SKIING_METERS_PER_MINUTE * this.minutes;
                        break;
                    case ActivityType.SWIMMING:
                        this.distance = SWIMMING_METERS_PER_MINUTE * this.minutes;
                        break;
                    // Other activities don't have ActivityDataPoints.DISTANCE
                    default:
                        break;
                }

                switch (this.activity_type) {
                    case ActivityType.WALKING:
                    case ActivityType.HIKING:
                        this.steps = this.minutes * 100;
                        break;
                    case ActivityType.RUNNING:
                        this.steps = this.minutes * 150;
                        break;
                    // Other activities don't have ActivityDataPoints.STEPS
                    default:
                        break;
                }
            }
        }

        /**
         * If {@link distance} is set, try to autofill the other fields like
         * calories, steps & minutes of the Activity by estimating their values. 
         */
        public void autofill_from_distance () {
            if (distance != 0 && ActivityDataPoints.DISTANCE in this.activity_info.available_data_points) {
                switch (this.activity_type) {
                    case ActivityType.BICYCLING:
                        this.minutes = this.distance / BICYCLING_METERS_PER_MINUTE;
                        break;
                    case ActivityType.HORSE_RIDING:
                        this.minutes = this.distance / HORSE_RIDING_METERS_PER_MINUTE;
                        break;
                    case ActivityType.HIKING:
                    case ActivityType.WALKING:
                        this.minutes = this.distance / WALKING_METERS_PER_MINUTE;
                        break;
                    case ActivityType.ROLLERBLADING:
                        this.minutes = this.distance / ROLLER_BLADING_METERS_PER_MINUTE;
                        break;
                    case ActivityType.RUNNING:
                    case ActivityType.TRACK_AND_FIELD:
                        this.minutes = this.distance / RUNNING_METERS_PER_MINUTE;
                        break;
                    case ActivityType.SKIING:
                        this.minutes = this.distance / SKIING_METERS_PER_MINUTE;
                        break;
                    case ActivityType.SWIMMING:
                        this.minutes = this.distance / SWIMMING_METERS_PER_MINUTE;
                        break;
                    // Other activities don't have ActivityDataPoints.DISTANCE
                    default:
                        break;
                }

                this.calories_burned = this.minutes * this.activity_info.average_calories_burned_per_minute;

                switch (this.activity_type) {
                    case ActivityType.WALKING:
                    case ActivityType.HIKING:
                    case ActivityType.RUNNING:
                        this.steps = (uint32) (this.distance * 1.4);
                        break;
                    // Other activities don't have ActivityDataPoints.STEPS
                    default:
                        break;
                }
            }
        }

        /**
         * If {@link steps} is set, try to autofill the other fields like
         * minutes, calories_burned & distance of the Activity by estimating their values. 
         */
        public void autofill_from_steps () {
            if (this.steps != 0 && ActivityDataPoints.STEP_COUNT in this.activity_info.available_data_points) {
                switch (this.activity_type) {
                case ActivityType.WALKING:
                    this.minutes = this.steps / 100;
                    this.distance = this.minutes * WALKING_METERS_PER_MINUTE;
                    break;
                case ActivityType.HIKING:
                    this.minutes = this.steps / 80;
                    this.distance = this.minutes * WALKING_METERS_PER_MINUTE;
                    break;
                case ActivityType.RUNNING:
                    this.minutes = this.steps / 150;
                    this.distance = this.minutes * RUNNING_METERS_PER_MINUTE;
                    break;
                // Other activities don't have ActivityDataPoints.STEPS
                default:
                    break;
                }

                this.calories_burned = this.activity_info.average_calories_burned_per_minute * this.minutes;
            }
        }

    }

    /**
     * A struct containing information about a certain activity
     */
    public struct ActivityInfo {
        ActivityType type;
        string name;
        ActivityDataPoints available_data_points;
        // As per https://keisan.casio.com/menu/system/000000000140
        uint32 average_calories_burned_per_minute;
        // As per https://www.arhs-nc.org/live-healthy/data/StepConversionChart.pdf
        // uint32 steps_per_minute;
    }

    /**
     * A class containing information about different activity types. Contains all supported activity types.
     */
    public enum ActivityType {
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
        WALKING;

        public static ActivityInfo? get_info_by_name (string name) {
            foreach (var x in ActivityType.get_values ()) {
                if (x.name == name) {
                    return x;
                }
            }

            return null;
        }

        public static ActivityInfo[] get_values () {
            return {
                {BASKETBALL, _ ("Basketball"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 6},
                {BICYCLING, _ ("Bicycling"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DISTANCE, 10},
                {BOXING, _ ("Boxing"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 7},
                {DANCING, _ ("Dancing"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 8},
                {FOOTBALL, _ ("Football"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 3},
                {GOLF, _ ("Golf"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION, 4},
                {HIKING, _ ("Hiking"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.STEP_COUNT | ActivityDataPoints.DISTANCE | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DURATION, 8},
                {HOCKEY, _ ("Hockey"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 10},
                {HORSE_RIDING, _ ("Horse Riding"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DISTANCE, 5},
                {OTHER_SPORTS, _ ("Other Sports"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 9},
                {ROLLERBLADING, _ ("Rollerblading"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DISTANCE, 10},
                {RUNNING, _ ("Running"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.STEP_COUNT | ActivityDataPoints.DISTANCE | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DURATION, 15},
                {SKIING, _ ("Skiing"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DISTANCE, 12},
                {SOCCER, _ ("Soccer"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 8},
                {SOFTBALL, _ ("Softball"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 5},
                {SWIMMING, _ ("Swimming"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DISTANCE, 12},
                {TENNIS, _ ("Tennis"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 6},
                {TRACK_AND_FIELD, _ ("Track And Field"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DISTANCE | ActivityDataPoints.STEP_COUNT, 5},
                {VOLLEYBAL, _ ("Volleyball"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.DURATION | ActivityDataPoints.HEART_RATE, 4},
                {WALKING, _ ("Walking"), ActivityDataPoints.CALORIES_BURNED | ActivityDataPoints.STEP_COUNT | ActivityDataPoints.DISTANCE | ActivityDataPoints.HEART_RATE | ActivityDataPoints.DURATION, 5},
            };
        }
    }
}

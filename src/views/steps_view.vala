/* steps.vala
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
     * An implementation of {@link GraphView} that visualizes the user's step records over time.
     */
    public class StepsGraphView : GraphView {
        public StepsGraphView (StepsGraphModel model, double stepgoal) {
            base (model.to_points (), _ ("Stepgoal"), stepgoal);

            this.hover_func = (point) => {
                /* TRANSLATORS: This is shown on-hover of points where %u is the steps and %s is the already localised date (e.g. 2020-09-11) */
                return _ ("%u steps on %s").printf ((uint) point.value, Util.datetime_from_date (point.date).format ("%x"));
            };
            this.x_lines_interval = 500;
        }
    }


    /**
     * An implementation of {@link View} visualizes streak counts and daily step records.
     */
     [GtkTemplate (ui = "/dev/Cogitri/Health/ui/step_view.ui")]
    public class StepView : View {
        private Settings settings;
        private StepsGraphView? steps_graph_view;
        private StepsGraphModel steps_graph_model;

        public StepView (StepsGraphModel model, TrackerDatabase db) {
            this.settings = Settings.get_instance ();
            this.steps_graph_model = model;

            if (!this.steps_graph_model.is_empty) {
                this.steps_graph_view = new StepsGraphView (this.steps_graph_model, this.settings.user_stepgoal);
                this.scrolled_window.child = (!) this.steps_graph_view;
                this.stack.visible_child_name = "data_page";
            }

            this.settings.changed[Settings.USER_STEPGOAL_KEY].connect (() => {
                this.update ();
            });
            db.activities_updated.connect (() => {
                this.update ();
            });
            this.update ();
        }

        /**
         * Reload the {@link StepsGraphModel}'s data and refresh labels & the {@link StepsGraphView}.
         */
        public override void update () {
            this.steps_graph_model.reload.begin ((obj, res) => {
                if (this.steps_graph_model.reload.end (res)) {
                    this.title_label.set_text (_ ("Today's steps: %u").printf (this.steps_graph_model.get_today_step_count ()));
                    var streak_count = this.steps_graph_model.get_streak_count_today (this.settings.user_stepgoal);
                    switch (streak_count) {
                        case 0:
                            var previous_streak = this.steps_graph_model.get_streak_count_yesterday (this.settings.user_stepgoal);
                            if (previous_streak == 0) {
                                this.goal_label.set_text (_ ("No streak yet. Reach your stepgoal for multiple days to start a streak!"));
                            } else {
                                this.goal_label.set_text (_ ("You're on a streak for %u days. Reach your stepgoal today to continue it!").printf (previous_streak));
                            }
                            break;
                        case 1:
                            this.goal_label.set_text (_ ("You've reached your stepgoal today. Keep going to start a streak!"));
                            break;
                        default:
                            this.goal_label.set_text (_ ("You're on a streak for %u days. Good job!").printf (streak_count));
                            break;
                    }
                    this.title_label.set_text (_ ("Today's steps: %u").printf (this.steps_graph_model.get_today_step_count ()));

                    if (this.steps_graph_view == null && !this.steps_graph_model.is_empty) {
                        this.steps_graph_view = new StepsGraphView (this.steps_graph_model, this.settings.user_stepgoal);
                        this.scrolled_window.child = (!) this.steps_graph_view;
                        this.stack.visible_child_name = "data_page";
                    } else if (this.steps_graph_view != null) {
                        ((!) this.steps_graph_view).points = this.steps_graph_model.to_points ();
                        ((!) this.steps_graph_view).limit = this.settings.user_stepgoal;
                    }
                }
            });
        }

    }
}

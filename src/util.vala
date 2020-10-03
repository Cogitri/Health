/* util.vala
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
    double kg_to_pb (double kg) {
        return kg / 0.45359237;
    }

    double pb_to_kg (double pb) {
        return pb * 0.45359237;
    }

    double inch_to_cm (double inch) {
        return inch * 2.54;
    }

    double cm_to_inch (double cm) {
        return cm / 2.54;
    }

    GLib.Date get_today_date () {
        return get_date_in_n_days (0);
    }

    GLib.Date get_date_in_n_days (int days) {
        var datetime = new GLib.DateTime.now ().add_days (days);
        var date = GLib.Date ();
        date.set_dmy ((uchar) datetime.get_day_of_month (), datetime.get_month (), (uchar) datetime.get_year ());
        return date;
    }

}

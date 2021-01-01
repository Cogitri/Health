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
    public double kg_to_pb (double kg) {
        return kg / 0.45359237;
    }

    public double pb_to_kg (double pb) {
        return pb * 0.45359237;
    }

    public double inch_to_cm (double inch) {
        return inch * 2.54;
    }

    public double cm_to_inch (double cm) {
        return cm / 2.54;
    }

    public GLib.Date get_today_date () {
        return get_date_in_n_days (0);
    }

    public GLib.Date get_date_in_n_days (int days) {
        var datetime = new GLib.DateTime.now ().add_days (days);
        return date_from_datetime (datetime);
    }

    public GLib.Date date_from_datetime (GLib.DateTime datetime) {
        var date = GLib.Date ();
        date.set_dmy ((uchar) datetime.get_day_of_month (), datetime.get_month (), (ushort) datetime.get_year ());
        return date;
    }

    public GLib.DateTime datetime_from_date (GLib.Date date) {
        return new GLib.DateTime.local (date.get_year (), date.get_month (), date.get_day (), 0, 0, 0);
    }


    public string date_to_iso_8601 (GLib.Date d) {
        char[] buf = new char[20];
        assert (d.strftime (buf, "%Y-%m-%d") != 0);
        return (string) buf;
    }

    public GLib.Date iso_8601_to_date (string iso) {
        GLib.Date d = GLib.Date ();
        d.set_parse (iso);
        return d;
    }

    public double yard_to_meters (uint32 yard) {
        return yard * 0.9144;
    }

    public double meters_to_yard (uint32 meters) {
        return meters * 1.09361;
    }
}

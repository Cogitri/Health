/* add_dialog.vala
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

    public class AddDialog : Gtk.Dialog {
        protected Gtk.Label dialog_label;
        protected Gtk.Entry dialog_entry;

        public AddDialog (Gtk.Window? parent) {
            Object (use_header_bar: 1);
            this.set_transient_for (parent);
            this.destroy_with_parent = true;
            this.modal = true;
            this.dialog_label = new Gtk.Label (null);
            this.dialog_label.visible = true;
            this.dialog_entry = new Gtk.Entry ();
            this.dialog_entry.visible = true;
            this.dialog_entry.input_purpose = Gtk.InputPurpose.DIGITS;

            var content_box = this.get_content_area ();
            content_box.pack_start (this.dialog_label, true, true, 6);
            content_box.pack_start (this.dialog_entry, true, true, 6);

            this.add_button (_ ("Save"), Gtk.ResponseType.OK);
            this.add_button (_ ("Cancel"), Gtk.ResponseType.CANCEL);
            this.set_default_response (Gtk.ResponseType.OK);
            this.dialog_entry.changed.connect (() => {
                this.set_response_sensitive (Gtk.ResponseType.OK, this.dialog_entry.get_text_length () != 0);
            });

            this.response.connect ((response_id) => {
                switch (response_id) {
                case Gtk.ResponseType.OK:
                    try {
                        this.save ();
                    } catch (DatabaseError e) {
                        error ("Failed to save new data due to error %s", e.message);
                    }
                    break;
                }
                this.destroy ();
            });
        }

        public virtual void save () throws DatabaseError {
        }

    }

    public class StepsAddDialog : AddDialog {
        public StepsAddDialog (Gtk.Window? parent) {
            base (parent);
            this.dialog_label.set_text (_ ("Add new step record"));
            this.dialog_entry.set_max_length (6);
        }

        public override void save () throws DatabaseError {
            var db = new SqliteDatabase ();
            db.open ();

            uint64 steps = 0;
            try {
                uint64.from_string (this.dialog_entry.get_text (), out steps);
            } catch (NumberParserError e) {
                error ("Failed to parse steps due to error %s", e.message);
            }

            db.save_steps (new Steps (get_today_date (), (uint32) steps));
        }

    }

    public class WeightAddDialog : AddDialog {
        public WeightAddDialog (Gtk.Window? parent) {
            base (parent);
            this.dialog_label.set_text (_ ("Add new weight record"));
            this.dialog_entry.set_max_length (6);
        }

        public override void save () throws DatabaseError {
            var db = new SqliteDatabase ();
            db.open ();

            double weight = 0;
            if (!double.try_parse (this.dialog_entry.get_text (), out weight)) {
                error ("Failed to parse weight");
            }

            db.save_weight (new Weight (get_today_date (), weight));
        }

    }
}

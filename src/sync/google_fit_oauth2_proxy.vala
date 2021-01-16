/* google_fit_oauth2_proxy.vala
 *
 * Copyright 2021 Rasmus Thomsen <oss@cogitri.dev>
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

    public class GoogleFitOAuth2Proxy : OAuth2Proxy {
        public const string GOOGLE_API_ENDPOINT = "https://www.googleapis.com/fitness/v1/";
        public const string GOOGLE_API_KEY = "AIzaSyAefLTWEhVRHI4zwtLQ1w8szeP-V3wz8jg";
        public const string GOOGLE_AUTH_ENDPOINT_URL = "https://accounts.google.com/o/oauth2/v2/auth";
        public const string GOOGLE_CLIENT_SECRET = "QXYmZ982XWCdwKTW8mI3BbPp";
        public const string GOOGLE_CLIENT_ID = "652904425115-cdqjiporv9klugv9m7m0tpu44jt6cacb.apps.googleusercontent.com";
        public const string[] GOOGLE_API_SCOPES = {
            "https://www.googleapis.com/auth/fitness.activity.write",
            "https://www.googleapis.com/auth/fitness.body.write",
        };
        public override string provider_name {
            get {
                return "GoogleFit";
            }
        }

        private Settings settings;

        public GoogleFitOAuth2Proxy () {
            Object (client_id: GOOGLE_CLIENT_ID, auth_endpoint: GOOGLE_AUTH_ENDPOINT_URL, url_format: GOOGLE_API_ENDPOINT);

            this.settings = Settings.get_instance ();
        }

        public async Gee.HashMap<string, uint32> get_all_steps () throws GLib.Error {
            var call = this.new_call ();
            call.set_function ("users/me/dataSources/derived:com.google.step_count.delta:com.google.android.gms:merge_step_deltas/datasets/0-%lld".printf (Util.datetime_to_ns (new GLib.DateTime.now ())));
            yield call.invoke_async (null);
            return this.process_steps_json (call.get_payload ());
        }

        public async Gee.HashMap<string, double?> get_all_weights () throws GLib.Error {
            var call = this.new_call ();
            call.set_function ("users/me/dataSources/derived:com.google.weight:com.google.android.gms:merge_weight/datasets/0-%lld".printf (Util.datetime_to_ns (new GLib.DateTime.now ())));
            yield call.invoke_async (null);
            return this.process_weights_json (call.get_payload ());
        }

        public async Gee.HashMap<string, uint32> get_steps_since (GLib.DateTime since) throws GLib.Error {
            var call = this.new_call ();
            call.set_function ("users/me/dataSources/derived:com.google.step_count.delta:com.google.android.gms:merge_step_deltas/datasets/%lld-%lld".printf (Util.datetime_to_ns (since), Util.datetime_to_ns (new GLib.DateTime.now ())));
            yield call.invoke_async (null);
            return this.process_steps_json (call.get_payload ());
        }

        public async Gee.HashMap<string, double?> get_weights_since (GLib.DateTime since) throws GLib.Error {
            var call = this.new_call ();
            call.set_function ("users/me/dataSources/derived:com.google.weight:com.google.android.gms:merge_weight/datasets/%lld-%lld".printf (Util.datetime_to_ns (since), Util.datetime_to_ns (new GLib.DateTime.now ())));
            yield call.invoke_async (null);
            return this.process_weights_json (call.get_payload ());
        }

        public override async void import_data () throws GLib.Error {
            info ("Trying to import all data from Google Fit");

            yield this.retrieve_access_token ();
            var db = TrackerDatabase.get_instance ();
            yield db.import_data (yield this.get_all_steps (), yield this.get_all_weights (), null);
            this.settings.timestamp_last_sync_google_fit = new GLib.DateTime.now ();
        }

        public async override void open_authentication_url () throws GLib.Error {
            string scopes = "";
            foreach (var scope in GOOGLE_API_SCOPES) {
                scopes += Soup.URI.encode (scope, null);
                if (GOOGLE_API_SCOPES[GOOGLE_API_SCOPES.length - 1] != scope) {
                    scopes += "+";
                }
            }

            var params = new GLib.HashTable<string, string> (null, null);
            params.insert ("prompt", "consent");
            params.insert ("response_type", "code");

            yield GLib.AppInfo.launch_default_for_uri_async (this.build_login_url_full (this.redirect_url, params) + "&scope=%s".printf (scopes), null);

            var query_params = yield this.start_listen_server ();

            string? code = null;
            if (query_params == null || (code = this.lookup_token ((!) query_params, "code")) == null) {
                warning ("Failed to retrieve OAuth2 token!");
            } else {
                var proxy = new Rest.Proxy ("https://oauth2.googleapis.com/token", false);
                var call = proxy.new_call ();
                call.set_method ("POST");
                call.add_params (
                    "client_id", GOOGLE_CLIENT_ID,
                    "client_secret", GOOGLE_CLIENT_SECRET,
                    "code", (!) code,
                    "grant_type", "authorization_code",
                    "redirect_uri", this.redirect_url
                );
                yield call.invoke_async (null);

                var json = (!) ((!) Json.from_string (call.get_payload ())).get_object ();
                this.access_token = json.get_string_member ("access_token");
                yield this.set_refresh_token (json.get_string_member ("refresh_token"));
                this.settings.sync_provider_setup_google_fit = true;
            }
        }

        public override async void sync_data () throws GLib.Error {
            if ((yield this.get_refresh_token ()) == null) {
                throw new OAuth2Error.NO_LIBSECRET_PASSWORD ("Google Fit Refresh token not set up, won't sync.");
            }

            yield this.retrieve_access_token ();
            var db = TrackerDatabase.get_instance ();
            var since = this.settings.timestamp_last_sync_google_fit;
            yield db.import_data (yield this.get_steps_since (since), yield this.get_weights_since (since), null);
            this.settings.timestamp_last_sync_google_fit = new GLib.DateTime.now ();
        }

        protected override async void retrieve_access_token () throws GLib.Error {
            var proxy = new Rest.Proxy ("https://oauth2.googleapis.com/token", false);
            var call = proxy.new_call ();
            call.set_method ("POST");
            call.add_params (
                "client_id", GOOGLE_CLIENT_ID,
                "client_secret", GOOGLE_CLIENT_SECRET,
                "grant_type", "refresh_token",
                "refresh_token", yield this.get_refresh_token ()
            );
            yield call.invoke_async (null);
            var json = (!) ((!) Json.from_string (call.get_payload ())).get_object ();
            this.set_access_token ((!) json.get_string_member ("access_token"));
        }

        protected Gee.HashMap<string, uint32> process_steps_json (string json_string) throws GLib.Error {
            var json = (!) ((!) Json.from_string (json_string)).get_object ();

            var ret = new Gee.HashMap<string, uint32> ();
            foreach (var point in json.get_array_member ("point").get_elements ()) {
                var point_obj = (!) point.get_object ();
                var modified_time = int64.parse (point_obj.get_string_member ("modifiedTimeMillis"));
                var datetime = new GLib.DateTime.from_unix_utc (modified_time / 1000);
                uint32 step_count = 0;
                foreach (var value in point_obj.get_array_member ("value").get_elements ()) {
                    step_count += (uint32) ((!) value.get_object ()).get_int_member_with_default ("intVal", 0);
                }
                var date = Util.date_to_iso_8601 (Util.date_from_datetime (datetime));
                if (ret.has_key (date)) {
                    ret.set (date, ret.get (date) + step_count);
                } else {
                    ret.set (date, step_count);
                }
            }

            return ret;
        }

        protected Gee.HashMap<string, double?> process_weights_json (string json_string) throws GLib.Error {
            var json = (!) ((!) Json.from_string (json_string)).get_object ();
            var ret = new Gee.HashMap<string, double?> ();
            foreach (var point in ((!) json.get_array_member ("point")).get_elements ()) {
                var point_obj = (!) point.get_object ();
                var datetime = new GLib.DateTime.from_unix_utc (int64.parse (point_obj.get_string_member ("modifiedTimeMillis")) / 1000);
                var weight_value = ((!) point_obj.get_array_member ("value").get_elements ().last ().data.get_object ()).get_double_member_with_default ("fpVal", 0.0);
                if (weight_value == 0) {
                    continue;
                }
                var date = Util.date_to_iso_8601 (Util.date_from_datetime (datetime));
                ret.set (date, weight_value);
            }

            return ret;
        }
    }
}

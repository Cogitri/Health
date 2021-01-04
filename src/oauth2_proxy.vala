/* oauth2_proxy.vala
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

    errordomain OAuth2Error {
        BAD_PARAMS,
        SERVER_LISTEN_FAILED,
        NO_LIBSECRET_PASSWORD,
    }

    public abstract class OAuth2Proxy : Rest.OAuth2Proxy {
        public string get_redirect_url () {
            return "http://127.0.0.1:%u".printf (this.get_server_port ());
        }

        public abstract uint get_server_port ();

        public async string? get_refresh_token_from_libsecret (string oauth2_provider) {
            var schema = new Secret.Schema (Config.APPLICATION_ID, Secret.SchemaFlags.NONE, "oauth2-provider", Secret.SchemaAttributeType.STRING);

            string? token = null;
            try {
                token = yield Secret.password_lookup (schema, null, "oauth2-provider", oauth2_provider);
            } catch (GLib.Error e) {
                warning ("Failed to retrieve OAuth2 Refresh-Token from libsecret due to error %s. Falling back to regular authentication.", e.message);
                return null;
            }

            return token;
        }

        public async abstract void open_authentication_url (Settings settings) throws GLib.Error;

        public string? get_parameter_from_query (string url, string parameter) {
            string? token = null;
            var uri = new Soup.URI (url);
            if (uri.query != null) {
                var params = Soup.Form.decode (uri.query);
                if (params != null) {
                    var encoded_token = params.lookup (parameter);
                    if (encoded_token != null) {
                        token = Soup.URI.decode (encoded_token);
                    }
                }
            }
            return token;
        }

        public async void store_refresh_stoken (string refresh_token, string oauth2_provider) {
            var schema = new Secret.Schema (Config.APPLICATION_ID, Secret.SchemaFlags.NONE, "oauth2-provider", Secret.SchemaAttributeType.STRING);

            try {
                yield Secret.password_store (schema, Secret.COLLECTION_DEFAULT, "Health %s OAuth2-Refresh-Token".printf (oauth2_provider), refresh_token, null, "oauth2-provider", oauth2_provider);
            } catch (GLib.Error e) {
                warning ("Failed to store OAuth2 refresh-token via libsecret due to error %s", e.message);
            }
        }
    }

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
        public GoogleFitOAuth2Proxy () {
            Object (client_id: GOOGLE_CLIENT_ID, auth_endpoint: GOOGLE_AUTH_ENDPOINT_URL, url_format: GOOGLE_API_ENDPOINT);
        }

        public override uint get_server_port () {
            return 34981;
        }

        public async Gee.HashMap<string, double?> get_all_weights () throws GLib.Error {
            var call = this.new_call ();
            call.set_function ("users/me/dataSources/derived:com.google.weight:com.google.android.gms:merge_weight/datasets/0-%lld".printf (GLib.get_real_time () * 1000));
            yield call.invoke_async (null);
            return this.process_weights_json (call.get_payload ());
        }

        public async Gee.HashMap<string, double?> get_weights_since (GLib.DateTime since) throws GLib.Error {
            var call = this.new_call ();
            call.set_function ("users/me/dataSources/derived:com.google.weight:com.google.android.gms:merge_weight/datasets/%lld-%lld".printf (since.to_unix () * 1000, GLib.get_real_time () * 1000));
            yield call.invoke_async (null);
            return this.process_weights_json (call.get_payload ());
        }

        public async Gee.HashMap<string, uint32> get_all_steps () throws GLib.Error {
            var call = this.new_call ();
            call.set_function ("users/me/dataSources/derived:com.google.step_count.delta:com.google.android.gms:merge_step_deltas/datasets/0-%lld".printf (GLib.get_real_time () * 1000));
            yield call.invoke_async (null);
            return this.process_steps_json (call.get_payload ());
        }

        public async Gee.HashMap<string, uint32> get_steps_since (GLib.DateTime since) throws GLib.Error {
            var call = this.new_call ();
            call.set_function ("users/me/dataSources/derived:com.google.step_count.delta:com.google.android.gms:merge_step_deltas/datasets/%lld-%lld".printf (since.to_unix () * 1000, GLib.get_real_time () * 1000));
            yield call.invoke_async (null);
            return this.process_steps_json (call.get_payload ());
        }

        private Gee.HashMap<string, double?> process_weights_json (string json_string) throws GLib.Error {
            var json = Json.from_string (json_string).get_object ();
            var ret = new Gee.HashMap<string, double?> ();
            foreach (var point in json.get_array_member ("point").get_elements ()) {
                var point_obj = point.get_object ();
                var datetime = new GLib.DateTime.from_unix_utc (int64.parse (point_obj.get_string_member ("modifiedTimeMillis")) / 1000);
                var weight_value = point_obj.get_array_member ("value").get_elements ().last ().data.get_object ().get_double_member_with_default ("fpVal", 0.0);
                if (weight_value == 0) {
                    continue;
                }
                var date = Util.date_to_iso_8601 (Util.date_from_datetime (datetime));
                if (ret.has_key (date)) {
                    ret.set (date, ret.get (date) + weight_value);
                } else {
                    ret.set (date, weight_value);
                }
            }

            return ret;
        }

        private Gee.HashMap<string, uint32> process_steps_json (string json_string) throws GLib.Error {
            var json = Json.from_string (json_string).get_object ();

            var ret = new Gee.HashMap<string, uint32> ();
            foreach (var point in json.get_array_member ("point").get_elements ()) {
                var point_obj = point.get_object ();
                var modified_time = int64.parse (point_obj.get_string_member ("modifiedTimeMillis"));
                var datetime = new GLib.DateTime.from_unix_utc (modified_time / 1000);
                uint32 step_count = 0;
                foreach (var value in point_obj.get_array_member ("value").get_elements ()) {
                    step_count += (uint32) value.get_object ().get_int_member_with_default ("intVal", 0);
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

        public async void import_data () throws GLib.Error {
            info ("Trying to import all data from Google Fit");

            var db = TrackerDatabase.get_instance ();
            yield db.import_data (yield this.get_all_steps (), yield this.get_all_weights (), null);
        }

        public async void import_data_since (GLib.DateTime since) throws GLib.Error {
            info ("Trying to import data since %s from Google Fit", since.format_iso8601 ());

            var db = TrackerDatabase.get_instance ();
            yield db.import_data (yield this.get_steps_since (since), yield this.get_weights_since (since), null);
        }

        private async void set_access_token_from_redirect_uri (string redirect_uri) throws GLib.Error {
            var proxy = new Rest.Proxy ("https://oauth2.googleapis.com/token", false);
            var call = proxy.new_call ();
            var token = this.get_parameter_from_query (redirect_uri, "code");
            call.set_method ("POST");
            call.add_params (
                "client_id", GOOGLE_CLIENT_ID,
                "client_secret", GOOGLE_CLIENT_SECRET,
                "code", token,
                "grant_type", "authorization_code",
                "redirect_uri", this.get_redirect_url ()
            );
            yield call.invoke_async (null);
            var json = Json.from_string (call.get_payload ()).get_object ();
            yield this.store_refresh_stoken (json.get_string_member ("refresh_token"), "GoogleFit");
            this.set_access_token (json.get_string_member ("access_token"));
        }

        private async bool set_access_token_from_libsecret () throws GLib.Error {
            var refresh_token = yield this.get_refresh_token_from_libsecret ("GoogleFit");
            if (refresh_token == null) {
                return false;
            }
            var proxy = new Rest.Proxy ("https://oauth2.googleapis.com/token", false);
            var call = proxy.new_call ();
            call.set_method ("POST");
            call.add_params (
                "client_id", GOOGLE_CLIENT_ID,
                "client_secret", GOOGLE_CLIENT_SECRET,
                "grant_type", "refresh_token",
                "refresh_token", refresh_token
            );
            yield call.invoke_async (null);
            var json = Json.from_string (call.get_payload ()).get_object ();
            // FIXME: Handle the refresh token being revoked
            this.set_access_token (json.get_string_member ("access_token"));

            return true;
        }

        public async void sync_data (Settings settings) throws GLib.Error {
            if (!yield this.set_access_token_from_libsecret ()) {
                throw new OAuth2Error.NO_LIBSECRET_PASSWORD ("Google Fit Refresh token not set up, won't sync.");
            }
            yield this.import_data_since (settings.timestamp_last_sync_google_fit);
            settings.timestamp_last_sync_google_fit = new GLib.DateTime.now ();
        }

        public async override void open_authentication_url (Settings settings) throws GLib.Error {
            if (settings.sync_provider_setup_google_fit && yield this.set_access_token_from_libsecret ()) {
                info ("Got token from libsecret; no need to open authentication dialog");
                return;
            }

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

            SourceFunc callback = this.open_authentication_url.callback;

            var listen_thread = new Thread<string?> ("oauth_listen_thread", () => {
                var context = new GLib.MainContext ();
                context.push_thread_default ();
                var loop = new MainLoop (context);
                var server = (Soup.Server) Object.new (typeof (Soup.Server));
                string? uri = null;

                server.add_handler (null, (server, msg, path, query, client) => {
                    uri = msg.get_uri ().to_string (false);
                    msg.set_status (200);
                    // FIXME: Make the HTML response nicer
                    msg.set_response ("text/html", Soup.MemoryUse.STATIC, "<html><head><title>Success.</title></head><body><h1>Successfully retrieved Authorization-Token, please return to GNOME Health.</html>".data);
                    loop.quit ();
                });
                try {
                    server.listen_local (this.get_server_port (), 0);
                    loop.run ();
                    server.disconnect ();
                    Idle.add ((owned) callback);
                    return uri;
                } catch (GLib.Error e) {
                    warning ("Failed to listen for OAuth2-Redirect due to error %s", e.message);
                    Idle.add ((owned) callback);
                    return null;
                }
            });


            yield GLib.AppInfo.launch_default_for_uri_async (this.build_login_url_full (this.get_redirect_url (), params) + "&scope=%s".printf (scopes), null);
            yield;
            var uri = listen_thread.join ();
                if (uri != null) {
                yield this.set_access_token_from_redirect_uri ((!) uri);
                settings.sync_provider_setup_google_fit = true;
            }
        }
    }

}

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
        public string redirect_url {
            owned get {
                return "http://127.0.0.1:%u".printf (this.server_port);
            }
        }

        public uint server_port {
            get {
                return 34981;
            }
        }

        public abstract string provider_name { get; }

        public async string? get_refresh_token () {
            var schema = new Secret.Schema (Config.APPLICATION_ID, Secret.SchemaFlags.NONE, "oauth2-provider", Secret.SchemaAttributeType.STRING);

            string? token = null;
            try {
                token = yield Secret.password_lookup (schema, null, "oauth2-provider", this.provider_name);
            } catch (GLib.Error e) {
                warning ("Failed to retrieve OAuth2 Refresh-Token from libsecret due to error %s. Falling back to regular authentication.", e.message);
                return null;
            }

            return token;
        }

        public async void set_refresh_token (string refresh_token) {
            var schema = new Secret.Schema (Config.APPLICATION_ID, Secret.SchemaFlags.NONE, "oauth2-provider", Secret.SchemaAttributeType.STRING);

            try {
                yield Secret.password_store (schema, Secret.COLLECTION_DEFAULT, "Health %s OAuth2-Refresh-Token".printf (this.provider_name), refresh_token, null, "oauth2-provider", this.provider_name);
            } catch (GLib.Error e) {
                warning ("Failed to store OAuth2 refresh-token via libsecret due to error %s", e.message);
            }
        }

        public async abstract void import_data () throws GLib.Error;

        public async abstract void open_authentication_url () throws GLib.Error;

        public async abstract void sync_data () throws GLib.Error;

        protected string? lookup_token (GLib.HashTable<weak string, weak string> query_params, string parameter) {
            string? encoded_value = query_params.lookup (parameter);
            if (encoded_value != null) {
                return Soup.URI.decode ((!) encoded_value);
            }

            return null;
        }

        protected async abstract void retrieve_access_token () throws GLib.Error;

        /**
         * Start a {@link Soup.Server} on localhost : {@link server_port}, waiting for the user to finish OAuth2 authentication.
         *
         * @return Returns the parameters the server got a request at as {@link GLib.HashTable}, where the key is the query param
         * name and the value is the query value. 
         */
        protected async GLib.HashTable<weak string, weak string>? start_listen_server () {
            SourceFunc callback = this.start_listen_server.callback;

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
                    msg.set_response ("text/html", Soup.MemoryUse.STATIC, "<html><head><title>Success.</title></head><body><h1>Successfully retrieved Authorization-Token, please return to Health.</html>".data);
                    loop.quit ();
                });
                try {
                    server.listen_local (this.server_port, 0);
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

            // Wait for server handler to schedule us again
            yield;

            var url = listen_thread.join ();
            if (url != null) {
                var uri = new Soup.URI ((!) url);
                unowned string? query = uri.query;
                if (query != null) {
                    return Soup.Form.decode ((!) query);
                }
            }

            return null;
        }
    }

}

/* sync_provider.rs
 *
 * Copyright 2020-2021 Rasmus Thomsen <oss@cogitri.dev>
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

use crate::{core::i18n, prelude::*};
use anyhow::Result;
use gtk::gio;
use i18n::i18n_f;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    url::Url,
    AuthorizationCode, CsrfToken, EmptyExtraTokenFields, RefreshToken, StandardTokenResponse,
    TokenResponse,
};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

/// [SyncProvider] is a trait that should be implemented by all 3rd party providers.
pub trait SyncProvider {
    /// Returns the URL to the API Endpoint
    fn api_url(&self) -> &'static str;

    /// Returns the name of the provider (which is used for storing it in the keyring).
    fn provider_name(&self) -> &'static str;

    /// Gets the OAuth2 token or reauthenticates with the refresh token if no token has been set yet.
    fn oauth2_token(
        &mut self,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>>;

    /// Perform the initial authentication. This should open the user's browser so they can
    /// authenticate with their provider.
    fn initial_authenticate(&mut self) -> Result<()>;

    /// Start the initial import. This should import all data from the sync provider to
    /// the Tracker DB.
    fn initial_import(&mut self) -> Result<()>;

    /// Exchange the refresh token we already stored for an access token.
    fn reauthenticate(&mut self) -> Result<()>;

    /// This should sync data that has been added since the last sync.
    fn sync_data(&mut self) -> Result<()>;

    /// Make a `GET` request against the specified `method`.
    ///
    /// # Arguments
    /// * `method` - The API method to query
    ///
    /// # Returns
    /// The deserialized JSON response
    fn get<T: serde::de::DeserializeOwned>(&mut self, method: &str) -> Result<T> {
        Ok(ureq::get(&format!("{}/{method}", self.api_url()))
            .set(
                "Authorization",
                &format!("Bearer {}", self.oauth2_token()?.access_token().secret()),
            )
            .call()?
            .into_json()?)
    }

    /// Make a `POST` request against the specified `method`.
    ///
    /// # Arguments
    /// * `method` - The API method to query
    ///
    /// # Returns
    /// The deserialized JSON response
    fn post<T: serde::de::DeserializeOwned>(
        &mut self,
        method: &str,
        data: ureq::serde_json::Value,
    ) -> Result<T> {
        Ok(ureq::post(&format!("{}/{method}", self.api_url()))
            .set(
                "Authorization",
                &format!("Bearer {}", self.oauth2_token()?.access_token().secret()),
            )
            .send_json(data)?
            .into_json()?)
    }

    /// Exchange a refresh token for an access token.
    fn exchange_refresh_token(
        &self,
        client: &BasicClient,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        match self.token() {
            Ok(Some(token)) => Ok(client
                .exchange_refresh_token(&token)
                .request(oauth2::ureq::http_client)?),
            Ok(None) => {
                Err(anyhow::anyhow!("{}", i18n("Can’t retrieve OAuth2 token when no refresh token is set! Please re-authenticate with your sync provider.")))
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    /// Get the libsecret Schema used by Health
    fn schema(&self) -> libsecret::Schema {
        libsecret::Schema::new(
            crate::config::APPLICATION_ID,
            libsecret::SchemaFlags::NONE,
            [("provider", libsecret::SchemaAttributeType::String)]
                .into_iter()
                .collect(),
        )
    }

    /// Retrieve the [RefreshToken] from the secret store.
    ///
    /// # Returns
    /// A `RefreshToken` if a refresh token is set, or `None` if no refresh token is set.
    /// May return an error if querying the secret store fails.
    fn token(&self) -> Result<Option<RefreshToken>> {
        if let Some(password) = libsecret::password_lookup_sync(
            Some(&self.schema()),
            [("provider", self.provider_name())].into_iter().collect(),
            None::<&gio::Cancellable>,
        )? {
            Ok(Some(RefreshToken::new(password.to_string())))
        } else {
            Ok(None)
        }
    }

    /// Set the [RefreshToken] in the secret store.
    ///
    /// # Arguments
    /// * `value` - The [RefreshToken] that should be stored.
    ///
    /// # Returns
    /// May return an error if querying the secret store fails.
    fn set_token(&self, value: RefreshToken) -> Result<()> {
        libsecret::password_store_sync(
            Some(&self.schema()),
            [("provider", self.provider_name())].into_iter().collect(),
            Some(libsecret::COLLECTION_DEFAULT),
            &i18n_f("Token for Health sync provider {}", &[self.provider_name()]),
            value.secret(),
            None::<&gio::Cancellable>,
        )?;

        Ok(())
    }

    /// Starts a server which listens for the user to finish authenticating with their OAuth2 provider
    /// and captures the OAuth2 `code` once the user is redirect to the server.
    ///
    /// # Arguments
    /// * `authorize_url` - The URL which should be opened in the user's browser so they can authenticate.
    ///
    /// # Returns
    /// The [AuthorizationCode] and [CsrfToken] that were returned by the sync provider, or an error if
    /// reading the response fails. Please keep in mind that the returned `CrfsToken` should always be compared
    /// to what you sent to the provider to make sure the request went through fine.
    #[allow(clippy::manual_flatten)]
    fn start_listen_server(authorize_url: &str) -> Result<(AuthorizationCode, CsrfToken)> {
        gio::AppInfo::launch_default_for_uri_future(authorize_url, None::<&gio::AppLaunchContext>)
            .block()?;

        let listener = TcpListener::bind("127.0.0.1:34981")?;
        for s in listener.incoming() {
            if let Ok(mut stream) = s {
                let code;
                let state;
                {
                    let mut reader = BufReader::new(&stream);

                    let mut request_line = String::new();
                    reader.read_line(&mut request_line)?;

                    let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                    let url = Url::parse(&("http://localhost".to_string() + redirect_url))?;

                    let code_pair = url
                        .query_pairs()
                        .find(|pair| {
                            let (key, _) = pair;
                            key == "code"
                        })
                        .unwrap();

                    let (_, value) = code_pair;
                    code = AuthorizationCode::new(value.into_owned());

                    let state_pair = url
                        .query_pairs()
                        .find(|pair| {
                            let (key, _) = pair;
                            key == "state"
                        })
                        .unwrap();

                    let (_, value) = state_pair;
                    state = CsrfToken::new(value.into_owned());
                }

                let message = i18n("Successfully authenticated, please return to Health.");
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{message}",
                    message.len(),
                );
                stream.write_all(response.as_bytes())?;

                return Ok((code, state));
            }
        }

        anyhow::bail!("{}", i18n("Couldn’t parse OAuth2 response"))
    }
}

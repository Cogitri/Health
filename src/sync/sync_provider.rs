use crate::core::i18n;
use oauth2::{
    basic::{BasicErrorResponseType, BasicTokenType},
    url::Url,
    AuthorizationCode, Client, CsrfToken, EmptyExtraTokenFields, RefreshToken,
    StandardErrorResponse, StandardTokenResponse, TokenResponse,
};
use secret_service::{Collection, EncryptionType, Error as SsError, SecretService};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

#[derive(Debug, thiserror::Error)]
#[error("{}", _0)]
pub enum SyncProviderError {
    CrsfMismatch(String),
    GLib(#[from] glib::Error),
    IO(#[from] std::io::Error),
    NoRefreshTokenSet(String),
    NoRequestReceived(String),
    RefreshFailed(String),
    RequestToken(String),
    SecretService(#[from] SsError),
    UReq(Box<ureq::Error>),
    UrlParse(#[from] oauth2::url::ParseError),
}

// Box ureq::Error since it's pretty huge (1008 Byte)
impl From<ureq::Error> for SyncProviderError {
    fn from(e: ureq::Error) -> Self {
        Self::UReq(Box::new(e))
    }
}

pub trait SyncProvider {
    fn get_api_url(&self) -> &'static str;

    fn get_provider_name(&self) -> &'static str;

    fn get_oauth2_token(
        &mut self,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, SyncProviderError>;

    fn initial_authenticate(&mut self) -> Result<(), SyncProviderError>;

    fn initial_import(&mut self) -> Result<(), SyncProviderError>;

    fn reauthenticate(&mut self) -> Result<(), SyncProviderError>;

    fn sync_data(&mut self) -> Result<(), SyncProviderError>;

    fn get<T: serde::de::DeserializeOwned>(
        &mut self,
        method: &str,
    ) -> Result<T, SyncProviderError> {
        Ok(ureq::get(&format!("{}/{}", self.get_api_url(), method))
            .set(
                "Authorization",
                &format!(
                    "Bearer {}",
                    self.get_oauth2_token()?.access_token().secret()
                ),
            )
            .call()?
            .into_json()?)
    }

    fn post<T: serde::de::DeserializeOwned>(
        &mut self,
        method: &str,
        data: ureq::SerdeValue,
    ) -> Result<T, SyncProviderError> {
        Ok(ureq::post(&format!("{}/{}", self.get_api_url(), method))
            .set(
                "Authorization",
                &format!(
                    "Bearer {}",
                    self.get_oauth2_token()?.access_token().secret()
                ),
            )
            .send_json(data)?
            .into_json()?)
    }

    fn exchange_refresh_token(
        &self,
        client: &Client<
            StandardErrorResponse<BasicErrorResponseType>,
            StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
            BasicTokenType,
        >,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, SyncProviderError>
    {
        match self.get_token() {
            Ok(Some(token)) => client
                .exchange_refresh_token(&token)
                .request(super::ureq_http_client::http_client)
                .map_err(|e| SyncProviderError::RefreshFailed(e.to_string())),
            Ok(None) => {
                Err(SyncProviderError::NoRefreshTokenSet(i18n("Can't retrieve OAuth2 token when no refesh token is set! Please re-authenticate with your sync provider.")))
            }
            Err(e) => {
                Err(e.into())
            }
        }
    }

    fn get_token(&self) -> Result<Option<RefreshToken>, SsError> {
        let ss = SecretService::new(EncryptionType::Dh)?;
        let collection = get_default_collection_unlocked(&ss)?;

        if let Some(password) = collection
            .get_all_items()?
            .iter()
            .find(|p| p.get_label().unwrap_or_default() == self.get_provider_name())
        {
            password.unlock()?;

            Ok(Some(RefreshToken::new(
                String::from_utf8(password.get_secret()?).map_err(|_| SsError::NoResult)?,
            )))
        } else {
            Ok(None)
        }
    }

    fn set_token(&self, value: RefreshToken) -> Result<(), SsError> {
        let ss = SecretService::new(EncryptionType::Dh)?;
        let collection = get_default_collection_unlocked(&ss)?;

        // Delete old entries
        for p in collection
            .get_all_items()?
            .iter()
            .filter(|p| p.get_label().unwrap_or_default() == self.get_provider_name())
        {
            p.unlock()?;
            p.delete()?;
        }

        collection.create_item(
            self.get_provider_name(),
            std::collections::HashMap::new(),
            value.secret().as_bytes(),
            true,
            "text/plain",
        )?;

        Ok(())
    }

    fn start_listen_server(
        authorize_url: &str,
    ) -> Result<(AuthorizationCode, CsrfToken), SyncProviderError> {
        gio::AppInfo::launch_default_for_uri(authorize_url, None::<&gio::AppLaunchContext>)?;

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
                            let &(ref key, _) = pair;
                            key == "code"
                        })
                        .unwrap();

                    let (_, value) = code_pair;
                    code = AuthorizationCode::new(value.into_owned());

                    let state_pair = url
                        .query_pairs()
                        .find(|pair| {
                            let &(ref key, _) = pair;
                            key == "state"
                        })
                        .unwrap();

                    let (_, value) = state_pair;
                    state = CsrfToken::new(value.into_owned());
                }

                let message = i18n("Sucessfully authenticated, please return to Health.");
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                    message.len(),
                    message
                );
                stream.write_all(response.as_bytes())?;

                return Ok((code, state));
            }
        }

        Err(SyncProviderError::NoRequestReceived(i18n(
            "Couldn't parse OAuth2 response",
        )))
    }
}

fn get_default_collection_unlocked<'a>(ss: &'a SecretService) -> Result<Collection<'a>, SsError> {
    let collection = match ss.get_default_collection() {
        Ok(collection) => Ok(collection),
        Err(SsError::NoResult) => ss.create_collection("default", "default"),
        Err(x) => Err(x),
    };

    match collection {
        Ok(c) => {
            c.unlock()?;
            Ok(c)
        }
        Err(e) => Err(e),
    }
}
use super::sync_provider::{SyncProvider, SyncProviderError};
use crate::core::i18n_f;
use failure::Fail;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    AuthUrl, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields, PkceCodeChallenge,
    RedirectUrl, Scope, StandardTokenResponse, TokenResponse, TokenUrl,
};

static GOOGLE_API_ENDPOINT: &str = "https://www.googleapis.com/fitness/v1/";
static GOOGLE_API_KEY: &str = "AIzaSyAefLTWEhVRHI4zwtLQ1w8szeP-V3wz8jg";
static GOOGLE_AUTH_ENDPOINT_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
static GOOGLE_TOKEN_ENDPOINT_URL: &str = "https://www.googleapis.com/oauth2/v3/token";
static GOOGLE_CLIENT_SECRET: &str = "QXYmZ982XWCdwKTW8mI3BbPp";
static GOOGLE_CLIENT_ID: &str =
    "652904425115-cdqjiporv9klugv9m7m0tpu44jt6cacb.apps.googleusercontent.com";

#[derive(Debug, Clone)]
pub struct GoogleFitSyncProvider {
    token: Option<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>>,
}

impl SyncProvider for GoogleFitSyncProvider {
    fn get_provider_name(&self) -> &'static str {
        "google-fit"
    }

    fn initial_authenticate(&mut self) -> Result<(), SyncProviderError> {
        let client = BasicClient::new(
            ClientId::new(GOOGLE_CLIENT_ID.to_string()),
            Some(ClientSecret::new(GOOGLE_CLIENT_SECRET.to_string())),
            AuthUrl::new(GOOGLE_AUTH_ENDPOINT_URL.to_string()).unwrap(),
            Some(TokenUrl::new(GOOGLE_TOKEN_ENDPOINT_URL.to_string()).unwrap()),
        )
        .set_redirect_url(RedirectUrl::new("http://localhost:34981".to_string()).unwrap());
        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
        let (authorize_url, csrf_state) = client
            .authorize_url(CsrfToken::new_random)
            // This example is requesting access to the "calendar" features and the user's profile.
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/fitness.activity.write".to_string(),
            ))
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/fitness.body.write".to_string(),
            ))
            .set_pkce_challenge(pkce_code_challenge)
            .url();

        let (code, returned_crfst_state) =
            GoogleFitSyncProvider::start_listen_server(authorize_url.as_str())?;

        if csrf_state.secret() != returned_crfst_state.secret() {
            return Err(SyncProviderError::CrsfMismatch(i18n_f(
                "CRSF Verification failed, got {}, expected {}",
                &[returned_crfst_state.secret(), csrf_state.secret()],
            )));
        }

        // Exchange the code with a token.
        self.token = Some(
            client
                .exchange_code(code)
                .set_pkce_verifier(pkce_code_verifier)
                .request(super::ureq_http_client::http_client)
                .map_err(|e| {
                    SyncProviderError::RequestToken(i18n_f(
                        "Requesting OAuth2 token failed due to error {}",
                        &[&e.cause().map(|e| e.to_string()).unwrap_or("".to_string())],
                    ))
                })?,
        );

        if let Some(refresh_token) = self.token.as_ref().unwrap().refresh_token() {
            self.set_token(refresh_token.clone())?;
        }

        Ok(())
    }
}

impl GoogleFitSyncProvider {
    pub fn new() -> Self {
        Self { token: None }
    }
}

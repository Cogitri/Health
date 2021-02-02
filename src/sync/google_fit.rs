use super::{
    sync_provider::{SyncProvider, SyncProviderError},
    DatabaseValue,
};
use crate::{
    core::{i18n_f, Settings},
    model::{Steps, Weight},
};
use chrono::{DateTime, FixedOffset, Utc};
use failure::Fail;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    AuthUrl, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields, PkceCodeChallenge,
    RedirectUrl, Scope, StandardTokenResponse, TokenResponse, TokenUrl,
};
use std::collections::HashMap;
use uom::si::{f32::Mass, mass::kilogram};

static GOOGLE_PROVIDER_NAME: &str = "google-fit";
static GOOGLE_API_ENDPOINT: &str = "https://www.googleapis.com/fitness/v1";
static GOOGLE_AUTH_ENDPOINT_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
static GOOGLE_TOKEN_ENDPOINT_URL: &str = "https://www.googleapis.com/oauth2/v3/token";
static GOOGLE_CLIENT_SECRET: &str = "QXYmZ982XWCdwKTW8mI3BbPp";
static GOOGLE_CLIENT_ID: &str =
    "652904425115-cdqjiporv9klugv9m7m0tpu44jt6cacb.apps.googleusercontent.com";

#[derive(serde::Deserialize)]
struct Value {
    #[serde(rename = "intVal")]
    pub int_val: Option<u32>,
    #[serde(rename = "fpVal")]
    pub fp_val: Option<f32>,
}

#[derive(serde::Deserialize)]
struct Point {
    #[serde(deserialize_with = "super::serialize::deserialize_modified_time_millis")]
    #[serde(rename = "modifiedTimeMillis")]
    pub date: DateTime<FixedOffset>,
    pub value: Vec<Value>,
}

#[derive(serde::Deserialize)]
struct Points {
    pub point: Vec<Point>,
}

#[derive(Debug, Clone)]
pub struct GoogleFitSyncProvider {
    sender: glib::Sender<DatabaseValue>,
    token: Option<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>>,
}

impl GoogleFitSyncProvider {
    // False-Positive in contains_key, we can't use .entry().insert_or() since we need the else condition
    #[allow(clippy::map_entry)]
    fn get_steps(
        &mut self,
        date_opt: Option<DateTime<FixedOffset>>,
    ) -> Result<Vec<Steps>, SyncProviderError> {
        let points = if let Some(date) = date_opt {
            self.get::<Points>(&format!("users/me/dataSources/derived:com.google.step_count.delta:com.google.android.gms:merge_step_deltas/datasets/{}-{}", date.timestamp_nanos(), Utc::now().timestamp_nanos()))
        } else {
            self.get::<Points>(&format!("users/me/dataSources/derived:com.google.step_count.delta:com.google.android.gms:merge_step_deltas/datasets/0-{}", Utc::now().timestamp_nanos()))
        }?;

        let mut steps_map = HashMap::<DateTime<FixedOffset>, u32>::new();
        for s in points.point {
            if steps_map.contains_key(&s.date) {
                steps_map.insert(
                    s.date,
                    steps_map.get(&s.date).unwrap()
                        + s.value.iter().map(|s| s.int_val.unwrap()).sum::<u32>(),
                );
            } else {
                steps_map.insert(s.date, s.value.iter().map(|s| s.int_val.unwrap()).sum());
            }
        }

        Ok(steps_map
            .into_iter()
            .map(|(date, value)| Steps::new(date, value))
            .collect())
    }

    fn get_weights(
        &mut self,
        date_opt: Option<DateTime<FixedOffset>>,
    ) -> Result<Vec<Weight>, SyncProviderError> {
        let points = if let Some(date) = date_opt {
            self.get::<Points>(&format!("users/me/dataSources/derived:com.google.weight:com.google.android.gms:merge_weight/datasets/{}-{}", date.timestamp_nanos(), Utc::now().timestamp_nanos()))
        } else {
            self.get::<Points>(&format!("users/me/dataSources/derived:com.google.weight:com.google.android.gms:merge_weight/datasets/0-{}", Utc::now().timestamp_nanos()))
        }?;

        Ok(points
            .point
            .into_iter()
            .map(|p| {
                Weight::new(
                    p.date,
                    Mass::new::<kilogram>(p.value.last().unwrap().fp_val.unwrap_or(0.0)),
                )
            })
            .collect())
    }
}

impl SyncProvider for GoogleFitSyncProvider {
    fn get_provider_name(&self) -> &'static str {
        GOOGLE_PROVIDER_NAME
    }

    fn get_api_url(&self) -> &'static str {
        GOOGLE_API_ENDPOINT
    }

    fn get_oauth2_token(
        &mut self,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, SyncProviderError>
    {
        if let Some(token) = &self.token {
            Ok(token.clone())
        } else {
            self.reauthenticate()?;
            Ok(self.token.clone().unwrap())
        }
    }

    fn reauthenticate(&mut self) -> Result<(), SyncProviderError> {
        let client = BasicClient::new(
            ClientId::new(GOOGLE_CLIENT_ID.to_string()),
            Some(ClientSecret::new(GOOGLE_CLIENT_SECRET.to_string())),
            AuthUrl::new(GOOGLE_AUTH_ENDPOINT_URL.to_string()).unwrap(),
            Some(TokenUrl::new(GOOGLE_TOKEN_ENDPOINT_URL.to_string()).unwrap()),
        );
        self.token = Some(self.exchange_refresh_token(&client)?);
        Ok(())
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
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/fitness.activity.read".to_string(),
            ))
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/fitness.activity.write".to_string(),
            ))
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/fitness.body.read".to_string(),
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
                        &[&e.cause()
                            .map_or(String::new(), std::string::ToString::to_string)],
                    ))
                })?,
        );

        if let Some(refresh_token) = self.token.as_ref().unwrap().refresh_token() {
            self.set_token(refresh_token.clone())?;
            let settings = Settings::new();
            settings.set_sync_provider_setup_google_fit(true);
            settings.set_timestamp_last_sync_google_fit(chrono::Local::now().into());
        }

        Ok(())
    }

    fn initial_import(&mut self) -> Result<(), SyncProviderError> {
        let steps = self.get_steps(None)?;
        self.sender.send(DatabaseValue::Steps(steps)).unwrap();

        let weights = self.get_weights(None)?;
        self.sender.send(DatabaseValue::Weights(weights)).unwrap();

        Ok(())
    }

    fn sync_data(&mut self) -> Result<(), SyncProviderError> {
        let settings = Settings::new();
        let last_sync_date = settings.get_timestamp_last_sync_google_fit();
        settings.set_timestamp_last_sync_google_fit(chrono::Local::now().into());

        let steps = self.get_steps(Some(last_sync_date))?;
        self.sender.send(DatabaseValue::Steps(steps)).unwrap();

        let weights = self.get_weights(Some(last_sync_date))?;
        self.sender.send(DatabaseValue::Weights(weights)).unwrap();

        Ok(())
    }
}

impl GoogleFitSyncProvider {
    pub fn new(sender: glib::Sender<DatabaseValue>) -> Self {
        Self {
            sender,
            token: None,
        }
    }
}

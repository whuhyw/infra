use chrono::Utc;
use md5::{Digest, Md5};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::models::{SignInData, WhuApiResponse};
use crate::{Error, Result, WhuApiOptions};

use crate::{crypto::Sm2CryptoService, transport::WhuApiTransport};

#[derive(Serialize)]
struct SignInQuery<'a> {
    p1: &'a str,
    p2: &'a str,
    p3: &'a str,
    t: i64,
    s: String,
}

struct TokenState {
    access_token: Option<String>,
    expires_at: i64,
}

pub(crate) struct TokenService {
    options: WhuApiOptions,
    crypto: Sm2CryptoService,
    transport: WhuApiTransport,
    state: Mutex<TokenState>,
}

impl TokenService {
    pub(crate) fn new(options: WhuApiOptions, transport: WhuApiTransport) -> Result<Self> {
        let crypto = Sm2CryptoService::new(&options.sm2_public_key)?;
        Ok(Self {
            options,
            crypto,
            transport,
            state: Mutex::new(TokenState {
                access_token: None,
                expires_at: i64::MIN,
            }),
        })
    }

    pub(crate) async fn get_access_token(&self) -> Result<String> {
        let mut state = self.state.lock().await;
        let now = Utc::now().timestamp();
        if let Some(token) = &state.access_token
            && now < state.expires_at.saturating_sub(600)
        {
            return Ok(token.clone());
        }

        let timestamp = now;
        let password_hash = format!("{:X}", Md5::digest(self.options.account_pass.as_bytes()));
        let plaintext = format!(
            "p1={}&p2={}&p3={}&t={timestamp}",
            self.options.account_id, password_hash, self.options.sys_id
        );
        let query = SignInQuery {
            p1: &self.options.account_id,
            p2: &password_hash,
            p3: &self.options.sys_id,
            t: timestamp,
            s: self.crypto.encrypt(&plaintext)?,
        };
        let url = self.transport.url("/v3/XINTFLg/SpecialSignIn")?;
        let wrapper: WhuApiResponse<SignInData> =
            self.transport.get_raw(url_with_query(url, &query)).await?;

        let data = match wrapper.data {
            Some(data) if wrapper.code == 0 && data.result == 0 => data,
            Some(data) => {
                let message = if data.message.is_empty() {
                    wrapper.message
                } else {
                    data.message
                };
                return Err(Error::SignIn(message));
            }
            None => return Err(Error::SignIn(wrapper.message)),
        };
        state.expires_at = jwt_expiry(&data.access_jwt).unwrap_or(i64::MIN);
        state.access_token = Some(data.access_jwt.clone());
        Ok(data.access_jwt)
    }
}

fn url_with_query(mut url: reqwest::Url, query: &SignInQuery<'_>) -> reqwest::Url {
    url.query_pairs_mut()
        .append_pair("p1", query.p1)
        .append_pair("p2", query.p2)
        .append_pair("p3", query.p3)
        .append_pair("t", &query.t.to_string())
        .append_pair("s", &query.s);
    url
}

fn jwt_expiry(token: &str) -> Option<i64> {
    let payload = token.split('.').nth(1)?;
    let decoded =
        base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, payload)
            .or_else(|_| {
                base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE, payload)
            })
            .ok()?;
    serde_json::from_slice::<serde_json::Value>(&decoded).ok()?["exp"].as_i64()
}

#[cfg(test)]
mod tests {
    use super::jwt_expiry;

    #[test]
    fn reads_expiration_from_jwt_payload() {
        assert_eq!(
            jwt_expiry("header.eyJleHAiOjE3MDAwMDAwMDB9.sig"),
            Some(1_700_000_000)
        );
    }
}

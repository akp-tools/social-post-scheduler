use crate::app_environment;

use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
    DecodingKey, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::{Entry, HashMap};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

const CACHE_TTL: Duration = Duration::from_secs(300);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CfAccessJwt {
    pub iss: String,
    pub sub: String,
    pub aud: Vec<String>,
    pub exp: u64,
    pub nbf: u64,
    pub iat: u64,
    pub email: String,
    pub country: String,
}

pub type JwkCache = Arc<Mutex<HashMap<String, (Instant, JwkSet)>>>;

pub fn init_cache() -> JwkCache {
    Arc::new(Mutex::new(HashMap::new()))
}

async fn get_public_keys_cached(cache: &JwkCache) -> Result<JwkSet, Box<dyn std::error::Error>> {
    let mut cache = cache.lock().await;
    let now = Instant::now();
    let jwks = match cache.entry("cf_access_certs".to_string()) {
        Entry::Vacant(entry) => {
            let jwks = get_public_keys().await?;
            entry.insert((now, jwks.to_owned()));
            jwks
        }
        Entry::Occupied(mut entry) => {
            let (ts, jwks) = entry.get();
            if ts.elapsed() < CACHE_TTL {
                jwks.to_owned()
            } else {
                debug!("JWK Set cache has expired");
                let jwks = get_public_keys().await?;
                entry.insert((now, jwks.to_owned()));
                jwks
            }
        }
    };
    Ok(jwks)
}

async fn get_public_keys() -> Result<JwkSet, String> {
    let env = app_environment::get().expect("Environment failed!?");

    // Name your user agent after your app?
    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&env.cf_access_certs_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response.json::<JwkSet>().await.map_err(|e| e.to_string())
}

pub async fn decode_jwt(token: &str) -> Result<TokenData<CfAccessJwt>, String> {
    let env = app_environment::get().expect("Environment failed!?");

    let jwks = match get_public_keys_cached(&env.jwk_cache).await {
        Ok(jwks) => jwks,
        Err(e) => return Err(e.to_string()),
    };

    let header = decode_header(token).expect("Error decoding JWT token header!");
    let kid = match header.kid {
        Some(k) => k,
        None => return Err("JWT has no `kid` header field!".to_string()),
    };

    if let Some(j) = jwks.find(&kid) {
        Ok(match j.algorithm {
            AlgorithmParameters::RSA(ref rsa) => {
                let decoding_key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e).unwrap();
                let mut validation = Validation::new(j.common.algorithm.unwrap());
                validation.leeway = 30;
                validation.set_audience(&[&env.cf_access_aud]);

                match decode::<CfAccessJwt>(token, &decoding_key, &validation) {
                    Ok(claims) => claims,
                    Err(e) => return Err(format!("Error validating JWT! {}", e)),
                }
            }
            _ => unreachable!("JWT algorithm should be RSA!"),
        })
    } else {
        Err("Couldn't find a key to decode the token!".to_string())
    }
}

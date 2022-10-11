use crate::app_environment;

use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
    DecodingKey, TokenData, Validation,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CfAccessJwt {
    pub iss: Option<String>,
    pub sub: Option<String>,
    pub aud: Vec<String>,
    pub exp: Option<i64>,
    pub nbf: Option<i64>,
    pub iat: Option<i64>,
    pub email: String,
    pub country: String,
}

pub async fn get_public_keys() -> Result<JwkSet, String> {
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

    let jwks = match get_public_keys().await {
        Ok(jwks) => jwks,
        Err(e) => return Err(e),
    };

    let header = decode_header(token).expect("Error decoding JWT token header!");
    let kid = match header.kid {
        Some(k) => k,
        None => return Err("Token has no `kid` header field!".to_string()),
    };

    if let Some(j) = jwks.find(&kid) {
        Ok(match j.algorithm {
            AlgorithmParameters::RSA(ref rsa) => {
                let decoding_key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e).unwrap();
                let mut validation = Validation::new(j.common.algorithm.unwrap());
                validation.validate_exp = false;

                let decoded = decode::<CfAccessJwt>(token, &decoding_key, &validation).unwrap();

                if !decoded.claims.aud.contains(&env.cf_access_aud) {
                    return Err("JWT doesn't contain the correct aud!".to_string());
                }

                decoded
            }
            _ => unreachable!("JWT algorithm should be RSA!"),
        })
    } else {
        Err("Couldn't find a key to decode the token!".to_string())
    }
}

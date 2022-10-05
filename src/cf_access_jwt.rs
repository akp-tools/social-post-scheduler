use crate::app_environment;

use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error, HttpMessage, HttpResponse,
};
use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
    DecodingKey, TokenData, Validation,
};
use serde::{Deserialize, Serialize};

use std::{
    future::{ready, Ready},
    rc::Rc,
};

use futures_util::future::LocalBoxFuture;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub aud: Vec<String>,
    pub email: String,
    pub iss: String,
    pub sub: String,
    pub country: String,
}

pub async fn get_public_keys() -> JwkSet {
    let env = app_environment::get().expect("Environment failed!?");

    let client = awc::Client::default();

    let request = client
        .get(&env.cf_access_certs_url)
        .insert_header((header::USER_AGENT, "AKPPostBufferer/1.0"));

    let mut response = request
        .send()
        .await
        .expect("Couldn't connect to CF Access Keys URL!");

    response
        .json::<JwkSet>()
        .await
        .expect("Failed to parse JwkSet!")
}

pub async fn decode_jwt(token: &str) -> Result<TokenData<Claims>, &str> {
    let env = app_environment::get().expect("Environment failed!?");

    let jwks = get_public_keys().await;

    let header = decode_header(token).expect("Error decoding JWT token header!");
    let kid = match header.kid {
        Some(k) => k,
        None => return Err("Token has no `kid` header field!"),
    };

    if let Some(j) = jwks.find(&kid) {
        Ok(match j.algorithm {
            AlgorithmParameters::RSA(ref rsa) => {
                let decoding_key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e).unwrap();
                let mut validation = Validation::new(j.common.algorithm.unwrap());
                validation.validate_exp = false;

                let decoded = decode::<Claims>(token, &decoding_key, &validation).unwrap();

                if !decoded.claims.aud.contains(&env.cf_access_aud) {
                    return Err("JWT doesn't contain the correct aud!");
                }

                decoded
            }
            _ => unreachable!("JWT algorithm should be RSA!"),
        })
    } else {
        Err("Couldn't find a key to decode the token!")
    }
}

pub struct CFAccessJWT;

impl<S, B> Transform<S, ServiceRequest> for CFAccessJWT
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = CFAccessJWTMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CFAccessJWTMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct CFAccessJWTMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CFAccessJWTMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, svc_request: ServiceRequest) -> Self::Future {
        let cf_access_jwt_assertion = match svc_request
            .headers()
            .to_owned()
            .get("cf-access-jwt-assertion")
        {
            Some(assertion) => assertion.to_str().unwrap(),
            None => {
                let (request, _pl) = svc_request.into_parts();
                let response = HttpResponse::InternalServerError()
                    .body("No CF Access JWT provided!")
                    .map_into_right_body();

                return Box::pin(async { Ok(ServiceResponse::new(request, response)) });
            }
        };

        let res = self.service.call(svc_request);

        Box::pin(async move {
            let token = decode_jwt(cf_access_jwt_assertion).await;

            svc_request.extensions_mut().insert(token.unwrap().claims);

            res.await.map(ServiceResponse::map_into_left_body)
        })
    }
}

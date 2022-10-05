mod models;

use crate::{app_environment::AppEnvironment, cf_access_jwt::Claims, facebook::models::*};

use actix::Addr;
use actix_redis::{Command, RedisActor};
use actix_web::{error, get, http::header, web, HttpResponse};
use rand::distributions::{Alphanumeric, DistString};
use redis_async::{resp::RespValue, resp_array};
use url::Url;

#[get("/api/v1/login/facebook")]
async fn facebook_login(
    env: web::Data<AppEnvironment>,
    redis: web::Data<Addr<RedisActor>>,
    claims: web::ReqData<Claims>,
) -> actix_web::Result<HttpResponse> {
    let claims = claims.into_inner();

    let redirect_url = match Url::parse(&env.base_url) {
        Ok(mut url) => {
            url.set_path("/api/v1/redirect/facebook");
            url.to_string()
        }
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let state = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    let redis_key = format!("fb_state+{}", &claims.email);

    let res = redis
        .send(Command(resp_array!["SET", redis_key, &state]))
        .await
        .map_err(error::ErrorInternalServerError)?
        .map_err(error::ErrorInternalServerError)?;

    match res {
        RespValue::SimpleString(x) if x == "OK" => true,
        _ => {
            log::error!("{res:?}");
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    let facebook_query_params = &[
        ("client_id", &env.facebook_client_id),
        ("redirect_uri", &redirect_url),
        ("state", &state),
        ("scope", &env.facebook_required_scopes),
    ];

    let facebook_base_url = match Url::parse_with_params(
        "https://www.facebook.com/v15.0/dialog/oauth",
        facebook_query_params,
    ) {
        Ok(url) => url.to_string(),
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    Ok(HttpResponse::TemporaryRedirect()
        .append_header((header::LOCATION, facebook_base_url))
        .finish())
}

#[get("/api/v1/redirect/facebook")]
async fn facebook_redirect(
    query: web::Query<FacebookRedirect>,
    env: web::Data<AppEnvironment>,
    redis: web::Data<Addr<RedisActor>>,
    claims: web::ReqData<Claims>,
) -> actix_web::Result<HttpResponse> {
    let claims = claims.into_inner();
    let redis_key = format!("fb_state+{}", &claims.email);

    let res = redis
        .send(Command(resp_array!["GET", redis_key]))
        .await
        .map_err(error::ErrorInternalServerError)?
        .map_err(error::ErrorInternalServerError)?;

    let expected_state = match res {
        RespValue::BulkString(x) => {
            String::from_utf8(x).map_err(error::ErrorInternalServerError)?
        }
        _ => {
            log::error!("{res:?}");
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    if query.state != expected_state {
        return Ok(HttpResponse::InternalServerError().finish());
    }

    let redirect_url = match Url::parse(&env.base_url) {
        Ok(mut url) => {
            url.set_path("/api/v1/redirect/facebook");
            url.to_string()
        }
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let facebook_query_params = &[
        ("client_id", &env.facebook_client_id),
        ("client_secret", &env.facebook_client_secret),
        ("redirect_uri", &redirect_url),
        ("code", &query.code.to_string()),
    ];

    let facebook_access_token_url = match Url::parse_with_params(
        "https://graph.facebook.com/v15.0/oauth/access_token",
        facebook_query_params,
    ) {
        Ok(url) => url.to_string(),
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let client = awc::Client::default();

    let request = client
        .get(facebook_access_token_url)
        .insert_header((header::USER_AGENT, "AKPPostBufferer/1.0"));

    let mut response = match request.send().await {
        Ok(response) => response,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let access_token = match response.json::<FacebookAccessToken>().await {
        Ok(token) => token,
        Err(_) => {
            return Ok(HttpResponse::TemporaryRedirect()
                .append_header((
                    header::LOCATION,
                    format!("{}/api/v1/login/facebook", &env.base_url),
                ))
                .finish())
        }
    };

    let result = match serde_json::to_string(&access_token) {
        Ok(json) => json,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let facebook_debug_token_query = &[
        ("input_token", access_token.access_token),
        (
            "access_token",
            format!(
                "{}|{}",
                &env.facebook_client_id, &env.facebook_client_secret
            ),
        ),
    ];

    let facebook_debug_token_url = match Url::parse_with_params(
        "https://graph.facebook.com/debug_token",
        facebook_debug_token_query,
    ) {
        Ok(url) => url.to_string(),
        Err(_) => return Ok(HttpResponse::InternalServerError().body("failed debug token url")),
    };

    let debug_request = client
        .get(facebook_debug_token_url)
        .insert_header((header::USER_AGENT, "AKPPostBufferer/1.0"));

    let mut debug_response = match debug_request.send().await {
        Ok(response) => response,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError()
                .body(format!("failed debug request send, {}", e)))
        }
    };

    let debug_info = serde_json::from_slice::<FacebookDebugTokenGraphContainer>(
        &debug_response.body().await.unwrap(),
    )
    .unwrap();

    // TODO: store all this info in the database
    Ok(HttpResponse::Ok()
        .append_header((header::CONTENT_TYPE, "application/json"))
        .body(result))
}

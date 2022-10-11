mod models;

use crate::{
    app_environment::AppEnvironment, facebook::models::*, fairings::db::RedisPool,
    jwt::CfAccessJwt, responders::location::LocationResponder,
};
use rand::distributions::{Alphanumeric, DistString};
use redis::AsyncCommands;
use rocket::{http::Status, serde::json::Json, State};
use rocket_db_pools::Connection;
use url::Url;

#[get("/api/v1/login/facebook")]
pub async fn facebook_login(
    env: &State<AppEnvironment>,
    mut redis: Connection<RedisPool>,
    claims: CfAccessJwt,
) -> Result<LocationResponder, Status> {
    let redirect_url = match Url::parse(&env.base_url) {
        Ok(mut url) => {
            url.set_path("/api/v1/redirect/facebook");
            url.to_string()
        }
        Err(_) => return Err(Status::InternalServerError),
    };

    let state = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    let redis_key = format!("fb_state+{}", &claims.email);

    match redis
        .set::<String, String, String>(redis_key, state.clone())
        .await
    {
        Ok(_) => (),
        Err(e) => {
            log::error!("{e:?}");
            return Err(Status::InternalServerError);
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
        Err(_) => return Err(Status::InternalServerError),
    };

    Ok(LocationResponder {
        location: facebook_base_url,
    })
}

#[get("/api/v1/redirect/facebook?<code>&<state>")]
pub async fn facebook_redirect(
    code: Option<String>,
    state: Option<String>,
    env: &State<AppEnvironment>,
    mut redis: Connection<RedisPool>,
    claims: CfAccessJwt,
    http_client: &State<reqwest::Client>,
) -> RedirectResponse<Json<TempResponse>> {
    let code = match code {
        Some(code) => code,
        _ => return RedirectResponse::Unauthorized(""),
    };

    let state = match state {
        Some(state) => state,
        _ => return RedirectResponse::Unauthorized(""),
    };

    let redis_key = format!("fb_state+{}", &claims.email);

    let expected_state: String = match redis.get(redis_key).await {
        Ok(data) => data,
        Err(e) => {
            log::error!("{e:?}");
            return RedirectResponse::InternalServerError("failed to get state");
        }
    };

    if state != expected_state {
        return RedirectResponse::Unauthorized("");
    }

    let redirect_url = match Url::parse(&env.base_url) {
        Ok(mut url) => {
            url.set_path("/api/v1/redirect/facebook");
            url.to_string()
        }
        Err(_) => return RedirectResponse::InternalServerError("failed to construct redirect_url"),
    };

    let facebook_query_params = &[
        ("client_id", &env.facebook_client_id),
        ("client_secret", &env.facebook_client_secret),
        ("redirect_uri", &redirect_url),
        ("code", &code),
    ];

    let facebook_access_token_url = match Url::parse_with_params(
        "https://graph.facebook.com/v15.0/oauth/access_token",
        facebook_query_params,
    ) {
        Ok(url) => url.to_string(),
        Err(_) => {
            return RedirectResponse::InternalServerError("failed to construct access_token_url")
        }
    };

    let access_token = match http_client
        .get(facebook_access_token_url)
        .send()
        .await
        .unwrap()
        .json::<FacebookAccessToken>()
        .await
    {
        Ok(token) => token,
        Err(_) => {
            return RedirectResponse::Redirect(LocationResponder {
                location: format!("{}/api/v1/login/facebook", &env.base_url),
            })
        }
    };

    let facebook_debug_token_query = &[
        ("input_token", access_token.access_token.clone()),
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
        Err(_) => return RedirectResponse::InternalServerError(""),
    };

    let debug_response = http_client
        .get(facebook_debug_token_url)
        .send()
        .await
        .unwrap();

    let debug_info = serde_json::from_str::<FacebookDebugTokenGraphContainer>(
        debug_response.text().await.unwrap().as_str(),
    )
    .unwrap();

    // TODO: store all this info in the database
    RedirectResponse::Ok(Json(TempResponse {
        access_token,
        debug_graph: debug_info,
    }))
}

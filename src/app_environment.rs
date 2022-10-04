use dotenv;
use std::env;

#[derive(Clone)]
pub struct AppEnvironment {
    pub base_url: String,
    pub facebook_client_id: String,
    pub facebook_client_secret: String,
    pub facebook_required_scopes: String,
    pub mongodb_url: String,
    pub redis_url: String,
    pub sentry_dsn: String,
    pub cf_access_aud: String,
    pub cf_access_team: String,
    pub cf_access_certs_url: String,
}

pub fn get() -> Result<AppEnvironment, &'static str> {
    dotenv::dotenv().ok();

    let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into());
    let facebook_client_id = env::var("FB_CLIENT_ID")
        .expect("No Facebook client ID provided in the FB_CLIENT_ID environment variable!");
    let facebook_client_secret = env::var("FB_CLIENT_SECRET")
        .expect("No Facebook client secret provided in the FB_CLIENT_SECRET environment variable!");
    let facebook_required_scopes = env::var("FB_REQUIRED_SCOPES").expect(
        "No Facebook required scopes provided in the FB_REQUIRED_SCOPES environment variable!",
    );
    let mongodb_url =
        env::var("MONGODB_URL").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost".into());
    let sentry_dsn = env::var("SENTRY_DSN").unwrap_or_else(|_| "".into());

    let cf_access_aud =
        env::var("CF_ACCESS_AUD").expect("Missing required variable CF_ACCESS_AUD!");
    let cf_access_team =
        env::var("CF_ACCESS_TEAM").expect("Missing required variable CF_ACCESS_TEAM!");
    let cf_access_certs_url = format!(
        "https://{}.cloudflareaccess.com/cdn-cgi/access/certs",
        cf_access_team
    );

    Ok(AppEnvironment {
        base_url,
        facebook_client_id,
        facebook_client_secret,
        facebook_required_scopes,
        mongodb_url,
        redis_url,
        sentry_dsn,
        cf_access_aud,
        cf_access_team,
        cf_access_certs_url,
    })
}

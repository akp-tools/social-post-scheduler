mod app_environment;
mod facebook;
mod fairings;
mod guards;
mod jwt;
mod responders;
mod test_endpoints;

#[macro_use]
extern crate rocket;

use rocket::{Build, Rocket};
use rocket_db_pools::Database;
use rocket_sentry::RocketSentry;

#[get("/")]
fn index() -> String {
    String::from("neat")
}

#[launch]
async fn rocket() -> Rocket<Build> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let env = match app_environment::get() {
        Ok(env) => env,
        Err(e) => panic!("{}", e),
    };

    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .map_err(|e| e.to_string())
        .expect("failed to create reqwest client!");

    rocket::build()
        .manage(env)
        .manage(client)
        .attach(crate::fairings::custom_headers::CustomHeaders)
        .attach(crate::fairings::db::RedisPool::init())
        .attach(crate::fairings::db::MongoDb::init())
        .attach(RocketSentry::fairing())
        .mount(
            "/",
            routes![
                index,
                test_endpoints::mongo_test,
                test_endpoints::cf_access_test,
                test_endpoints::redis_test,
                facebook::facebook_login,
                facebook::facebook_redirect,
            ],
        )
}

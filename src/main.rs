mod app_environment;
mod cf_access_jwt;
mod facebook;
mod test_endpoints;

use actix_web::{middleware, web, App, HttpServer};
use tokio;

type RedisPool = r2d2::Pool<redis::Client>;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let env = match app_environment::get() {
        Ok(env) => env,
        Err(e) => panic!("{}", e),
    };

    if &env.sentry_dsn.as_str().chars().count() > &0 {
        // Sentry setup
        let _guard = sentry::init((
            env.sentry_dsn.as_str(),
            sentry::ClientOptions {
                release: sentry::release_name!(),
                session_mode: sentry::SessionMode::Request,
                auto_session_tracking: true,
                ..Default::default()
            },
        ));
    }

    std::env::set_var("RUST_BACKTRACE", "1");

    let mongo_client = mongodb::Client::with_uri_str(&env.mongodb_url)
        .await
        .expect(&format!(
            "Failed to connect to MongoDB at {}",
            &env.mongodb_url
        ));
    let redis_client = redis::Client::open((&env.redis_url).to_string())
        .expect(&format!("Failed to connect to Redis at {}", &env.redis_url));

    let redis_connection_pool: RedisPool = r2d2::Pool::new(redis_client).unwrap();

    log::info!("starting HTTP server at 0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(env.to_owned()))
            .app_data(web::Data::new(mongo_client.clone()))
            .app_data(web::Data::new(redis_connection_pool.clone()))
            .wrap(sentry_actix::Sentry::new())
            .wrap(middleware::DefaultHeaders::new().add(("X-Powered-By", "rainbows and shit")))
            .wrap(cf_access_jwt::CFAccessJWT)
            // enable logging - always register logger middleware last
            .wrap(middleware::Logger::default())
            .service(test_endpoints::mongo_test)
            .service(test_endpoints::redis_test)
            .service(test_endpoints::cf_access_test)
            .service(facebook::facebook_login)
            .service(facebook::facebook_redirect)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

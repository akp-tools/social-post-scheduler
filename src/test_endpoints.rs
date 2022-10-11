use crate::{
    fairings::db::MongoDb, fairings::db::RedisPool, guards::headers::Headers, jwt::CfAccessJwt,
};

use redis::AsyncCommands;
use rocket::http::Status;
use rocket_db_pools::Connection;

#[get("/mongo-test")]
pub async fn mongo_test(mongo: Connection<MongoDb>) -> Result<String, Status> {
    let db_names = match mongo.list_database_names(None, None).await {
        Ok(db_names) => db_names,
        Err(_) => return Err(Status::InternalServerError),
    };

    Ok(db_names.join(", "))
}

#[get("/redis-test")]
pub async fn redis_test(mut redis: Connection<RedisPool>) -> Result<String, Status> {
    match redis.incr::<&str, _, i32>("test", 1).await {
        Ok(x) => Ok(x.to_string()),
        Err(e) => {
            log::error!("{e:?}");
            Err(Status::InternalServerError)
        }
    }
}

#[get("/cf-access-test")]
pub async fn cf_access_test(headers: Headers<'_>, claims: CfAccessJwt) -> String {
    let mut header_str = String::new();

    for header in headers.iter() {
        header_str += format!("{}: {:?}\n", header.name(), header.value()).as_str();
    }

    format!("{}\n\n\n{:#?}", header_str, claims)
}

use crate::{cf_access_jwt::Claims, RedisPool};

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use mongodb::Client;
use redis::Commands;

#[get("/mongo-test")]
async fn mongo_test(client: web::Data<Client>) -> impl Responder {
    let db_names = match client.list_database_names(None, None).await {
        Ok(db_names) => db_names,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().body(db_names.join(", "))
}

#[get("/redis-test")]
async fn redis_test(redis: web::Data<RedisPool>) -> impl Responder {
    let data: i32 = match redis.get().expect("oops").incr("test", 1) {
        Ok(data) => data,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().body(data.to_string())
}

#[get("/cf-access-test")]
async fn cf_access_test(req: HttpRequest, claims: web::ReqData<Claims>) -> impl Responder {
    let mut headers = "".to_string();

    for (key, val) in req.headers().iter() {
        headers += format!("{}: {:?}\n", key, val).as_str();
    }

    let claims = claims.into_inner();

    HttpResponse::Ok().body(format!("{}\n\n\n{:#?}", headers, claims))
}

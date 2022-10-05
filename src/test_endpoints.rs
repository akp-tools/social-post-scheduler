use crate::cf_access_jwt::Claims;

use actix::Addr;
use actix_redis::{Command, RedisActor};
use actix_web::{error, get, web, HttpRequest, HttpResponse, Responder};
use mongodb::Client;
use redis_async::{resp::RespValue, resp_array};

#[get("/mongo-test")]
async fn mongo_test(client: web::Data<Client>) -> impl Responder {
    let db_names = match client.list_database_names(None, None).await {
        Ok(db_names) => db_names,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().body(db_names.join(", "))
}

#[get("/redis-test")]
async fn redis_test(redis: web::Data<Addr<RedisActor>>) -> actix_web::Result<HttpResponse> {
    let res = redis
        .send(Command(resp_array!["INCR", "test"]))
        .await
        .map_err(error::ErrorInternalServerError)?
        .map_err(error::ErrorInternalServerError)?;

    match res {
        RespValue::Integer(x) => Ok(HttpResponse::Ok().body(x.to_string())),
        _ => {
            log::error!("{res:?}");
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
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

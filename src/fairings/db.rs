use rocket_db_pools::{deadpool_redis, mongodb::Client, Database};

#[derive(Database)]
#[database("mongo_pool")]
pub struct MongoDb(Client);

#[derive(Database)]
#[database("redis_pool")]
pub struct RedisPool(deadpool_redis::Pool);

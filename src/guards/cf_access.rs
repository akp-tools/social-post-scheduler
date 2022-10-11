use crate::jwt::{decode_jwt, CfAccessJwt};

use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};

#[derive(Debug)]
pub enum CfAccessJwtError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CfAccessJwt {
    type Error = CfAccessJwtError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match req.headers().get_one("cf-access-jwt-assertion") {
            None => Outcome::Failure((Status::BadRequest, CfAccessJwtError::Missing)),
            Some(jwt) => match decode_jwt(jwt).await {
                Ok(token) => Outcome::Success(token.claims),
                Err(_) => Outcome::Failure((Status::BadRequest, CfAccessJwtError::Invalid)),
            },
        }
    }
}

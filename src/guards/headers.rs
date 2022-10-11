pub struct Headers<'r>(&'r rocket::http::HeaderMap<'r>);

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for Headers<'r> {
    type Error = std::convert::Infallible;

    async fn from_request(
        req: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        rocket::request::Outcome::Success(Headers(req.headers()))
    }
}

impl<'r> std::ops::Deref for Headers<'r> {
    type Target = rocket::http::HeaderMap<'r>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

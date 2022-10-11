use rocket::{http::Status, response, response::Responder, Request, Response};

pub struct LocationResponder {
    pub location: String,
}

impl<'r> Responder<'r, 'static> for LocationResponder {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .raw_header("Location", self.location)
            .status(Status::TemporaryRedirect)
            .ok()
    }
}

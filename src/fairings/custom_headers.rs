use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

pub struct CustomHeaders;

#[rocket::async_trait]
impl Fairing for CustomHeaders {
    fn info(&self) -> Info {
        Info {
            name: "Custom Headers",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("X-Powered-By", "Rainbows and Shit"));
        response.set_header(Header::new("X-Cazif-Likes-Men", "Men don't like Cazif"));
    }
}

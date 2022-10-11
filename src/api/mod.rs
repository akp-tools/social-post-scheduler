mod facebook;

use crate::api::facebook::*;

pub fn stage() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("API", |rocket| async {
        rocket.mount("/api/v1", routes![facebook_login, facebook_redirect])
    })
}

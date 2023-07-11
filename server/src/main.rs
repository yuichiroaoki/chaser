use rocket::{fairing::{Fairing, Info, Kind}, http::Header, Request, Response};

#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

mod path;
mod quote;

pub struct CORS;

#[get("/")]
pub fn health_check() -> &'static str {
    "OK"
}

// ref: https://stackoverflow.com/a/64904947/13743156
#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}


#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/healthcheck", routes![health_check])
        .mount("/path", routes![path::get_path])
        .mount("/quote", routes![quote::quote_prices])
        .attach(CORS)
}

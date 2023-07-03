#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

mod path;
mod quote;

#[get("/")]
pub fn health_check() -> &'static str {
    "OK"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/healthcheck", routes![health_check])
        .mount("/path", routes![path::get_path])
        .mount("/quote", routes![quote::quote_prices])
}

#![feature(proc_macro_hygiene, decl_macro)]
use jsonwebtoken::DecodingKey;
use rocket::fairing::AdHoc;
use rocket::{get, routes, Rocket};
use rocket_jwt::{Decoding, JWT};
use std::env;

#[macro_use]
extern crate lazy_static;

#[get("/")]
fn index(_jwt: JWT) -> String {
    "".to_owned()
}

fn rocket(s: &'static [u8]) -> Rocket {
    rocket::ignite()
        .mount("/", routes![index])
        .attach(AdHoc::on_attach("SecretKey", move |r| {
            Ok(r.manage(Decoding {
                hs256: DecodingKey::from_secret(s),
            }))
        }))
}

fn main() -> Result<(), rocket::error::Error> {
    lazy_static! {
        static ref SECRET_KEY: String =
            env::var("SECRET_KEY").expect("SECRET_KEY in env");
    }
    rocket(SECRET_KEY.as_ref()).launch()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::Header as jwtHeader;
    use jsonwebtoken::{encode, EncodingKey};
    use rocket::http::{Header, Status};
    use rocket::local::Client;
    use rocket_jwt::Claims;

    fn secret_key() -> &'static [u8] {
        lazy_static! {
            static ref SECRET_KEY: String = "very_secret".to_string();
        }
        SECRET_KEY.as_ref()
    }

    fn jwt() -> String {
        let my_claims = Claims {
            exp: 10_000_000_000,
        };
        encode(
            &jwtHeader::default(),
            &my_claims,
            &EncodingKey::from_secret(secret_key()),
        )
        .unwrap()
    }

    #[rocket::async_test]
    async fn test_401() {
        let client = Client::new(rocket(secret_key())).unwrap();
        let response = client.get("/").dispatch().await;
        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[rocket::async_test]
    async fn test_200() {
        let header = Header::new("Authorization", "Bearer ".to_owned() + &jwt());
        let client = Client::new(rocket(secret_key())).unwrap();
        let response = client.get("/").header(header).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }
}

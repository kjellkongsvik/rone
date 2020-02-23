#![feature(proc_macro_hygiene, decl_macro)]
use rocket::fairing::AdHoc;
use rocket::{get, routes, Rocket};
use rocket_jwt::{TokenSecret, JWT};
use std::env;

#[macro_use]
extern crate lazy_static;

#[get("/")]
fn index(_jwt: JWT) -> String {
    "".to_owned()
}

fn rocket() -> Rocket {
    rocket::ignite().mount("/", routes![index])
}

fn main() -> Result<(), rocket::error::Error> {
    lazy_static! {
        static ref SECRET_KEY: String =
            env::var("SECRET_KEY").expect("SECRET_KEY in env");
    }
    rocket()
        .attach(AdHoc::on_attach("TokenSecret", |r| {
            Ok(r.manage(TokenSecret(SECRET_KEY.to_string())))
        }))
        .launch()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::Header as jwtHeader;
    use jsonwebtoken::{encode, EncodingKey};
    use rocket::http::{Header, Status};
    use rocket::local::Client;
    use rocket_jwt::Claims;

    fn secret_key() -> String {
        lazy_static! {
            static ref TOKEN_SECRET: String = "very_secret".to_string();
        }
        TOKEN_SECRET.to_string()
    }

    fn jwt() -> String {
        let my_claims = Claims {
            exp: 10_000_000_000,
        };
        encode(
            &jwtHeader::default(),
            &my_claims,
            &EncodingKey::from_secret(secret_key().as_ref()),
        )
        .unwrap()
    }

    fn manage_token_secret() -> AdHoc {
        AdHoc::on_attach("TokenSecret", |r| Ok(r.manage(TokenSecret(secret_key()))))
    }

    #[rocket::async_test]
    async fn test_401() {
        let client = Client::new(rocket().attach(manage_token_secret())).unwrap();
        let response = client.get("/").dispatch().await;
        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[rocket::async_test]
    async fn test_200() {
        let header = Header::new("Authorization", "Bearer ".to_owned() + &jwt());
        let client = Client::new(rocket().attach(manage_token_secret())).unwrap();
        let response = client.get("/").header(header).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }
}

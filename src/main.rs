#![feature(proc_macro_hygiene, decl_macro)]
use rocket::fairing::AdHoc;
use rocket::{get, routes, Rocket};
use rocket_jwt::{TokenSecret, JWT};

#[get("/")]
fn index(_jwt: JWT) -> String {
    "".to_owned()
}

fn rocket() -> Rocket {
    rocket::ignite().mount("/", routes![index])
}

fn main() -> Result<(), rocket::error::Error> {
    rocket()
        .attach(AdHoc::on_attach("TokenSecret", |r| {
            let token_val = match r.config().get_string("token_secret") {
                Ok(t) => t,
                _ => return Err(r),
            };
            Ok(r.manage(TokenSecret(token_val)))
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
        "very_secret".to_string()
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
        let header = Header::new("Authorization", jwt());
        let client = Client::new(rocket().attach(manage_token_secret())).unwrap();
        let response = client.get("/").header(header).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }
}

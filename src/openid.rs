use futures::executor::block_on;
use reqwest::Error;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Oid {
    jwks_uri: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Keys {
    e: String,
    n: String,
    alg: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Jwk {
    keys: Vec<Keys>,
}

async fn keys<'a, T>(uri: &str) -> Result<T, Error>
where
    for<'de> T: Deserialize<'de> + 'a,
{
    Ok(reqwest::get(uri).await?.json::<T>().await?)
}

pub fn en(uri: &str) -> Result<Vec<Keys>, Error> {
    Ok(
        block_on(async { keys::<Jwk>(&keys::<Oid>(uri).await?.jwks_uri).await })?
            .keys
            .into_iter()
            .filter(|k| k.alg == "RS256")
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[rocket::async_test]
    async fn e_n() -> Result<(), Error> {
        let disc = "/.well-known/openid-configuration";
        let some_uri = reqwest::Url::parse(&server_url())
            .unwrap()
            .join("/jwks")
            .unwrap();
        let disc_body = format!(r#"{{"jwks_uri": "{}"}}"#, some_uri);
        let e = "AQAB";
        let n = "yeNlzlub94YgerT030codqEztjfU_S6X4DbDA_iVKkjAWtYfPHDzz_sPCT1Axz6isZdf3lHpq_gYX4Sz-cbe4rjmigxUxr-FgKHQy3HeCdK6hNq9ASQvMK9LBOpXDNn7mei6RZWom4wo3CMvvsY1w8tjtfLb-yQwJPltHxShZq5-ihC9irpLI9xEBTgG12q5lGIFPhTl_7inA1PFK97LuSLnTJzW0bj096v_TMDg7pOWm_zHtF53qbVsI0e3v5nmdKXdFf9BjIARRfVrbxVxiZHjU6zL6jY5QJdh1QCmENoejj_ytspMmGW7yMRxzUqgxcAqOBpVm0b-_mW3HoBdjQ";
        let jwk_body = format!(
            r#"
            {{
                "keys": [
                    {{  "alg": "RS256",
                        "kty": "RSA",
                        "e": "{}",
                        "n": "{}",
                        "kid": "NjVBRjY5MDlCMUIwNzU4RTA2QzZFMDQ4QzQ2MDAyQjVDNjk1RTM2Qg"
                    }}
                ]
            }}
            "#,
            e, n
        );

        let disc_mock = mock("GET", disc)
            .with_header("content-type", "application/json")
            .with_body(disc_body)
            .expect(2)
            .create();
        let jwk_mock = mock("GET", some_uri.path())
            .with_header("content-type", "application/json")
            .with_body(jwk_body)
            .expect(2)
            .create();

        assert_eq!(
            keys::<Oid>(
                &reqwest::Url::parse(&server_url())
                    .unwrap()
                    .join(disc)
                    .unwrap()
                    .to_string()
            )
            .await
            .unwrap()
            .jwks_uri,
            some_uri.to_string()
        );

        assert_eq!(
            keys::<Jwk>(&some_uri.to_string()).await.unwrap(),
            Jwk {
                keys: vec!(Keys {
                    e: e.to_owned(),
                    n: n.to_owned(),
                    alg: "RS256".into()
                })
            }
        );

        assert_eq!(
            en(&reqwest::Url::parse(&server_url())
                .unwrap()
                .join(disc)
                .unwrap()
                .to_string())
            .unwrap(),
            vec!(Keys {
                e: e.to_owned(),
                n: n.to_owned(),
                alg: "RS256".into()
            })
        );

        jwk_mock.assert();
        disc_mock.assert();

        Ok(())
    }
}

use reqwest;
use reqwest::Error;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Oid {
    jwks_uri: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Keys {
    e: String,
    n: String,
}

async fn jwks_uri(v: &str) -> Result<String, Error> {
    Ok(reqwest::get(v).await?.json::<Oid>().await?.jwks_uri)
}

async fn keys(v: &str) -> Result<Keys, Error> {
    Ok(reqwest::get(v).await?.json::<Keys>().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[rocket::async_test]
    async fn e_n() {
        let e = "AQAB";
        let n = "yeNlzlub94YgerT030codqEztjfU_S6X4DbDA_iVKkjAWtYfPHDzz_sPCT1Axz6isZdf3lHpq_gYX4Sz-cbe4rjmigxUxr-FgKHQy3HeCdK6hNq9ASQvMK9LBOpXDNn7mei6RZWom4wo3CMvvsY1w8tjtfLb-yQwJPltHxShZq5-ihC9irpLI9xEBTgG12q5lGIFPhTl_7inA1PFK97LuSLnTJzW0bj096v_TMDg7pOWm_zHtF53qbVsI0e3v5nmdKXdFf9BjIARRfVrbxVxiZHjU6zL6jY5QJdh1QCmENoejj_ytspMmGW7yMRxzUqgxcAqOBpVm0b-_mW3HoBdjQ";
        let body = format!(
            r#"
            {{
                "keys": [
                    {{  "alg": "RS256",
                        "kty": "RSA",
                        "e": "{}",
                        "n": "{}",
                        "kid": "NjVBRjY5MDlCMUIwNzU4RTA2QzZFMDQ4QzQ2MDAyQjVDNjk1RTM2Qg",
                    }}
                ]
            }}
        "#,
            e, n
        );
        let mock = mock("GET", "/")
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();

        assert_eq!(
            keys(&(server_url())).await.unwrap(),
            Keys {
                e: e.into(),
                n: n.into(),
            }
        );

        mock.assert();
    }

    #[rocket::async_test]
    async fn with_jwks_uri() {
        let disc = "/.well-known/openid-configuration";
        let some_uri = "some.uri";
        let body = format!(r#"{{"jwks_uri": "{}"}}"#, some_uri);
        let mock = mock("GET", disc)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();

        assert_eq!(jwks_uri(&(server_url() + disc)).await.unwrap(), some_uri);

        mock.assert();
    }
}

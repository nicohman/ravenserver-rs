use jwt::{decode, encode, Algorithm, Header, Validation};
use reqwest::Url;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use rocket_failure::errors::*;
use std::borrow::Cow;
lazy_static! {
    pub static ref SECRET: String = env!("RAVENSERVER_SECRET").to_string();
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UserToken {
    pub name: String,
    pub id: String,
}
pub fn decode_user(un: impl Into<String>) -> Result<UserToken, jwt::errors::Error> {
    Ok(decode::<UserToken>(
        &un.into(),
        &SECRET.as_bytes(),
        &Validation {
            validate_exp: false,
            ..Default::default()
        },
    )?
    .claims)
}
pub fn encode_user(user: UserToken) -> Result<String, jwt::errors::Error> {
    Ok(encode::<UserToken>(&Header::default(), &user, SECRET.as_ref())?)
}
impl<'a, 'r> FromRequest<'a, 'r> for UserToken {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<UserToken, ()> {
        let url =
            Url::parse(&("https://demenses.net/?".to_string() + request.uri().query().unwrap()))
                .expect("Couldn't parse URI");
        println!("{:?}", url);
        for (k, v) in url.query_pairs() {
            if k == Cow::Borrowed("token") {
                if let Ok(token) = decode_user(v.clone()) {
                    return Outcome::Success(token);
                } else {
                    println!("{:?}", decode_user(v));
                    return Outcome::Failure((Status::InternalServerError, ()));
                }
            }
        }
        return Outcome::Failure((Status::Unauthorized, ()));
    }
}

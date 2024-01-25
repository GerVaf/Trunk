use actix_web::{HttpResponse, get, web::Json};
use argon2::verify_encoded;
use chrono::{Utc, Duration};
use jsonwebtoken::{Header, encode, EncodingKey};
use surrealdb::sql::Id;

use crate::structures::{DB, Resp, Login, Claims, DbUserInfo};

#[get("/login")]
pub async fn login(info: Json<Login>) -> HttpResponse {
    let db = DB.get().await;
    if db.use_ns("ns").use_db("db").await.is_err() {
        return HttpResponse::InternalServerError().json(Resp::new(
            "Sorry We are having some problem when opening our database!",
        ));
    }

    match db.select::<Option<DbUserInfo>>(("user", Id::String(info.username.to_string()))).await {
        Ok(Some(user)) => {
            match verify_encoded(&user.password, info.password.as_bytes()) {
                Ok(stat) => {
                    if !stat {
                        return HttpResponse::NotAcceptable().json(Resp::new("Sorry Wrong Password!"));
                    }

                    let exp = usize::try_from((Utc::now()+Duration::days(9_999_999)).timestamp()).unwrap();
                    let claims = Claims {
                        username: user.username,
                        password: user.password,
                        exp
                    };
                    encode(&Header::default(), &claims, &EncodingKey::from_secret("kshashdfjklasdhfsdhfkasjhfasdhHKHJHKJHSKJHKJSHJKHSJKHJKFHSKJ".as_bytes())).map_or_else(|_| HttpResponse::InternalServerError().json(Resp::new("Sorry We're Having Some Problem In Creating Your Account!")), |token| HttpResponse::Ok().json(Resp::new(&token)))
                },
                Err(_) => {
                    HttpResponse::InternalServerError().json(Resp::new("Sorry Something Went Wrong While Checking Your Password!"))
                }
            }
        },
        Ok(None) => {
            HttpResponse::Unauthorized().json(Resp::new("User Not Found!"))
        },
        Err(_) => {
            HttpResponse::InternalServerError().json(Resp::new("Sorry We're Having Some Problem in Searching Your Account!"))
        }
    }
}

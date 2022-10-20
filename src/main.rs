#[macro_use]
extern crate actix_web;
#[macro_use]
extern crate diesel;

use {
    actix_web::{web,middleware, App,HttpResponse, HttpServer,cookie::{self, Key},},
    actix_web::web::Data,
    
    actix_web::http::header,
   
};
use serde::{
    Deserialize, 
    Serialize
};
use actix_session::{
    config::PersistentSession, storage::CookieSessionStore, Session, SessionMiddleware,
};
use http::{HeaderMap, Method};
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::{
    AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use url::Url;
struct AppState {
    oauth: BasicClient,
}
async fn index(session: Session) -> HttpResponse {
    let login = session.get::<String>("login").unwrap();
    let link = if login.is_some() { "logout" } else { "login" };

    let html = format!(
        r#"<html>
        <head><title>Hithub Test</title></head>
        <body>
            {} <a href="/{}">{}</a>
        </body>
    </html>"#,
        login.unwrap_or("".to_string()),
        link,
        link
    );

    HttpResponse::Ok().body(html)
}

async fn login(data: web::Data<AppState>) -> HttpResponse {

   /*let (pkce_code_challenge, _pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, _csrf_token) = &data
        .oauth
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("public_repo".to_string()))
        .add_scope(Scope::new("user:email".to_string()))
        .set_pkce_challenge(pkce_code_challenge)
        .url();*/
        let (authorize_url, csrf_state) = &data.oauth
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the user's public repos and email.
        .add_scope(Scope::new("public_repo".to_string()))
        .add_scope(Scope::new("user:email".to_string()))
        .url();
    HttpResponse::Found()
        .append_header((header::LOCATION, authorize_url.to_string()))
        .finish()
       // HttpResponse::Ok().body(format!("username:"))
}

async fn logout(session: Session) -> HttpResponse {
    session.remove("login");
    HttpResponse::Found()
        .append_header((header::LOCATION, "/".to_string()))
        .finish()
}
#[derive(Deserialize)]
struct AuthRequest {
    code: String,
    state: String,
}

async fn auth(
    session: Session,
    data: web::Data<AppState>,
    params: web::Query<AuthRequest>,
) -> HttpResponse {
    let code = AuthorizationCode::new(params.code.clone());
    let _state = CsrfToken::new(params.state.clone());

    // Exchange the code with a token.
    let token = &data
        .oauth
        .exchange_code(code)
        .request(http_client)
        .expect("exchange_code failed");

    //let user_info = read_user(&data.api_base_url, token.access_token());
    session.insert("access_token", token.access_token().clone()).unwrap();
   

    let html = format!(
        r#"<html>
        <head><title>OAuth2 Test</title></head>
        <body>
            Gitlab user info:
            <pre></pre>
            <a href="/">Home</a>
        </body>
    </html>"#,
    );
    HttpResponse::Ok().body(html)
}

#[actix_rt::main]
async fn main() {
    HttpServer::new(|| {

        let github_client_id = ClientId::new(
            String::from("77c964a9f6a7106a5a0e")
        );
        let github_client_secret = ClientSecret::new(
            String::from("916d6cc2e912082f89891120b929680494467ba6")
        );
        let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
            .expect("Invalid authorization endpoint URL");
        let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
            .expect("Invalid token endpoint URL");
    
        // Set up the config for the Github OAuth2 process.
        let client = BasicClient::new(
            github_client_id,
            Some(github_client_secret),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(
            RedirectUrl::new("0.0.0.0:9090/auth".to_string()).expect("Invalid redirect URL"),
        );
        App::new()
        .app_data(web::Data::new(AppState {
                oauth: client,
            }))

            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    // customize session and cookie expiration
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(cookie::time::Duration::hours(2)),
                    )
                    .build(),
            )
            //.service(web::resource("/").
            //route(web::get().to(index)))
            //.service(web::resource("/login").
            //route(web::get().to(login)))
            .route("/", web::get().to(index))
            .route("/login", web::get().to(login))
            .route("/logout", web::get().to(logout))
            .route("/auth", web::get().to(auth))
    })
    .bind("0.0.0.0:9090")
    .expect("Can not bind to port 5000")
    .run()
    .await
    .unwrap();
}
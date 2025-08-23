# 🚀 Summary

- Backend: Rust + Actix Web
- Auth: JWT stored in HTTP-only cookies
- Database: SQL Server using tiberius
- Repository Pattern: SqlSrvDbFactoryBaseRepo + UserRepo
- Middleware: JwtAuth automatically protects routes under /user/\*
- Public Routes: /register, /login
- Protected Routes: /user/me, /user/{id} (update & delete)

# 📂 Folder Structure

```bash
my_actix_app/
│
├─ .env
├─ Cargo.toml
└─ src/
   ├─ main.rs
   ├─ models/user.rs
   ├─ services/auth_service.rs
   ├─ utils/jwt.rs
   ├─ repos/sqlsrv_repo.rs
   ├─ repos/user_repo.rs
   ├─ handlers/user_handler.rs
   └─ middleware/jwt_auth.rs
```

# 📝 `.env`

```env
# Logging
RUST_LOG=info

# JWT Secret
JWT_SECRET=supersecretjwtkey

# SQL Server connection string
DB_CONN=server=tcp:localhost,1433;User ID=sa;Password=YourPassword123;TrustServerCertificate=true;Database=mydb
```

# 📦 `Cargo.toml`

```toml
[package]
name = "my_actix_app"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4"
actix-rt = "2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
jsonwebtoken = "9"
actix-session = "0.7"
bcrypt = "0.14"
uuid = { version = "1", features = ["v4"] }
tiberius = { version = "0.11", features = ["sql-browser-tokio"] }
tokio = { version = "1", features = ["full"] }
tokio-util = "0.7"
futures = "0.3"
dotenvy = "0.15"
lazy_static = "1.5"
```

# 🗄 SQL Table Setup

```SQL
CREATE TABLE Users (
    Id UNIQUEIDENTIFIER PRIMARY KEY,
    Username NVARCHAR(100) NOT NULL UNIQUE,
    PasswordHash NVARCHAR(255) NOT NULL
);
```

# 🔑 Code

`src/main.rs`

```rust
use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;
use std::env;
use std::sync::Mutex;

mod models;
mod utils;
mod handlers;
mod services;
mod repos;
mod middleware;

use repos::sqlsrv_repo::SqlSrvDbFactoryBaseRepo;
use handlers::user_handler;
use middleware::jwt_auth::JwtAuth;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
    let connection_string = env::var("DB_CONN").expect("DB_CONN must be set");

    let db = SqlSrvDbFactoryBaseRepo::connect(&connection_string)
        .await
        .expect("Failed to connect to DB");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Mutex::new(db.clone())))
            .app_data(web::Data::new(secret.clone()))
            // Public routes
            .service(user_handler::register)
            .service(user_handler::login)
            // Protected routes under /user
            .service(
                web::scope("/user")
                    .wrap(JwtAuth { secret: secret.clone() })
                    .service(user_handler::me)
                    .service(user_handler::update)
                    .service(user_handler::delete)
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

`src/models/user.rs`

```rust
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
}
```

`src/services/auth_service.rs`

```rust
use crate::models::user::User;
use bcrypt::{hash, verify};
use uuid::Uuid;

pub fn hash_password(password: &str) -> String {
    hash(password, 4).unwrap()
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    verify(password, hash).unwrap()
}

pub fn create_user(username: &str, password: &str) -> User {
    User {
        id: Uuid::new_v4(),
        username: username.to_string(),
        password_hash: hash_password(password),
    }
}
```

`src/utils/jwt.rs`

```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn create_jwt(user_id: &str, secret: &str) -> String {
    let expiration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600;
    let claims = Claims { sub: user_id.to_string(), exp: expiration as usize };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())).unwrap()
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<String, ()> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())
        .map(|data| data.claims.sub)
        .map_err(|_| ())
}
```

`src/repos/sqlsrv_repo.rs`

```rust
use tiberius::{Client, Config, error::Error, QueryResult, Row};
use tokio_util::compat::TokioAsyncWriteCompatExt;
use tokio::net::TcpStream;
use futures::TryStreamExt;

#[derive(Clone)]
pub struct SqlSrvDbFactoryBaseRepo {
    pub client: Client<TcpStream>,
}

impl SqlSrvDbFactoryBaseRepo {
    pub async fn connect(connection_string: &str) -> Result<Self, Error> {
        let mut config: Config = connection_string.parse().unwrap();
        let tcp = TcpStream::connect(config.get_addr()).await?;
        tcp.set_nodelay(true)?;
        let client = Client::connect(config, tcp.compat_write()).await?;
        Ok(SqlSrvDbFactoryBaseRepo { client })
    }

    pub async fn execute(&mut self, sql: &str) -> Result<QueryResult, Error> {
        self.client.simple_query(sql).await
    }

    pub async fn query_one(&mut self, sql: &str) -> Result<Option<Row>, Error> {
        let mut stream = self.client.query(sql, &[]).await?;
        Ok(stream.try_next().await?)
    }
}
```

`src/repos/user_repo.rs`

```rust
use crate::models::user::User;
use crate::repos::sqlsrv_repo::SqlSrvDbFactoryBaseRepo;
use tiberius::Row;
use uuid::Uuid;

pub struct UserRepo<'a> {
    pub db: &'a mut SqlSrvDbFactoryBaseRepo,
}

impl<'a> UserRepo<'a> {
    pub async fn create(&mut self, user: &User) -> Result<(), tiberius::error::Error> {
        let sql = format!(
            "INSERT INTO Users (Id, Username, PasswordHash) VALUES ('{}','{}','{}')",
            user.id, user.username, user.password_hash
        );
        self.db.execute(&sql).await?;
        Ok(())
    }

    pub async fn get_by_username(&mut self, username: &str) -> Result<Option<User>, tiberius::error::Error> {
        let sql = format!("SELECT Id, Username, PasswordHash FROM Users WHERE Username='{}'", username);
        if let Some(row) = self.db.query_one(&sql).await? {
            Ok(Some(Self::map_row_to_user(row)))
        } else {
            Ok(None)
        }
    }

    pub async fn update(&mut self, user: &User) -> Result<(), tiberius::error::Error> {
        let sql = format!(
            "UPDATE Users SET Username='{}', PasswordHash='{}' WHERE Id='{}'",
            user.username, user.password_hash, user.id
        );
        self.db.execute(&sql).await?;
        Ok(())
    }

    pub async fn delete(&mut self, id: Uuid) -> Result<(), tiberius::error::Error> {
        let sql = format!("DELETE FROM Users WHERE Id='{}'", id);
        self.db.execute(&sql).await?;
        Ok(())
    }

    fn map_row_to_user(row: Row) -> User {
        User {
            id: row.get::<Uuid, _>("Id").unwrap(),
            username: row.get::<String, _>("Username").unwrap(),
            password_hash: row.get::<String, _>("PasswordHash").unwrap(),
        }
    }
}
```

`src/handlers/user_handler.rs`

```rust
use actix_web::{post, get, put, delete, web, HttpResponse, HttpRequest, cookie::Cookie, Responder};
use crate::models::user::User;
use crate::services::auth_service::{create_user, verify_password};
use crate::utils::jwt::create_jwt;
use crate::repos::sqlsrv_repo::SqlSrvDbFactoryBaseRepo;
use crate::repos::user_repo::UserRepo;
use uuid::Uuid;

#[post("/register")]
pub async fn register(
    user: web::Json<User>,
    secret: web::Data<String>,
    db: web::Data<std::sync::Mutex<SqlSrvDbFactoryBaseRepo>>,
) -> impl Responder {
    let mut db = db.lock().unwrap();
    let mut repo = UserRepo { db: &mut *db };
    let new_user = create_user(&user.username, &user.password_hash);

    if let Err(e) = repo.create(&new_user).await {
        return HttpResponse::InternalServerError().body(format!("DB Error: {:?}", e));
    }
    HttpResponse::Ok().json(new_user)
}

#[post("/login")]
pub async fn login(
    user: web::Json<User>,
    secret: web::Data<String>,
    db: web::Data<std::sync::Mutex<SqlSrvDbFactoryBaseRepo>>,
) -> impl Responder {
    let mut db = db.lock().unwrap();
    let mut repo = UserRepo { db: &mut *db };

    if let Ok(Some(db_user)) = repo.get_by_username(&user.username).await {
        if verify_password(&db_user.password_hash, &user.password_hash) {
            let token = create_jwt(&db_user.id.to_string(), secret);
            let cookie = Cookie::build("auth", token).path("/").http_only(true).finish();
            return HttpResponse::Ok().cookie(cookie).body("Logged in");
        }
    }
    HttpResponse::Unauthorized().body("Invalid credentials")
}

#[get("/me")]
pub async fn me(req: HttpRequest) -> impl Responder {
    if let Some(user_id) = req.extensions().get::<String>() {
        return HttpResponse::Ok().body(format!("Hello, user {}", user_id));
    }
    HttpResponse::Unauthorized().body("Unauthorized")
}

#[put("/{id}")]
pub async fn update(user: web::Json<User>, id: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("Update user {}", id))
}

#[delete("/{id}")]
pub async fn delete(id: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("Delete user {}", id))
}
```

`src/middleware/jwt_auth.rs`

```rust
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::rc::Rc;
use crate::utils::jwt::verify_jwt;

pub struct JwtAuth {
    pub secret: String,
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtAuthMiddleware {
            service: Rc::new(service),
            secret: self.secret.clone(),
        })
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    secret: String,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = Rc::clone(&self.service);
        let secret = self.secret.clone();

        Box::pin(async move {
            if let Some(cookie) = req.cookie("auth") {
                if let Ok(user_id) = verify_jwt(cookie.value(), &secret) {
                    req.extensions_mut().insert(user_id);
                    return svc.call(req).await;
                }
            }
            Ok(req.into_response(HttpResponse::Unauthorized().finish()))
        })
    }
}
```

✅ That’s the complete working codebase for Actix Web + SQL Server + JWT cookies + repository pattern.

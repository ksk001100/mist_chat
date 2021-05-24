use actix_web::{web, App, HttpServer};
use anyhow::Result;
use sqlx::mysql::MySqlPool;
use std::sync::Mutex;
use dotenv::dotenv;
use std::env;
use listenfd::ListenFd;

struct AppStateWithCounter {
    counter: Mutex<i64>,
}

async fn index(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;

    format!("Request number: {}", counter)
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let mut listenfd = ListenFd::from_env();

    let database_url = env::var("DATABASE_URL")?;
    let db_pool = MySqlPool::new(&database_url).await?;

    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(counter.clone())
            .data(db_pool.clone())
            .route("/", web::get().to(index))
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host = env::var("HOST")?;
            let port = env::var("PORT")?;
            server.bind(format!("{}:{}", host, port))?
        }
    };

    server.run().await?;

    Ok(())
}

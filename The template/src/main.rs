mod routes;
mod ratelimiter;

use warp::Filter;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::oneshot;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    let rate_limiter = Arc::new(Mutex::new(ratelimiter::RateLimiter::new(5, 8, std::time::Duration::new(60, 0))));

    let routes = routes::create_routes(rate_limiter.clone());

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        let mut sigterm = signal(SignalKind::terminate()).unwrap();

        tokio::select! {
            _ = sigint.recv() => println!("Received SIGINT, shutting down gracefully..."),
            _ = sigterm.recv() => println!("Received SIGTERM, shutting down gracefully..."),
        }

        let _ = shutdown_tx.send(());
    });

    let addr = ([127, 0, 0, 1], 3030);
    println!("Server running on http://127.0.0.1:3030");

    let (_, server) = warp::serve(routes)
        .bind_with_graceful_shutdown(addr, async {
            shutdown_rx.await.ok();
        });

    server.await;
    println!("Server has been gracefully shut down.");
}


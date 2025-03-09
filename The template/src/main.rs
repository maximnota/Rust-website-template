use warp::Filter;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::oneshot;
use warp::reply::{html};
use std::path::Path;

#[tokio::main]
async fn main() {
    // Serve static HTML files from the "pages" folder
    let pages = warp::fs::dir("pages");

    // Serve index.html for the root path
    let index = warp::path::end().map(|| {
        warp::reply::html(include_str!("../pages/index.html").to_string())
    });

    // Serve .html files without requiring the .html extension
    let dynamic_pages = warp::path!(String).map(|path: String| {
        let file_path = Path::new("pages").join(format!("{}.html", path));
        if file_path.exists() {
            html(std::fs::read_to_string(file_path).unwrap())
        } else {
            html("<html><body><h1>404 Not Found</h1></body></html>".to_string())
        }
    });

    // ✅ Serve static files (CSS, JS, images) from "static" directory
    let static_files = warp::path("static").and(warp::fs::dir("static"));

    // Combine all routes
    let routes = index.or(dynamic_pages).or(pages).or(static_files);

    // Create a oneshot channel for graceful shutdown
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Spawn a task to listen for shutdown signals
    tokio::spawn(async move {
        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        let mut sigterm = signal(SignalKind::terminate()).unwrap();

        tokio::select! {
            _ = sigint.recv() => {
                println!("Received SIGINT, shutting down gracefully...");
            }
            _ = sigterm.recv() => {
                println!("Received SIGTERM, shutting down gracefully...");
            }
        }

        let _ = shutdown_tx.send(());
    });

    let addr = ([127, 0, 0, 1], 3030);
    println!("Server running on http://127.0.0.1:3030");

    // Run the server with graceful shutdown
    let (_, server) = warp::serve(routes)
        .bind_with_graceful_shutdown(addr, async {
            shutdown_rx.await.ok();
        });

    server.await;
    println!("Server has been gracefully shut down.");
}
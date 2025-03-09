use warp::Filter;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::oneshot;
use warp::reply::{html};
use std::fmt::format;
use std::path::Path;
use std::fs;

#[tokio::main]
async fn main() {
    let pages = warp::fs::dir("pages");

    let index = warp::path::end().map(|| {
        warp::reply::html(include_str!("../pages/index.html").to_string())
    });

    let dynamic_pages = warp::path!(String).map(|path: String| {
        let file_path = Path::new("pages").join(format!("{}.html", path));
        if file_path.exists() {
            html(std::fs::read_to_string(file_path).unwrap_or_else(|_| "Error reading page.".to_string()))
        } else {
            // For the dynamic 404 page, include the 404.html and its associated CSS
            let error_404_path = Path::new("error_pages").join("html/404.html");
            let error_404_html = fs::read_to_string(error_404_path).unwrap_or_else(|_| "404 Error".to_string());
            
            // Include the CSS for the 404 page
            let error_404_css_path = Path::new("error_pages").join("css/404.css");
            let error_404_css = fs::read_to_string(error_404_css_path).unwrap_or_else(|_| "body { background-color: #f00; }".to_string());

            // Inject CSS into the HTML
            let full_html = format!(
                "<style>{}</style>{}",
                error_404_css, error_404_html
            );

            html(full_html)
        }
    });

    let static_files = warp::path("static").and(warp::fs::dir("static"));

    let routes = index.or(dynamic_pages).or(pages).or(static_files);

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Spawn a task to handle signals (SIGINT and SIGTERM)
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

        // Signal the shutdown channel to notify the server to shut down
        let _ = shutdown_tx.send(());
    });

    let addr = ([127, 0, 0, 1], 3030);
    println!("Server running on http://127.0.0.1:3030");

    // Run the warp server with graceful shutdown
    let (_, server) = warp::serve(routes)
        .bind_with_graceful_shutdown(addr, async {
            shutdown_rx.await.ok();  // Wait for the shutdown signal
        });

    // Await server shutdown
    server.await;
    println!("Server has been gracefully shut down.");
}


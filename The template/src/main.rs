use warp::Filter;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::oneshot;
use warp::reply::html;
use std::path::Path;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use warp::filters::addr::remote;
use std::net::SocketAddr;

mod ratelimiting;
pub use crate::ratelimiting::RateLimiter as ExternalRateLimiter;

#[tokio::main]
async fn main() {
    let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(5, Duration::new(60, 0))));

    let pages = warp::fs::dir("pages");

    // Apply rate limiting to the index page (main.html)
    let index = warp::path::end()
        .and(remote()) // Extract IP
        .and(warp::header::optional::<String>("x-forwarded-for")) // Extract forwarded IP
        .and(with_rate_limiter(rate_limiter.clone()))
        .map(|remote: Option<SocketAddr>, forwarded: Option<String>, rate_limiter: Arc<Mutex<RateLimiter>>| {
            let ip = extract_real_ip(remote, forwarded);
            let mut limiter = rate_limiter.lock().unwrap();

            if limiter.allow_request(&ip) {
                html(include_str!("../pages/index.html").to_string())
            } else {
                serve_rate_limit_page()
            }
        });

    let dynamic_pages = warp::path!(String)
        .and(remote())
        .and(warp::header::optional::<String>("x-forwarded-for"))
        .and(with_rate_limiter(rate_limiter.clone()))
        .map(|path: String, remote: Option<SocketAddr>, forwarded: Option<String>, rate_limiter: Arc<Mutex<RateLimiter>>| {
            let ip = extract_real_ip(remote, forwarded);
            let mut limiter = rate_limiter.lock().unwrap();

            if limiter.allow_request(&ip) {
                let file_path = Path::new("pages").join(format!("{}.html", path));
                if file_path.exists() {
                    html(fs::read_to_string(file_path).unwrap_or_else(|_| "Error reading page.".to_string()))
                } else {
                    serve_404_page()
                }
            } else {
                serve_rate_limit_page()
            }
        });

    let static_files = warp::path("static").and(warp::fs::dir("static"));

    let routes = index.or(dynamic_pages).or(pages).or(static_files);

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

// Extract real IP from headers or connection
fn extract_real_ip(remote: Option<SocketAddr>, forwarded: Option<String>) -> String {
    if let Some(forwarded_ip) = forwarded {
        if let Some(real_ip) = forwarded_ip.split(',').next() {
            return real_ip.trim().to_string();
        }
    }
    remote.map(|addr| addr.ip().to_string()).unwrap_or_else(|| "unknown".to_string())
}

// Helper function to serve 404 page
fn serve_404_page() -> warp::reply::Html<String> {
    let error_404_path = Path::new("error_pages").join("html/404.html");
    let error_404_html = fs::read_to_string(error_404_path).unwrap_or_else(|_| "404 Error".to_string());
    let error_404_css_path = Path::new("error_pages").join("css/404.css");
    let error_404_css = fs::read_to_string(error_404_css_path).unwrap_or_else(|_| "body { background-color: #f00; }".to_string());
    let full_html = format!("<style>{}</style>{}", error_404_css, error_404_html);
    html(full_html)
}

// Helper function to serve rate limit page
fn serve_rate_limit_page() -> warp::reply::Html<String> {
    let rate_limit_page = Path::new("error_pages").join("html/ratelimit.html");
    let error_rate_limit_html = fs::read_to_string(rate_limit_page).unwrap_or_else(|_| "Rate limit exceeded".to_string());
    let rate_limit_css_path = Path::new("error_pages").join("css/ratelimit.css");
    let rate_limit_css = fs::read_to_string(rate_limit_css_path).unwrap_or_else(|_| "body { background-color: #f00; }".to_string());
    let full_html = format!("<style>{}</style>{}", rate_limit_css, error_rate_limit_html);
    html(full_html)
}

// Inject rate limiter into routes
fn with_rate_limiter(
    rate_limiter: Arc<Mutex<RateLimiter>>,
) -> impl Filter<Extract = (Arc<Mutex<RateLimiter>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || rate_limiter.clone())
}

// Rate limiter struct
pub struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>, 
    max_requests: usize,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_duration: Duration) -> Self {
        RateLimiter {
            requests: HashMap::new(),
            max_requests,
            window_duration,
        }
    }

    pub fn allow_request(&mut self, ip: &str) -> bool {
        let now = Instant::now();
        let entry = self.requests.entry(ip.to_string()).or_insert_with(Vec::new);

        entry.retain(|&timestamp| now.duration_since(timestamp) < self.window_duration);

        if entry.len() < self.max_requests {
            entry.push(now);
            true
        } else {
            false
        }
    }
}


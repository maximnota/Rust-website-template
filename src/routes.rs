use warp::Filter;
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::Path;
use warp::reply::html;
use warp::filters::addr::remote;
use std::net::SocketAddr;
use crate::ratelimiter::{RateLimiter, RateLimitStatus};

// Create all routes
pub fn create_routes(rate_limiter: Arc<Mutex<RateLimiter>>) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let index = warp::path::end()
        .and(remote())
        .and(warp::header::optional::<String>("x-forwarded-for"))
        .and(with_rate_limiter(rate_limiter.clone()))
        .map(handle_request);

    let dynamic_pages = warp::path!(String)
        .and(remote())
        .and(warp::header::optional::<String>("x-forwarded-for"))
        .and(with_rate_limiter(rate_limiter.clone()))
        .map(handle_request_dynamic);

    let static_files = warp::path("static").and(warp::fs::dir("static"));
    
    index.or(dynamic_pages).or(static_files).boxed()
}

// Handle request to index page
fn handle_request(remote: Option<SocketAddr>, forwarded: Option<String>, rate_limiter: Arc<Mutex<RateLimiter>>) -> impl warp::Reply {
    let ip = extract_real_ip(remote, forwarded);
    let mut limiter = rate_limiter.lock().unwrap();

    match limiter.check_request(&ip) {
        RateLimitStatus::Allowed => html(include_str!("../pages/index.html").to_string()),
        RateLimitStatus::Warning => serve_rate_limit_page(true),
        RateLimitStatus::Banned => html("<h1>403 Forbidden</h1><p>You have been banned.</p>".to_string()),
    }
}

// Handle requests to dynamic pages
fn handle_request_dynamic(path: String, remote: Option<SocketAddr>, forwarded: Option<String>, rate_limiter: Arc<Mutex<RateLimiter>>) -> impl warp::Reply {
    let ip = extract_real_ip(remote, forwarded);
    let mut limiter = rate_limiter.lock().unwrap();

    match limiter.check_request(&ip) {
        RateLimitStatus::Allowed => {
            let file_path = Path::new("pages").join(format!("{}.html", path));
            if file_path.exists() {
                html(fs::read_to_string(file_path).unwrap_or_else(|_| "Error reading page.".to_string()))
            } else {
                serve_404_page()
            }
        },
        RateLimitStatus::Warning => serve_rate_limit_page(true),
        RateLimitStatus::Banned => serve_rate_limit_page(false),
    }
}

// Extract real IP address
fn extract_real_ip(remote: Option<SocketAddr>, forwarded: Option<String>) -> String {
    if let Some(forwarded_ip) = forwarded {
        if let Some(real_ip) = forwarded_ip.split(',').next() {
            return real_ip.trim().to_string();
        }
    }
    remote.map(|addr| addr.ip().to_string()).unwrap_or_else(|| "unknown".to_string())
}

// Serve 404 page
fn serve_404_page() -> warp::reply::Html<String> {
    let path = Path::new("error_pages/html/404.html");
    let html_content = fs::read_to_string(path).unwrap_or_else(|_| "404 Not Found".to_string());
    html(html_content)
}

// Serve rate limit page
fn serve_rate_limit_page(is_warning: bool) -> warp::reply::Html<String> {
    let path = Path::new("error_pages/html/ratelimit.html");
    let mut html_content = fs::read_to_string(path).unwrap_or_else(|_| "Rate limit exceeded".to_string());

    if is_warning {
        html_content = html_content.replace("Rate limit exceeded", "Warning: You are nearing the rate limit!");
    }

    html(html_content)
}

// Middleware to inject rate limiter
fn with_rate_limiter(
    rate_limiter: Arc<Mutex<RateLimiter>>,
) -> impl Filter<Extract = (Arc<Mutex<RateLimiter>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || rate_limiter.clone())
}


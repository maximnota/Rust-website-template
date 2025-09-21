# Rust Web Application Template

A simple, fast, and secure web server built with Rust using the Warp framework. Features built-in rate limiting, static file serving, and graceful shutdown handling.

## Features

- 🚀 Fast HTTP server using Warp
- 🛡️ Built-in rate limiting with IP-based tracking
- 📁 Static file serving
- 🎨 Custom error pages (404, rate limit)
- ⚡ Graceful shutdown handling
- 🐳 Docker support
- 🚂 Railway deployment ready

## Local Development

### Prerequisites

- Rust 1.75+ installed
- Cargo package manager

### Running Locally

1. Clone the repository
2. Install dependencies:
   ```bash
   cargo build
   ```

3. Start the development server:
   ```bash
   cargo run
   ```

4. Visit `http://localhost:3030` in your browser

### Project Structure

```
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Main server logic
│   └── ratelimiter.rs   # Rate limiting implementation
├── pages/               # HTML pages
│   └── index.html       # Main page
├── static/              # Static assets (CSS, JS, images)
├── error_pages/         # Error page templates
│   ├── html/
│   │   ├── 404.html
│   │   └── ratelimit.html
│   └── css/
│       ├── 404.css
│       └── ratelimit.css
├── Dockerfile           # Docker configuration
├── railway.toml         # Railway deployment config
└── Cargo.toml           # Rust dependencies
```

## Deployment to Railway

### Quick Deploy

1. Fork this repository
2. Connect your GitHub account to [Railway](https://railway.app)
3. Create a new project and connect your forked repository
4. Railway will automatically detect the Dockerfile and deploy your application
5. Your app will be available at the provided Railway URL

### Manual Deploy

1. Install the Railway CLI:
   ```bash
   npm install -g @railway/cli
   ```

2. Login to Railway:
   ```bash
   railway login
   ```

3. Initialize Railway project:
   ```bash
   railway init
   ```

4. Deploy:
   ```bash
   railway up
   ```

### Environment Variables

The application automatically reads the `PORT` environment variable provided by Railway. No additional configuration needed.

## Rate Limiting

The server includes built-in rate limiting:

- **Warning threshold**: 4 requests per minute
- **Ban threshold**: 5 requests per minute
- **Window duration**: 60 seconds

Rate limiting is applied per IP address and includes support for `x-forwarded-for` headers (useful for reverse proxies).

## Health Check

The application includes a health check endpoint at `/health` that returns a simple "OK" status. This is used by Railway to monitor application health.

## API Endpoints

- `GET /` - Serves the main index page
- `GET /{page}` - Serves dynamic pages from the `pages/` directory
- `GET /static/*` - Serves static files
- `GET /health` - Health check endpoint

## Customization

### Adding New Pages

1. Create a new HTML file in the `pages/` directory
2. The file will be automatically accessible at `/{filename}` (without the .html extension)

### Modifying Rate Limits

Edit the rate limiter configuration in `src/lib.rs`:

```rust
let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(
    4,                    // warning_threshold
    5,                    // ban_threshold  
    Duration::new(60, 0)  // window_duration
)));
```

### Custom Error Pages

Modify the HTML and CSS files in the `error_pages/` directory to customize the appearance of error pages.

## Building for Production

### Using Docker

```bash
docker build -t rust-web-app .
docker run -p 3030:3030 rust-web-app
```

### Native Build

```bash
cargo build --release
./target/release/rwa
```

## License

This project is open source and available under the [MIT License](LICENSE).
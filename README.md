# 🚀 Rust URL Shortener

A lightning-fast URL shortener service built with Rust. Turn long, unwieldy URLs into short, memorable links in milliseconds!

## ✨ What Can It Do?

- **Shorten URLs**: Transform `https://this-is-a-very-long-url-that-nobody-wants-to-type.com/with/lots/of/parameters?and=stuff` into `yourdomain.com/abc123`
- **Custom Short Links**: Create your own memorable short URLs (like `yourdomain.com/rust`)
- **Track Clicks**: See how many times your shortened links have been clicked
- **API-First**: Easy-to-use REST API for all your shortening needs

## 🔧 Built With

- **Rust** + **Tokio**: For blazing fast, concurrent operations
- **Axum**: Modern, ergonomic web framework
- **SQLite**: Lightweight, embedded database (async-ready!)
- **Clean Architecture**: Handlers → Services → Storage layers

## 🚦 Getting Started

### Prerequisites

- Rust (latest stable)
- SQLite

### Quick Start

1. **Clone & Enter**
   ```bash
   git clone https://github.com/yourusername/rust_url_shortener.git
   cd rust_url_shortener
   ```

2. **Set Environment** (or use defaults)
   ```bash
   cp .env.example .env
   ```

3. **Launch!**
   ```bash
   cargo run
   ```

4. **Enjoy at http://localhost:8080**

## 🔌 API Guide

### Shorten a URL
```bash
curl -X POST http://localhost:8080/api/shorten \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com/very/long/path"}'
```
Response:
```json
{
  "short_id": "abc123",
  "original_url": "https://example.com/very/long/path"
}
```

### Create Custom Short URL
```bash
curl -X POST http://localhost:8080/api/shorten \
  -H "Content-Type: application/json" \
  -d '{"url": "https://rust-lang.org", "custom_id": "rust"}'
```

### Get Link Stats
```bash
curl http://localhost:8080/api/stats/rust
```

### Use Your Short Link
Just open `http://localhost:8080/rust` in your browser!

## 🧪 Testing

The project includes both unit tests and end-to-end API tests to ensure everything works correctly.

### Unit Tests
```bash
cargo test
```

### API Integration Tests
```bash
./test_api.sh
```

## 🔄 CI/CD Pipeline

The project uses GitHub Actions to automate testing and quality checks:

- ✅ **Unit Tests**: All code tests
- 🔌 **API Integration Tests**: End-to-end API functionality tests
- 📝 **Code Formatting**: Ensures consistent code style
- 🔍 **Clippy Lints**: Catches common mistakes and improvements

## 🏗️ Architecture

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│  HTTP Layer │────▶│  Services   │────▶│   Storage   │
│  (Handlers) │◀────│ (Business   │◀────│  (Database) │
└─────────────┘      │    Logic)   │      └─────────────┘
                     └─────────────┘
```

## 🔮 Future Improvements

- Redis-based caching layer
- PostgreSQL storage backend
- Analytics dashboard
- Rate limiting
- Link expiration
- Prometheus metrics integration

## 📜 License

MIT

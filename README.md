# MCP Client + Server SDK (Production-Ready)

This is a containerized MCP orchestration system with:

- SQLite-backed config (global + project-level)
- Web-based admin interface (`/admin/mcp-editor`)
- Next.js client + CLI agent support
- Docker + Compose orchestration
- TLS auto-renewal with Certbot
- Remote debugging + live reload
- Reverse-proxy with NGINX

---

## Quick Start

### 1. Clone & Build

```bash
docker-compose up --build
```

Then access:
- App: http://localhost
- Admin UI: http://localhost/admin/mcp-editor

---

## TLS Auto-Renewal (via Certbot)

The system uses `certbot-renew.sh` inside the `certbot` container to auto-renew every 12 hours.

Make sure ports 80 and 443 are open and DNS is pointed to your server before production deployment.

---

## Remote Debugging

Debug server is exposed on port `9229`.

---

## Configuration

- MCP server endpoints stored in `mcp_config.db`
- Admins can edit endpoints dynamically via UI
- Configs scoped globally or by session

---

## Dev Mode

```bash
npm install
npm run dev
```

---

## NGINX TLS Reverse Proxy

Auto config via:

```nginx
server {
  listen 80;
  server_name mcp.local;

  location /.well-known/acme-challenge/ {
    root /var/www/certbot;
  }

  location / {
    proxy_pass http://app:3000;
  }
}

server {
  listen 443 ssl;
  server_name mcp.local;

  ssl_certificate /etc/letsencrypt/live/mcp.local/fullchain.pem;
  ssl_certificate_key /etc/letsencrypt/live/mcp.local/privkey.pem;

  location / {
    proxy_pass http://app:3000;
  }
}
```

---

## Environment Template

`.env.example` is included for custom environment variables.

---

## License

MIT

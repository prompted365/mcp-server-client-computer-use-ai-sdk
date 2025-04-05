# Deploying MCP SDK on Railway

This guide helps you deploy the production-ready MCP client/server app on [Railway](https://railway.app).

---

## Setup Instructions

1. **Push this repo to GitHub**

2. **Create a new project on Railway**

3. **Connect your GitHub repo**

4. **Add environment variables**

| Key           | Value         |
|---------------|---------------|
| `NODE_ENV`    | `production`  |
| `PORT`        | `3000`        |
| `CLOUD_DEPLOY`| `true`        |

5. **Add a volume**
   - Mount it at `/data` (default location for SQLite)
   - Name it `mcp_data` or similar

6. **Deploy**

---

## Notes

- TLS termination must be handled by Railway or a fronting proxy (Cloudflare recommended)
- This deployment disables Docker and Compose
- Certbot should be handled externally for domain-based SSL

---

## Management

- Admin UI: `/admin/mcp-editor`
- SQLite file: stored in `/data/mcp_config.db` volume

--- 

## Debug

- Add `DEBUG_MODE=true` to see console logs
- Use Railway shell to inspect `/data/mcp_config.db`

---

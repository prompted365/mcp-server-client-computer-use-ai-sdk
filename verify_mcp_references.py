import re
import os
from pathlib import Path

base_dir = Path(__file__).resolve().parent

def check(file, patterns):
    path = base_dir / file
    if not path.exists():
        return {file: "❌ FILE MISSING"}
    content = path.read_text()
    return {pattern: "✅" if re.search(pattern, content) else "❌" for pattern in patterns}

results = {}

# Check Docker/compose/live reload
results.update(check("docker-compose.yml", [
    "volumes:.*mcp_config.db",
    "command:.*dev",
    "certbot/certbot",
    "9229"
]))

# Check TLS and nginx rewrite rules
results.update(check("nginx.conf", [
    "listen 443 ssl",
    "certbot",
    "proxy_pass http://app:3000"
]))

# Check environment variables
results.update(check(".env.example", [
    "NODE_ENV",
    "PORT",
    "MCP_DB_PATH",
    "DEBUG_MODE"
]))

# Check config.ts for db usage
results.update(check("src/lib/mcp/config.ts", [
    "sqlite3",
    "MCP_COOKIE_KEY"
]))

# Check config route API
results.update(check("src/app/api/mcp/config/route.ts", [
    "session_id",
    "INSERT OR REPLACE"
]))

for k, v in results.items():
    print(f"{k}: {v}")

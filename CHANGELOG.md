# Changelog: Multi-Server MCP Client Support

## Overview
This update introduces dynamic MCP server selection per user/session via cookie. It enables runtime switching between multiple backend endpoints and supports both CLI and web-based workflows through persistent configuration.

---

## New Features

### 1. Dynamic MCP Server Resolution via Cookie
- **File**: `src/lib/mcp/config.ts`
- **Added**: `getMCPBaseURL()` function
  - Maps session IDs (e.g., `dev`, `prod`, `test`) to endpoint URLs.
  - Reads session ID from the `mcp_session_id` cookie via Next.js `cookies()` API.

- **Added**: `setMCPSessionCookie()` function
  - Allows API routes to persist selected endpoint.

---

### 2. MCP Server Selector UI
- **File**: `src/components/MCPEndpointSelector.tsx`
- **Added**: React component to let users choose between predefined endpoints (default, dev, test, prod).
- **Behavior**:
  - Sends `POST` to `/api/mcp/session`
  - Refreshes page after server selection

---

### 3. MCP Session Cookie API
- **File**: `src/app/api/mcp/session/route.ts`
- **Added**: API route to store `mcp_session_id` via a secure HTTP cookie.

---

### 4. Initialize API Updated to Use Configurable Server
- **File**: `src/app/api/mcp/initialize/route.ts`
- **Changed**:
  - Replaced hardcoded MCP URL with `getMCPBaseURL()`.
  - Injected selected URL into log and JSON response.

---

## Implications

- **Multi-instance control**: Different users/sessions can operate on different machines or environments.
- **Security**: Client does not have direct access to endpoint URLs—IDs are mapped internally.
- **Extensibility**: Future server environments can be added via configuration updates without UI refactoring.

---

## Next Steps (Optional)

- Persist dropdown selection in `localStorage` for improved UX.
- Display the currently connected MCP endpoint in UI.
- Add a backend validation layer to only allow whitelisted endpoint aliases.
- Auto-reconnect on session change (instead of full reload).

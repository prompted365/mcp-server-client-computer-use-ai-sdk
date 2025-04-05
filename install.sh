#!/bin/bash
echo "==== MCP Client Installer ===="

# Step 1: Install dependencies
echo "Installing dependencies..."
npm install || { echo 'npm install failed'; exit 1; }

# Step 2: Ensure SQLite is installed
if ! command -v sqlite3 &> /dev/null; then
  echo "SQLite is not installed. Please install SQLite and rerun this script."
  exit 1
fi

# Step 3: Initialize SQLite DB if not present
if [ ! -f "./mcp_config.db" ]; then
  echo "Initializing local SQLite database..."
  sqlite3 mcp_config.db < ./schema.sql
fi

# Step 4: Notify user
echo "Installation complete!"
echo "Start the dev server using: npm run dev"
echo "Access MCP admin at: http://localhost:3000/admin/mcp-editor"

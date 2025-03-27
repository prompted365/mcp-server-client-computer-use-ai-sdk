// Browser/Next.js compatible version of setup

// Check if MCP server is running
export async function checkMCPServer() {
  try {
    console.log("checking mcp server connection...");
    
    // Direct connection to 127.0.0.1:8080
    const payload = {
      jsonrpc: "2.0", 
      id: "health-check",
      method: "initialize",
      params: {
        clientInfo: {
          name: "mcp-client-health-check",
          version: "1.0.0"
        },
        capabilities: {}
      }
    };
    
    const response = await fetch('http://127.0.0.1:8080/mcp', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
    });
    
    if (!response.ok) {
      throw new Error(`HTTP error: ${response.status}`);
    }
    
    const data = await response.json();
    
    if (data.result) {
      console.log("mcp server is running and responding properly");
      return true;
    }
    
    console.log("mcp server responded but with unexpected format");
    return false;
  } catch (error) {
    console.error("failed to connect to mcp server:", error.message);
    return false;
  }
}

// Setup environment - simplified for Next.js
export async function setupEnvironment() {
  // API keys should be handled through Next.js environment variables
  // in .env.local files or deployment environment
  
  const serverRunning = await checkMCPServer();
  if (!serverRunning) {
    console.error("mcp server check failed - functionality may be limited");
    // Don't exit process in Next.js - just return false
    return false;
  }
  
  return true;
}
import Anthropic from "@anthropic-ai/sdk";

// enhanced logging utility with colors for better readability
export const log = {
  info: (msg: string, ...args: any[]) => console.log(`\x1b[36m[info]\x1b[0m ${msg}`, ...args), 
  success: (msg: string, ...args: any[]) => console.log(`\x1b[32m[success]\x1b[0m ${msg}`, ...args),
  error: (msg: string, ...args: any[]) => console.error(`\x1b[31m[error]\x1b[0m ${msg}`, ...args),
  warn: (msg: string, ...args: any[]) => console.log(`\x1b[33m[warn]\x1b[0m ${msg}`, ...args),
  debug: (msg: string, ...args: any[]) => console.log(`\x1b[90m[debug]\x1b[0m ${msg}`, ...args),
  // New logging methods for specific UI elements
  highlight: (msg: string, ...args: any[]) => console.log(`\x1b[1m\x1b[35m${msg}\x1b[0m`, ...args),
  iteration: (msg: string, ...args: any[]) => console.log(`\x1b[36m${msg}\x1b[0m`, ...args),
  response: (msg: string) => console.log(`\n\x1b[1m\x1b[37mresponse:\x1b[0m ${msg}`),
  tool: (name: string, result: any) => {
    const truncateJSON = (obj: any, maxLength = 500): string => {
      const str = JSON.stringify(obj);
      if (str.length <= maxLength) return str;
      return str.substring(0, maxLength) + '... [truncated]';
    };
    
    if (result?.isError) {
      console.log(`\x1b[31m[tool ${name}]\x1b[0m ${truncateJSON(result)}`);
    } else {
      console.log(`\x1b[32m[tool ${name}]\x1b[0m ${truncateJSON(result)}`);
    }
  }
};

class DesktopControlClient {
  private connected = false;
  private serverUrl = "";
  private requestId = 0;
  private anthropic = new Anthropic();
  
  // Connect to the MCP server via http
  async connect(serverUrl: string) {
    log.info(`connecting to mcp server: ${serverUrl}`);
    
    try {
      this.serverUrl = serverUrl;
      const response = await this.makeRequest("initialize", {});
      
      if (response.result) {
        this.connected = true;
        log.success('mcp client session established successfully');
        return true;
      } else {
        log.error('failed to establish mcp client session:', response.error);
        return false;
      }
    } catch (error) {
      log.error('failed to establish mcp client session:', error);
      return false;
    }
  }
  
  // Make a JSON-RPC request
  private async makeRequest(method: string, params: any) {
    const id = `request-${++this.requestId}`;
    
    const response = await fetch("http://127.0.0.1:8080/mcp", {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id,
        method,
        params
      })
    });
    
    return await response.json();
  }
  
  // Check if connected
  isConnected(): boolean {
    return this.connected;
  }
  
  // List available tools
  async listTools() {
    if (!this.isConnected()) {
      log.error('cannot list tools: not connected');
      throw new Error('Not connected to MCP server');
    }
    
    try {
      // In standard MCP, this would be tools/list
      // But our rust server exposes tools through initialize
      const response = await this.makeRequest("initialize", {});
      const tools = response.result.capabilities.tools.functions;
      
      // Create simplified view - one line per tool
      log.info('available tools:');
      tools.forEach((tool: any) => {
        const propertyNames = Object.keys(tool.parameters.properties || {}).join(', ');
        log.debug(`- ${tool.name}: ${propertyNames}`);
      });
      
      return { tools };
    } catch (error) {
      log.error('failed to list tools:', error);
      throw error;
    }
  }
  
  // Call a tool
  async callTool(name: string, args: Record<string, any>) {
    if (!this.isConnected()) {
      log.error('cannot call tool: not connected');
      throw new Error('Not connected to MCP server');
    }
    
    log.info(`calling tool "${name}" with args: ${JSON.stringify(args)}`);
    
    try {
      const response = await this.makeRequest("executeToolFunction", {
        function: name,
        arguments: args
      });
      
      log.tool(name, response.result);
      return response.result;
    } catch (error) {
      log.error(`error calling tool "${name}":`, error);
      throw error;
    }
  }
  
  // Disconnect from the server
  async disconnect() {
    this.connected = false;
    log.success('mcp client session closed');
  }
}

// Export an instance that can be used throughout your application
export const desktopClient = new DesktopControlClient();

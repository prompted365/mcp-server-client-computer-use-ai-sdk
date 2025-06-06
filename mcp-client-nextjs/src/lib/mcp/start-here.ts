import Anthropic from "@anthropic-ai/sdk";
import { logBuffer } from './log-buffer';

// enhanced logging utility with colors for better readability
export const log = {
  info: (msg: string, ...args: unknown[]) => {
    console.log(`\x1b[36m[info]\x1b[0m ${msg}`, ...args);
    logBuffer.addLog('info', formatLogMessage(msg, args));
  }, 
  success: (msg: string, ...args: unknown[]) => {
    console.log(`\x1b[32m[success]\x1b[0m ${msg}`, ...args);
    logBuffer.addLog('success', formatLogMessage(msg, args));
  },
  error: (msg: string, ...args: unknown[]) => {
    console.error(`\x1b[31m[error]\x1b[0m ${msg}`, ...args);
    logBuffer.addLog('error', formatLogMessage(msg, args));
  },
  warn: (msg: string, ...args: unknown[]) => {
    console.log(`\x1b[33m[warn]\x1b[0m ${msg}`, ...args);
    logBuffer.addLog('warn', formatLogMessage(msg, args));
  },
  debug: (msg: string, ...args: unknown[]) => {
    console.log(`\x1b[90m[debug]\x1b[0m ${msg}`, ...args);
    logBuffer.addLog('debug', formatLogMessage(msg, args));
  },
  // New logging methods for specific UI elements
  highlight: (msg: string, ...args: unknown[]) => {
    console.log(`\x1b[1m\x1b[35m${msg}\x1b[0m`, ...args);
    logBuffer.addLog('highlight', formatLogMessage(msg, args));
  },
  iteration: (msg: string, ...args: unknown[]) => {
    console.log(`\x1b[36m${msg}\x1b[0m`, ...args);
    logBuffer.addLog('iteration', formatLogMessage(msg, args));
  },
  response: (msg: string) => {
    console.log(`\n\x1b[1m\x1b[37mresponse:\x1b[0m ${msg}`);
    logBuffer.addLog('response', msg);
  },
  tool: (name: string, result: unknown) => {
    const truncatedResult = truncateJSON(result);
    if (typeof result === 'object' && result !== null && 'isError' in result) {
      console.log(`\x1b[31m[tool ${name}]\x1b[0m ${truncatedResult}`);
      logBuffer.addLog('tool-error', `[${name}] ${truncatedResult}`);
    } else {
      console.log(`\x1b[32m[tool ${name}]\x1b[0m ${truncatedResult}`);
      logBuffer.addLog('tool', `[${name}] ${truncatedResult}`);
    }
  }
};

// Helper functions
function formatLogMessage(msg: string, args: unknown[]): string {
  if (args.length === 0) return msg;
  
  try {
    const formattedArgs = args.map(arg => 
      typeof arg === 'object' ? truncateJSON(arg) : String(arg)
    ).join(' ');
    return `${msg} ${formattedArgs}`;
  } catch (e) {
    return `${msg} [args formatting error]`;
  }
}

function truncateJSON(obj: unknown, maxLength = 500): string {
  try {
    const str = JSON.stringify(obj);
    if (str.length <= maxLength) return str;
    return str.substring(0, maxLength) + '... [truncated]';
  } catch (e) {
    return '[unserializable object]';
  }
}

type MCPResponse = {
  result?: unknown;
  error?: string;
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
  private async makeRequest(method: string, params: Record<string, unknown>) {
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
    
    return await response.json() as MCPResponse;
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
      const tools = (response.result as { capabilities: { tools: { functions: unknown[] } } }).capabilities.tools.functions;
      
      // Create simplified view - one line per tool
      log.info('available tools:');
      tools.forEach((tool: Record<string, unknown>) => {
        const params = tool.parameters as { properties?: Record<string, unknown> };
        const propertyNames = Object.keys(params.properties || {}).join(', ');
        log.debug(`- ${tool.name}: ${propertyNames}`);
      });
      
      return { tools };
    } catch (error) {
      log.error('failed to list tools:', error);
      throw error;
    }
  }
  
  // Call a tool
  async callTool(name: string, args: Record<string, unknown>) {
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
      
      // Check if result exists before logging
      if (response && 'result' in response) {
        log.tool(name, response.result);
        return response.result;
      } else {
        log.tool(name, response); // Log the entire response if result is missing
        return response; // Still return whatever we got
      }
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

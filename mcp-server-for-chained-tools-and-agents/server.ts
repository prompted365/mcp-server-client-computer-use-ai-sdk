import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import Anthropic from "@anthropic-ai/sdk";
import dotenv from "dotenv";
import { z } from "zod";
import fetch from "node-fetch";

// Load environment variables
dotenv.config();

// Server configuration
const MCP_SERVER_URL = process.env.MCP_SERVER_URL || "http://127.0.0.1:8080/mcp";

// Define global constants
export const CLAUDE_MODEL = "claude-3-7-sonnet-20250219";

// Initialize Anthropic client
const anthropic = new Anthropic({
  apiKey: process.env.ANTHROPIC_API_KEY,
});

// Add these interface definitions at the top of your file
interface JsonRpcResponse {
  jsonrpc: string;
  id: string;
  result?: any;
  error?: {
    code: number;
    message: string;
  };
}

// Create a dedicated client for the Rust MCP server
class RustMcpClient {
  private connected = false;
  private requestId = 0;
  
  constructor(private serverUrl: string) {}
  
  // Check server availability
  async checkAvailability() {
    console.error(`checking rust mcp server availability at ${this.serverUrl}...`);
    
    try {
      const response = await this.makeRequest("initialize", {
        clientInfo: {
          name: "agent-tools-server",
          version: "1.0.0"
        },
        capabilities: {}
      });
      
      if (response.result) {
        console.error("rust mcp server is available");
        this.connected = true;
        return true;
      } else {
        console.error("rust mcp server returned an error:", response.error);
        return false;
      }
    } catch (error) {
      console.error("failed to connect to rust mcp server:", error);
      return false;
    }
  }
  
  // Tool management methods
  async listTools() {
    if (!this.connected) {
      await this.checkAvailability();
    }
    
    try {
      const response = await this.makeRequest("initialize", {
        clientInfo: {
          name: "agent-tools-server",
          version: "1.0.0"
        },
        capabilities: {}
      });
      
      const tools = response.result?.capabilities?.tools?.functions || [];
      console.error(`available tools: ${tools.map((t: any) => t.name).join(", ")}`);
      
      return { tools };
    } catch (error) {
      console.error("failed to list tools:", error);
      throw error;
    }
  }
  
  async callTool(name: string, args: Record<string, any>) {
    if (!this.connected) {
      await this.checkAvailability();
    }
    
    console.error(`calling tool "${name}" with args:`, args);
    
    try {
      const response = await this.makeRequest("executeToolFunction", {
        function: name,
        arguments: args
      });
      
      if (response.error) {
        throw new Error(`Tool execution failed: ${response.error.message}`);
      }
      
      return response.result;
    } catch (error) {
      console.error(`error calling tool "${name}":`, error);
      throw error;
    }
  }
  
  // Make a JSON-RPC request to the Rust MCP server
  private async makeRequest(method: string, params: any): Promise<JsonRpcResponse> {
    const id = `request-${++this.requestId}`;
    
    try {
      const response = await fetch(this.serverUrl, {
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
      
      if (!response.ok) {
        throw new Error(`HTTP error: ${response.status} ${response.statusText}`);
      }
      
      return await response.json() as JsonRpcResponse;
    } catch (error) {
      console.error(`error making request to ${this.serverUrl}:`, error);
      throw error;
    }
  }
  
  isConnected() {
    return this.connected;
  }
}

// Create our client instance
const rustClient = new RustMcpClient(MCP_SERVER_URL);

// Initialize the MCP server
const server = new McpServer({
  name: "agent-tools-server",
  version: "1.0.0"
});

// Start the server
async function main() {
  try {
    // First check if Rust MCP server is available
    const available = await rustClient.checkAvailability();
    
    if (!available) {
      throw new Error(`Failed to connect to Rust MCP server at ${MCP_SERVER_URL}`);
    }
    
    // List available tools for logging purposes
    await rustClient.listTools();
    
    // Register agent tools
    await registerTools();
    
    // Start our agent tools server
    const transport = new StdioServerTransport();
    await server.connect(transport);
    console.error("agent tools server running on stdio transport");
  } catch (error) {
    console.error("failed to start agent tools server:", error);
    process.exit(1);
  }
}

// Register all agent tools
async function registerTools() {
  // Import and register the send-discord-message tool
  const { registerSendDiscordMessageTool } = await import('./send-discord-message.js');
  registerSendDiscordMessageTool(server, rustClient, anthropic);
  
  // Add more tool registrations here as you build them
}

// Start the server
main().catch(console.error);

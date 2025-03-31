import dotenv from "dotenv";
import path from "path";
import fs from "fs";
import { exec } from "child_process";
import { promisify } from "util";

// Create the exec promise function
const execPromise = promisify(exec);

// Load environment variables
export async function setupEnvironment() {
  // First try loading from .env file
  dotenv.config();

  // Check if API key is set
  if (!process.env.ANTHROPIC_API_KEY) {
    // Try to load from config file
    const configDir = path.join(process.env.HOME || "", ".screenpipe");
    const configPath = path.join(configDir, "config.json");

    if (fs.existsSync(configPath)) {
      try {
        const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
        if (config.anthropicApiKey) {
          process.env.ANTHROPIC_API_KEY = config.anthropicApiKey;
        }
      } catch (error) {
        console.error("error loading config:", error);
      }
    }

    // If still not set, show error
    if (!process.env.ANTHROPIC_API_KEY) {
      console.error(
        "missing ANTHROPIC_API_KEY - please set in .env file or config.json"
      );
      process.exit(1);
    }
  }
  
  // Validate API key format
  const apiKey = process.env.ANTHROPIC_API_KEY;
  if (!apiKey.startsWith('sk-ant-')) {
    console.error("\n======================================");
    console.error("invalid ANTHROPIC_API_KEY format");
    console.error("api key should start with 'sk-ant-'");
    console.error(`found: ${apiKey.substring(0, 7)}...`);
    console.error("please check your .env file or config.json");
    console.error("======================================\n");
    process.exit(1); // Exit immediately with error code
  }

  // check if rust mcp server is running
  const checkServer = async () => {
    try {
      // use the correct JSON-RPC format for MCP
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
      
      console.log("checking mcp server connection...");
      
      // Direct connection to 127.0.0.1:8080 since we've verified it works
      const curlCommand = `curl -s -X POST http://127.0.0.1:8080/mcp -H "Content-Type: application/json" -d '${JSON.stringify(payload)}'`;
      
      const { stdout, stderr } = await execPromise(curlCommand);
      
      if (stderr && stderr.length > 0) {
        console.error(`curl stderr: ${stderr}`);
        // Note: curl often writes progress info to stderr but still succeeds
        // Only fail if stdout is empty
        if (!stdout) {
          throw new Error(stderr);
        }
      }
      
      // Check if we got a valid JSON response
      try {
        const response = JSON.parse(stdout);
        if (response.result) {
          console.log("mcp server is running and responding properly");
          return true;
        }
      } catch (jsonError) {
        console.error("invalid json response from server:", stdout.substring(0, 100));
        throw new Error("Invalid JSON response from server");
      }
      
      console.log("mcp server responded but with unexpected format");
      return false;
    } catch (error) {
      console.error("failed to connect to mcp server at http://127.0.0.1:8080/mcp");
      console.error(`error details: ${error.message || error}`);
      console.error("please ensure the rust server is running");
      process.exit(1);
    }
  };
  
  await checkServer();
}
    
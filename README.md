# Computer Use AI SDK

* We've built an MCP server that controls computer

* You've heard of OpenAI's operator, you've heard of Claude's computer use. Now the open source alternative: Computer Use SDK from screenpipe.

* It's native on macOS—no virtual machine bs, no guardrails. Use it with any app or website however you want.

* No pixel-based bs—it relies on underlying desktop-rendered elements, making it much faster and far more reliable than pixel-based vision models.

* You can now build your own agents getting started with our simple Hello World Template using our MCP server and client.

* There are tools that our MCP Server provides out of the box:
    * Launch apps
    * Read content
    * Click
    * Enter text
    * Press keys

* These will be computational primitives to allow the AI to control your computer and do your tasks for you. What will you build? Come check us out at https://screenpi.pe

## Demos

get latest whatsapp messages
![Image](https://github.com/user-attachments/assets/6401c930-07e5-4459-b54c-a8c70fdca73f)

send message in imessage
![Image](https://github.com/user-attachments/assets/46e02640-7ad2-4643-b213-df03abfddba7)

open arc browser 
![Image](https://github.com/user-attachments/assets/8656be95-951d-4f13-8ee9-41babb821abb)

## Get started

```bash
git clone https://github.com/m13v/computer-use-ai-sdk.git
cd MCP-server-client-computer-use-ai-sdk
```

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Install Node.js and npm (if not already installed)
# Visit https://nodejs.org/ or use nvm

# Set your Anthropic API key as an environment variable
export ANTHROPIC_API_KEY=sk-ant-xxxx  # Replace with your actual Anthropic API key
# For Windows, use: set ANTHROPIC_API_KEY=sk-ant-xxxx
# For permanent setup, add to your shell profile (.bashrc, .zshrc, etc.)
```

```bash
# run backend server
cd mcp-server-os-level
cargo run --bin server
# keep it running
```

### Option 1: CLI Interface

```bash
# run CLI interface client in a new terminal  (good for debugging)
cd mcp-client-cli-interface
npm install  # install dependencies first
npx tsx main.ts
```

### Option 2: Web app Interface

```bash
# run CLI interface client in a new terminal  (good for debugging)
cd mcp-client-nextjs
npm install  # install dependencies first
npm run dev
# go to provided localhost web page
```


## Usage

Beginner level:

In the CLI interface, type a prompt (click to copy):

```bash
get text from Whatsapp
```

```bash
give me interactable elements from messages app and then type hello world and send
```

```bash
open arc browser
```

Intermediate level:

## Run an example chained tool

```bash
# Navigate to the server directory
cd mcp-server-chained-tools 

# Install dependencies
npm i 

# Set your API key (alternatively, create a .env file)
export ANTHROPIC_API_KEY=sk-ant...

# Build the server
npm run build
```

## Test tools through the MCP Inspector

```bash
# Install the MCP Inspector globally if you don't have it yet
npm install -g @modelcontextprotocol/inspector

# Make sure you're in the right directory (where .env is located)

# Make sure your .env file has the necessary credentials
# It should contain: ANTHROPIC_API_KEY=sk-ant...

# Run the server with the inspector
npx @modelcontextprotocol/inspector node build/server.js
```

This will launch the MCP Inspector in your browser:

1. The Inspector will connect to your server via STDIO transport
2. Click "Connect" to establish the connection 
3. Click "List Tools" to see available tools, including `send-discord-message`
4. Click on a tool to test it
5. For the Discord tool, provide the following parameters:
   - `messageType`: "dm" 
   - `recipient`: "username" (the Discord username)
   - `prompt`: "prompt to generate a message, e.g. generate a hellow world type of phrase"
6. Click "Run" to execute the tool
7. Check the logs in both the Inspector and your terminal

Note: Your server communicates via STDIO transport only, so you cannot make direct HTTP/curl requests to it. All interactions must go through the MCP Inspector or another MCP client that supports STDIO transport.

## What do I do with it?

- Build custom worfklows of agents to performs various actions
- Build custom UI to make it easy for users to automate their computer work
- Save workflow and run in cron
- Combine with other MCP servers to do something cool, e.g.: fill out a google sheet based on the history of people i talk to throughout the day

## Request features and endpoints in github issues

https://github.com/m13v/computer-use-ai-sdk/issues/new/choose
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

agent sending a message
https://github.com/user-attachments/assets/f8687500-9a8c-4a96-81b6-77562feff093

get latest whatsapp messages
![Image](https://github.com/user-attachments/assets/6401c930-07e5-4459-b54c-a8c70fdca73f)

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

# Set your Anthropic API key as an environment variable
export ANTHROPIC_API_KEY=sk-ant-xxxx  # Replace with your actual Anthropic API key
# For Windows, use: set ANTHROPIC_API_KEY=sk-ant-xxxx
# For permanent setup, add to your shell profile (.bashrc, .zshrc, etc.)

npx tsx main.ts
```

### Option 2: Web app Interface

```bash
# run CLI interface client in a new terminal  (good for debugging)
cd mcp-client-nextjs
npm install  # install dependencies first

# Set API key via command line
echo "ANTHROPIC_API_KEY=sk-ant-XXXXXXXX" > .env  # replace XXXXXXXX with your actual key
# Or append if you want to keep other env variables
# echo "ANTHROPIC_API_KEY=sk-ant-XXXXXXXX" >> .env

npm run dev
# go to provided localhost web page
```


## What do I do with it?

- Build custom worfklows of agents to performs various actions
- Build custom UI to make it easy for users to automate their computer work
- Save workflow and run in cron
- Combine with other MCP servers to do something cool, e.g.: fill out a google sheet based on the history of people i talk to throughout the day

## Request features and endpoints in github issues

https://github.com/m13v/computer-use-ai-sdk/issues/new/choose
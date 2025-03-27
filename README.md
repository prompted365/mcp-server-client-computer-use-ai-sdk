# Computer Use AI SDK

* We’ve built an MCP server that controls computer

* You’ve heard of OpenAI’s operator, you’ve heard of Claude’s computer use. Now the open source alternative: Computer Use SDK from screenpipe.

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
cd computer-use-ai-sdk
```
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Install Node.js and npm (if not already installed)
# Visit https://nodejs.org/ or use nvm
```

```bash
# run backend server
cd rust-backend
cargo run --bin server
# keep it running
```

```bash
# run frontend client in a new terminal
cd hello-world-mcp-client
npx tsx main.ts
```

## Usage

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

## What do I do with it?

- Build custom worfklows of agents to performs various actions
- Build custom UI to make it easy for users to automate their computer work
- Save workflow and run in cron
- Combine with other MCP servers to do something cool, e.g.: fill out a google sheet based on the history of people i talk to throughout the day


## Request features and endpoints in github issues

https://github.com/m13v/computer-use-ai-sdk/issues/new/choose
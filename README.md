# Computer Use AI SDK


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
open arc browser and scroll to the bottom of the page
```


## What do I do with it?

- Build custom worfklows of agents to performs various actions
- Build custom UI to make it easy for users to automate their computer work
- Save workflow and run in cron
- Combine with other MCP servers to do something cool, e.g.: fill out a google sheet based on the history of people i talk to throughout the day

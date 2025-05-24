# Tauri Plugin: Model Context Protocol (MCP)

A Tauri plugin and MCP server that allow AI Agents such as Cursor and Claude Code to debug within your tauri application.

### Features

The Tauri MCP Plugin provides a comprehensive set of tools that allow AI models and external applications to interact with Tauri applications:

#### Window Interaction
- **Take Screenshot**: Capture images of any Tauri window with configurable quality and size
- **Window Management**: Control window position, size, focus, minimize/maximize state
- **DOM Access**: Retrieve the HTML DOM content from webviews windows

#### User Input Simulation
- **Mouse Movement**: Simulate mouse clicks, movements, and scrolling
- **Text Input**: Programmatically input text into focused elements
- **Execute JavaScript**: Run arbitrary JavaScript code in the application context

#### Data & Storage
- **Local Storage Management**: Get, set, remove, and clear localStorage entries
- **Ping**: Simple connectivity testing to verify the plugin is responsive

## How to build
```bash
pnpm i 
pnpm run build && pnpm run build-plugin
```

Follow instructions at https://v2.tauri.app/start/create-project/

in src-tauri/cargo.toml add
```toml
tauri-plugin-mcp = { path = "../../tauri-plugin-mcp" }
```

In package.json
```json
    "tauri-plugin-mcp": "file:../tauri-mcp",
```

Then, register the plugin in your Tauri application:

## Only include the MCP plugin in development builds
### Take care to set the Application name correctly, this is how it identifies the window to screenshot
```rust
    #[cfg(debug_assertions)]
    {
        info!("Development build detected, enabling MCP plugin");
        tauri::Builder::default()
        .plugin(tauri_mcp::init_with_config(
         tauri_mcp::PluginConfig::new(String::new("APPLICATION_NAME")) 
                .start_socket_server(true)
                .socket_path("/tmp/tauri-mcp.sock")
        ));
    }
```

To setup MCP
```bash
cd mcp-server-ts
pnpm i
pnpm build
```

```json
{
  "mcpServers": {
      "tauri-mcp": {
          "command": "node",
          "args": [
            "$HOME/tauri-plugin-mcp/mcp-server-ts/build/index.js"
          ]
      }
  }
}
```

## Communication Between Tauri Plugin MCP Components

The Tauri MCP plugin uses an IPC socket-based architecture to expose Tauri application functionality to external clients:

### Socket Server (Rust)

The `socket_server.rs` component:

- Creates a platform-specific socket (Unix socket on macOS/Linux, named pipe on Windows)
- Listens for client connections on the socket
- Processes incoming JSON commands
- Executes Tauri API calls based on the commands
- Returns results as JSON responses

### Socket Client (TypeScript)

The `client.ts` component:

- Connects to the socket using the appropriate platform-specific path
- Provides a Promise-based API for sending commands
- Handles reconnection logic and error management
- Parses JSON responses from the server
# System Controller Architecture

## System Design

### High-Level Architecture
```
┌─────────────────┐    Network     ┌─────────────────┐
│  Remote Client  │◄──────────────►│ System Server   │
│                 │   (TCP/WS)     │                 │
└─────────────────┘                └─────────────────┘
                                           │
                                           ▼
                                   ┌─────────────────┐
                                   │ Platform Layer  │
                                   │ ┌─────┬─────┬───┤
                                   │ │Win  │macOS│Lin│
                                   │ └─────┴─────┴───┤
                                   └─────────────────┘
                                           │
                                           ▼
                                   ┌─────────────────┐
                                   │   OS APIs       │
                                   │ Mouse│Kbd│Scrn │
                                   └─────────────────┘
```

### Core Components

#### 1. Network Layer (`src/server/network.rs`)
- **TCP Server**: Main communication endpoint
- **WebSocket Upgrade**: Optional for web clients
- **Message Router**: Dispatches commands to appropriate handlers
- **Connection Manager**: Handles client connections and authentication

#### 2. Input Control (`src/server/input.rs`)
- **Mouse Controller**: Position, clicks, scrolling
- **Keyboard Controller**: Key presses, text input
- **Input Validation**: Sanitize and validate input commands
- **Rate Limiting**: Prevent input flooding

#### 3. Display Management (`src/display.rs`)
- **Screen Capture**: Multi-monitor screenshot capability
- **Display Enumeration**: List available displays
- **Image Compression**: Efficient transmission of screen data
- **Streaming**: Real-time screen updates

#### 4. Window Management (`src/window.rs`)
- **Window Enumeration**: List all windows
- **Window Information**: Title, position, process details
- **Active Window Tracking**: Monitor focus changes
- **Click-to-Window Mapping**: Identify windows under cursor

#### 5. Platform Abstraction (`src/platform/`)
- **Windows Implementation**: Win32 API integration
- **macOS Implementation**: Core Graphics and Cocoa
- **Linux Implementation**: X11 and Wayland support
- **Unified Interface**: Common API across platforms

## Data Flow

### Command Processing
```
Client Request → Network Layer → Protocol Parser → Command Router → Platform Handler → OS API → Response
```

### Screen Capture Flow
```
Capture Request → Display Manager → Platform Capture → Image Compression → Network Transmission
```

## Protocol Specification

### Message Format
```json
{
  "id": "unique_request_id",
  "type": "command_type",
  "payload": {
    // Command-specific data
  },
  "timestamp": "2025-08-18T10:30:00Z"
}
```

### Command Types

#### Mouse Commands
```json
{
  "type": "mouse_move",
  "payload": { "x": 100, "y": 200 }
}

{
  "type": "mouse_click", 
  "payload": { "button": "left", "x": 100, "y": 200 }
}

{
  "type": "mouse_scroll",
  "payload": { "x": 0, "y": -3 }
}
```

#### Keyboard Commands
```json
{
  "type": "key_press",
  "payload": { "key": "Enter" }
}

{
  "type": "type_text",
  "payload": { "text": "Hello World" }
}
```

#### Display Commands
```json
{
  "type": "capture_screen",
  "payload": { "display_id": 0, "format": "png" }
}

{
  "type": "get_displays",
  "payload": {}
}
```

#### Window Commands
```json
{
  "type": "get_window_info",
  "payload": { "x": 100, "y": 200 }
}

{
  "type": "list_windows",
  "payload": {}
}
```

## Security Architecture

### Authentication Flow
```
1. Client connects → 2. Server sends challenge → 3. Client provides token → 4. Server validates → 5. Session established
```

### Security Layers
- **TLS Encryption**: All network communication encrypted
- **Token Authentication**: JWT or similar token-based auth
- **Rate Limiting**: Per-client command rate limits
- **Access Control**: Configurable permissions per client
- **Audit Logging**: All commands logged with client identification

## Configuration

### Server Configuration
```toml
[server]
bind_address = "0.0.0.0"
port = 8080
enable_tls = true
cert_path = "/path/to/cert.pem"
key_path = "/path/to/key.pem"

[security]
enable_auth = true
token_expiry = 3600
max_connections = 10
rate_limit_per_second = 100

[logging]
level = "info"
audit_file = "/var/log/system-controller-audit.log"
```

## Platform-Specific Considerations

### Windows
- **Dependencies**: `winapi`, `windows-rs`
- **Permissions**: May require admin privileges for some operations
- **APIs**: Win32 for input, GDI+ for screen capture

### macOS
- **Dependencies**: `core-graphics`, `cocoa`
- **Permissions**: Accessibility permissions required
- **APIs**: Core Graphics for display, Cocoa for window management

### Linux
- **Dependencies**: `x11`, `wayland-client`
- **Display Servers**: Support both X11 and Wayland
- **APIs**: X11/XInput for input, X11/Wayland for screen capture

## Performance Considerations

### Screen Capture Optimization
- **Differential Updates**: Only send changed regions
- **Compression**: PNG/JPEG compression for still images
- **Frame Rate Control**: Configurable capture frequency
- **Resolution Scaling**: Option to reduce resolution for bandwidth

### Memory Management
- **Streaming**: Process large images in chunks
- **Buffer Pooling**: Reuse image buffers
- **Garbage Collection**: Efficient cleanup of temporary data

## Error Handling

### Error Categories
1. **Network Errors**: Connection failures, timeouts
2. **Permission Errors**: Insufficient privileges
3. **Platform Errors**: OS-specific failures
4. **Protocol Errors**: Malformed commands
5. **Security Errors**: Authentication failures

### Error Response Format
```json
{
  "id": "request_id",
  "type": "error",
  "payload": {
    "code": "ERROR_CODE", 
    "message": "Human readable error",
    "details": {}
  }
}
```
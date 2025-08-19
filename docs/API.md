# System Controller API Documentation

This document provides comprehensive documentation for the System Controller API, including protocol specification, command reference, and integration examples.

## Table of Contents

- [Protocol Overview](#protocol-overview)
- [Connection Management](#connection-management)
- [Authentication](#authentication)
- [Command Reference](#command-reference)
- [Response Formats](#response-formats)
- [Error Handling](#error-handling)
- [Rate Limiting](#rate-limiting)
- [WebSocket Support](#websocket-support)
- [Binary Data Transfer](#binary-data-transfer)
- [Client Examples](#client-examples)

## Protocol Overview

The System Controller API uses a JSON-based protocol over TCP or WebSocket connections. All communications are secured with TLS encryption and require JWT-based authentication.

### Protocol Characteristics
- **Transport**: TCP with optional WebSocket upgrade
- **Format**: JSON for commands and responses, binary for screen capture data
- **Encoding**: UTF-8 for text, little-endian for binary data
- **Authentication**: JWT bearer tokens with configurable expiration
- **Compression**: Optional gzip compression for large payloads

### Message Structure

All messages follow a consistent structure:

```json
{
  "id": "unique-command-id",
  "type": "command_type",
  "payload": { /* command-specific data */ },
  "timestamp": "ISO8601-timestamp",
  "auth_token": "jwt-bearer-token"
}
```

## Connection Management

### TCP Connection

```python
import socket
import ssl
import json

# Create secure TCP connection
context = ssl.create_default_context()
sock = socket.create_connection(('localhost', 8080))
secure_sock = context.wrap_socket(sock, server_hostname='localhost')

def send_command(command):
    message = json.dumps(command).encode('utf-8')
    secure_sock.send(len(message).to_bytes(4, 'little') + message)

def receive_response():
    length = int.from_bytes(secure_sock.recv(4), 'little')
    response = secure_sock.recv(length).decode('utf-8')
    return json.loads(response)
```

### WebSocket Connection

```javascript
const WebSocket = require('ws');

const ws = new WebSocket('wss://localhost:8080', {
  rejectUnauthorized: false // Only for self-signed certificates
});

ws.on('open', () => {
  console.log('Connected to System Controller');
});

ws.on('message', (data) => {
  const response = JSON.parse(data.toString());
  console.log('Received:', response);
});

function sendCommand(command) {
  ws.send(JSON.stringify(command));
}
```

## Authentication

### Login Flow

#### 1. Authentication Request

```json
{
  "id": "auth-001",
  "type": "authenticate",
  "payload": {
    "username": "admin",
    "password": "your-password"
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### 2. Authentication Response

**Success**:
```json
{
  "command_id": "auth-001",
  "status": "success",
  "data": {
    "type": "auth_success",
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbiIsImV4cCI6MTcwNTMzMDIwMCwiaWF0IjoxNzA1MzI2NjAwLCJqdGkiOiI1ZjlkNGVlNi0yMzQ1LTRhYmMtODkwMC0xMjM0NTY3ODkwYWIifQ.signature",
    "expires_at": "2024-01-15T11:30:00Z",
    "permissions": ["input_control", "screen_capture", "window_management"]
  },
  "timestamp": "2024-01-15T10:30:01Z"
}
```

**Failure**:
```json
{
  "command_id": "auth-001",
  "status": "error",
  "error": "Invalid username or password",
  "timestamp": "2024-01-15T10:30:01Z"
}
```

### Token Management

#### Token Refresh

```json
{
  "id": "refresh-001",
  "type": "refresh_token",
  "payload": {
    "refresh_token": "your-refresh-token"
  },
  "timestamp": "2024-01-15T11:25:00Z"
}
```

#### Token Validation

Include the JWT token in all subsequent requests:

```json
{
  "id": "validate-001",
  "type": "validate_token",
  "payload": {},
  "timestamp": "2024-01-15T10:35:00Z",
  "auth_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
}
```

## Command Reference

### Input Control Commands

#### Mouse Movement

```json
{
  "id": "mouse-move-001",
  "type": "mouse_move",
  "payload": {
    "type": "mouse_move",
    "x": 100,
    "y": 200
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

**Parameters**:
- `x` (integer): X coordinate (0 to screen width)
- `y` (integer): Y coordinate (0 to screen height)

#### Mouse Click

```json
{
  "id": "mouse-click-001",
  "type": "mouse_click",
  "payload": {
    "type": "mouse_click",
    "button": "Left",
    "x": 150,
    "y": 250
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

**Parameters**:
- `button` (string): Mouse button ("Left", "Right", "Middle")
- `x` (integer): X coordinate for click
- `y` (integer): Y coordinate for click

#### Mouse Scroll

```json
{
  "id": "mouse-scroll-001",
  "type": "mouse_scroll",
  "payload": {
    "type": "mouse_scroll",
    "x": 0,
    "y": -3
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

**Parameters**:
- `x` (integer): Horizontal scroll amount
- `y` (integer): Vertical scroll amount (positive = up, negative = down)

#### Key Press

```json
{
  "id": "key-press-001",
  "type": "key_press",
  "payload": {
    "type": "key_press",
    "key": "Enter"
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

**Parameters**:
- `key` (string): Key name (see [Key Names](#key-names) section)

#### Key Release

```json
{
  "id": "key-release-001",
  "type": "key_release",
  "payload": {
    "type": "key_release",
    "key": "Enter"
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

#### Type Text

```json
{
  "id": "type-text-001",
  "type": "type_text",
  "payload": {
    "type": "type_text",
    "text": "Hello, World!"
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

**Parameters**:
- `text` (string): Text to type (max 1000 characters)

### Display Management Commands

#### Capture Screen

```json
{
  "id": "capture-001",
  "type": "capture_screen",
  "payload": {
    "type": "capture_screen",
    "display_id": 0,
    "format": "png",
    "quality": 85
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

**Parameters**:
- `display_id` (integer): Display identifier (0 for primary)
- `format` (string, optional): Image format ("png", "jpeg") - default: "png"
- `quality` (integer, optional): JPEG quality 1-100 - default: 85

#### Get Displays

```json
{
  "id": "displays-001",
  "type": "get_displays",
  "payload": {
    "type": "get_displays"
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

### Window Management Commands

#### Get Window at Position

```json
{
  "id": "window-at-001",
  "type": "get_window_info",
  "payload": {
    "type": "get_window_info",
    "x": 100,
    "y": 200
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

**Parameters**:
- `x` (integer): X coordinate to check
- `y` (integer): Y coordinate to check

#### List All Windows

```json
{
  "id": "list-windows-001",
  "type": "list_windows",
  "payload": {
    "type": "list_windows",
    "include_minimized": true
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

**Parameters**:
- `include_minimized` (boolean, optional): Include minimized windows - default: true

### System Information Commands

#### Get System Status

```json
{
  "id": "status-001",
  "type": "system_status",
  "payload": {
    "type": "system_status"
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

#### Get Performance Metrics

```json
{
  "id": "metrics-001",
  "type": "performance_metrics",
  "payload": {
    "type": "performance_metrics",
    "include_history": false
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

## Response Formats

### Standard Response Structure

```json
{
  "command_id": "original-command-id",
  "status": "success|error",
  "error": "error-message-if-failed",
  "data": { /* response-specific data */ },
  "timestamp": "2024-01-15T10:30:01Z",
  "processing_time_ms": 15
}
```

### Success Response Examples

#### Input Command Success

```json
{
  "command_id": "mouse-move-001",
  "status": "success",
  "data": {
    "type": "operation_complete",
    "operation": "mouse_move",
    "coordinates": {"x": 100, "y": 200}
  },
  "timestamp": "2024-01-15T10:30:01Z",
  "processing_time_ms": 5
}
```

#### Display Information Response

```json
{
  "command_id": "displays-001",
  "status": "success",
  "data": {
    "type": "display_info",
    "displays": [
      {
        "id": 0,
        "name": "Primary Display",
        "width": 1920,
        "height": 1080,
        "x": 0,
        "y": 0,
        "is_primary": true,
        "scale_factor": 1.0
      },
      {
        "id": 1,
        "name": "Secondary Display",
        "width": 1680,
        "height": 1050,
        "x": 1920,
        "y": 0,
        "is_primary": false,
        "scale_factor": 1.0
      }
    ]
  },
  "timestamp": "2024-01-15T10:30:01Z",
  "processing_time_ms": 12
}
```

#### Window Information Response

```json
{
  "command_id": "list-windows-001",
  "status": "success",
  "data": {
    "type": "window_info",
    "windows": [
      {
        "id": 12345,
        "title": "Browser Window - Example.com",
        "x": 100,
        "y": 100,
        "width": 800,
        "height": 600,
        "process_name": "chrome.exe",
        "is_visible": true,
        "is_minimized": false,
        "z_order": 1
      }
    ]
  },
  "timestamp": "2024-01-15T10:30:01Z",
  "processing_time_ms": 8
}
```

#### Screen Capture Response

```json
{
  "command_id": "capture-001",
  "status": "success",
  "data": {
    "type": "screen_capture",
    "display_id": 0,
    "format": "png",
    "width": 1920,
    "height": 1080,
    "data_size": 524288,
    "compression_ratio": 0.15,
    "capture_time_ms": 45
  },
  "timestamp": "2024-01-15T10:30:01Z",
  "processing_time_ms": 52
}
```

*Note: Binary image data follows immediately after this JSON response*

### Error Response Examples

#### Authentication Error

```json
{
  "command_id": "mouse-move-001",
  "status": "error",
  "error": "Authentication required",
  "error_code": "AUTH_REQUIRED",
  "timestamp": "2024-01-15T10:30:01Z"
}
```

#### Permission Error

```json
{
  "command_id": "capture-001",
  "status": "error",
  "error": "Insufficient permissions for screen capture",
  "error_code": "PERMISSION_DENIED",
  "required_permission": "screen_capture",
  "timestamp": "2024-01-15T10:30:01Z"
}
```

#### Validation Error

```json
{
  "command_id": "mouse-move-001",
  "status": "error",
  "error": "Invalid coordinates: x=-100, y=-50",
  "error_code": "VALIDATION_ERROR",
  "validation_errors": [
    {
      "field": "x",
      "message": "X coordinate must be non-negative"
    },
    {
      "field": "y", 
      "message": "Y coordinate must be non-negative"
    }
  ],
  "timestamp": "2024-01-15T10:30:01Z"
}
```

## Error Handling

### Error Codes

| Code | Description | Category |
|------|-------------|----------|
| `AUTH_REQUIRED` | Authentication token required | Authentication |
| `AUTH_INVALID` | Invalid or expired token | Authentication |
| `AUTH_EXPIRED` | Token has expired | Authentication |
| `PERMISSION_DENIED` | Insufficient permissions | Authorization |
| `RATE_LIMIT_EXCEEDED` | Too many requests | Rate Limiting |
| `VALIDATION_ERROR` | Invalid request parameters | Validation |
| `PLATFORM_ERROR` | Platform-specific operation failed | Platform |
| `RESOURCE_UNAVAILABLE` | Requested resource not available | Resource |
| `INTERNAL_ERROR` | Internal server error | System |
| `CONNECTION_ERROR` | Connection-related error | Network |

### Error Response Structure

```json
{
  "command_id": "original-id",
  "status": "error",
  "error": "Human-readable error message",
  "error_code": "MACHINE_READABLE_CODE",
  "details": {
    /* Additional error-specific information */
  },
  "timestamp": "2024-01-15T10:30:01Z",
  "retry_after_ms": 1000  // Optional: suggested retry delay
}
```

## Rate Limiting

### Rate Limit Headers

Rate limiting information is included in response headers:

```json
{
  "command_id": "mouse-move-001",
  "status": "success",
  "data": { /* response data */ },
  "rate_limit": {
    "limit": 100,
    "remaining": 95,
    "reset_at": "2024-01-15T10:31:00Z",
    "retry_after_ms": null
  },
  "timestamp": "2024-01-15T10:30:01Z"
}
```

### Rate Limit Exceeded

```json
{
  "command_id": "mouse-move-100",
  "status": "error",
  "error": "Rate limit exceeded",
  "error_code": "RATE_LIMIT_EXCEEDED",
  "rate_limit": {
    "limit": 100,
    "remaining": 0,
    "reset_at": "2024-01-15T10:31:00Z",
    "retry_after_ms": 5000
  },
  "timestamp": "2024-01-15T10:30:01Z"
}
```

## WebSocket Support

### Connection Upgrade

```http
GET / HTTP/1.1
Host: localhost:8080
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: x3JJHMbDL1EzLkh9GBhXDw==
Sec-WebSocket-Version: 13
```

### WebSocket-Specific Features

#### Heartbeat/Ping

```json
{
  "id": "ping-001",
  "type": "ping",
  "payload": {},
  "timestamp": "2024-01-15T10:30:00Z"
}
```

Response:
```json
{
  "command_id": "ping-001",
  "status": "success",
  "data": {
    "type": "pong",
    "server_time": "2024-01-15T10:30:01Z"
  },
  "timestamp": "2024-01-15T10:30:01Z"
}
```

#### Subscription to Events

```json
{
  "id": "subscribe-001",
  "type": "subscribe",
  "payload": {
    "events": ["window_focus_changed", "display_configuration_changed"]
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

Event notifications:
```json
{
  "type": "event",
  "event_type": "window_focus_changed",
  "data": {
    "previous_window": {"id": 12345, "title": "Old Window"},
    "current_window": {"id": 67890, "title": "New Window"}
  },
  "timestamp": "2024-01-15T10:31:00Z"
}
```

## Binary Data Transfer

### Screen Capture Data

Screen capture responses include binary image data following the JSON response:

1. **JSON Response**: Contains metadata about the capture
2. **Binary Data**: Raw image data in specified format

#### Reading Binary Data

```python
import json
import struct

def receive_screen_capture(sock):
    # Read JSON response first
    json_length = struct.unpack('<I', sock.recv(4))[0]
    json_data = json.loads(sock.recv(json_length).decode('utf-8'))
    
    if json_data['status'] == 'success' and 'screen_capture' in json_data['data']['type']:
        # Read binary image data
        data_size = json_data['data']['data_size']
        image_data = sock.recv(data_size)
        
        return json_data, image_data
    
    return json_data, None
```

### Compression

Binary data can be optionally compressed using gzip:

```json
{
  "id": "capture-001",
  "type": "capture_screen",
  "payload": {
    "type": "capture_screen",
    "display_id": 0,
    "format": "png",
    "compress": true
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

Response includes compression info:
```json
{
  "data": {
    "type": "screen_capture",
    "compressed": true,
    "compression_type": "gzip",
    "original_size": 2097152,
    "compressed_size": 524288,
    "compression_ratio": 0.25
  }
}
```

## Key Names

### Special Keys
- `Enter`, `Return`
- `Tab`
- `Space`, `Spacebar`
- `Backspace`
- `Delete`
- `Escape`, `Esc`
- `Home`, `End`
- `PageUp`, `PageDown`
- `Insert`

### Arrow Keys
- `Up`, `Down`, `Left`, `Right`
- `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`

### Function Keys
- `F1` through `F24`

### Modifier Keys
- `Shift`, `LeftShift`, `RightShift`
- `Control`, `Ctrl`, `LeftControl`, `RightControl`
- `Alt`, `LeftAlt`, `RightAlt`
- `Meta`, `Cmd`, `Windows`, `Super`

### Alphanumeric Keys
- `A` through `Z` (case insensitive)
- `0` through `9`

### Punctuation and Symbols
- `Comma`, `Period`, `Semicolon`, `Quote`
- `Minus`, `Plus`, `Equals`
- `LeftBracket`, `RightBracket`
- `Backslash`, `Slash`
- `Backtick`, `Tilde`

## Client Examples

### Python Client

```python
import json
import socket
import ssl
import time
from datetime import datetime

class SystemControllerClient:
    def __init__(self, host='localhost', port=8080):
        self.host = host
        self.port = port
        self.sock = None
        self.auth_token = None
        
    def connect(self):
        context = ssl.create_default_context()
        context.check_hostname = False
        context.verify_mode = ssl.CERT_NONE  # Only for development
        
        sock = socket.create_connection((self.host, self.port))
        self.sock = context.wrap_socket(sock, server_hostname=self.host)
        
    def send_command(self, command_type, payload, command_id=None):
        if not command_id:
            command_id = f"{command_type}-{int(time.time())}"
            
        command = {
            "id": command_id,
            "type": command_type,
            "payload": payload,
            "timestamp": datetime.now().isoformat() + "Z"
        }
        
        if self.auth_token:
            command["auth_token"] = self.auth_token
            
        message = json.dumps(command).encode('utf-8')
        length = len(message).to_bytes(4, 'little')
        
        self.sock.send(length + message)
        return self.receive_response()
    
    def receive_response(self):
        length = int.from_bytes(self.sock.recv(4), 'little')
        response = self.sock.recv(length).decode('utf-8')
        return json.loads(response)
    
    def authenticate(self, username, password):
        response = self.send_command("authenticate", {
            "username": username,
            "password": password
        })
        
        if response['status'] == 'success':
            self.auth_token = response['data']['token']
            return True
        return False
    
    def mouse_move(self, x, y):
        return self.send_command("mouse_move", {
            "type": "mouse_move",
            "x": x,
            "y": y
        })
    
    def mouse_click(self, button, x, y):
        return self.send_command("mouse_click", {
            "type": "mouse_click", 
            "button": button,
            "x": x,
            "y": y
        })
    
    def type_text(self, text):
        return self.send_command("type_text", {
            "type": "type_text",
            "text": text
        })
    
    def capture_screen(self, display_id=0):
        return self.send_command("capture_screen", {
            "type": "capture_screen",
            "display_id": display_id
        })
    
    def get_displays(self):
        return self.send_command("get_displays", {
            "type": "get_displays"
        })
    
    def disconnect(self):
        if self.sock:
            self.sock.close()

# Usage example
if __name__ == "__main__":
    client = SystemControllerClient()
    client.connect()
    
    if client.authenticate("admin", "changeme123!"):
        print("Authentication successful")
        
        # Get display information
        displays = client.get_displays()
        print("Displays:", displays)
        
        # Move mouse and click
        client.mouse_move(100, 100)
        time.sleep(0.1)
        client.mouse_click("Left", 100, 100)
        
        # Type some text
        client.type_text("Hello from Python!")
        
    client.disconnect()
```

### Node.js WebSocket Client

```javascript
const WebSocket = require('ws');
const { v4: uuidv4 } = require('uuid');

class SystemControllerClient {
    constructor(url = 'wss://localhost:8080') {
        this.url = url;
        this.ws = null;
        this.authToken = null;
        this.pendingCommands = new Map();
    }
    
    connect() {
        return new Promise((resolve, reject) => {
            this.ws = new WebSocket(this.url, {
                rejectUnauthorized: false // Only for development
            });
            
            this.ws.on('open', () => {
                console.log('Connected to System Controller');
                resolve();
            });
            
            this.ws.on('error', reject);
            
            this.ws.on('message', (data) => {
                const response = JSON.parse(data.toString());
                const commandId = response.command_id;
                
                if (this.pendingCommands.has(commandId)) {
                    const { resolve, reject } = this.pendingCommands.get(commandId);
                    this.pendingCommands.delete(commandId);
                    
                    if (response.status === 'success') {
                        resolve(response);
                    } else {
                        reject(new Error(response.error));
                    }
                }
            });
        });
    }
    
    sendCommand(commandType, payload, commandId = null) {
        return new Promise((resolve, reject) => {
            if (!commandId) {
                commandId = `${commandType}-${uuidv4()}`;
            }
            
            const command = {
                id: commandId,
                type: commandType,
                payload: payload,
                timestamp: new Date().toISOString()
            };
            
            if (this.authToken) {
                command.auth_token = this.authToken;
            }
            
            this.pendingCommands.set(commandId, { resolve, reject });
            
            this.ws.send(JSON.stringify(command));
            
            // Timeout after 30 seconds
            setTimeout(() => {
                if (this.pendingCommands.has(commandId)) {
                    this.pendingCommands.delete(commandId);
                    reject(new Error('Command timeout'));
                }
            }, 30000);
        });
    }
    
    async authenticate(username, password) {
        try {
            const response = await this.sendCommand('authenticate', {
                username: username,
                password: password
            });
            
            this.authToken = response.data.token;
            return true;
        } catch (error) {
            console.error('Authentication failed:', error.message);
            return false;
        }
    }
    
    async mouseMove(x, y) {
        return this.sendCommand('mouse_move', {
            type: 'mouse_move',
            x: x,
            y: y
        });
    }
    
    async mouseClick(button, x, y) {
        return this.sendCommand('mouse_click', {
            type: 'mouse_click',
            button: button,
            x: x,
            y: y
        });
    }
    
    async typeText(text) {
        return this.sendCommand('type_text', {
            type: 'type_text',
            text: text
        });
    }
    
    async captureScreen(displayId = 0) {
        return this.sendCommand('capture_screen', {
            type: 'capture_screen',
            display_id: displayId
        });
    }
    
    async getDisplays() {
        return this.sendCommand('get_displays', {
            type: 'get_displays'
        });
    }
    
    disconnect() {
        if (this.ws) {
            this.ws.close();
        }
    }
}

// Usage example
async function main() {
    const client = new SystemControllerClient();
    
    try {
        await client.connect();
        
        if (await client.authenticate('admin', 'changeme123!')) {
            console.log('Authentication successful');
            
            // Get display information
            const displays = await client.getDisplays();
            console.log('Displays:', displays);
            
            // Move mouse and click
            await client.mouseMove(100, 100);
            await new Promise(resolve => setTimeout(resolve, 100));
            await client.mouseClick('Left', 100, 100);
            
            // Type some text
            await client.typeText('Hello from Node.js!');
        }
    } catch (error) {
        console.error('Error:', error);
    } finally {
        client.disconnect();
    }
}

main();
```

### C# Client

```csharp
using System;
using System.Net.WebSockets;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using System.Collections.Generic;

public class SystemControllerClient
{
    private ClientWebSocket _webSocket;
    private string _authToken;
    private readonly string _url;
    
    public SystemControllerClient(string url = "wss://localhost:8080")
    {
        _url = url;
        _webSocket = new ClientWebSocket();
    }
    
    public async Task ConnectAsync()
    {
        await _webSocket.ConnectAsync(new Uri(_url), CancellationToken.None);
    }
    
    public async Task<JsonDocument> SendCommandAsync(string commandType, object payload, string commandId = null)
    {
        commandId ??= $"{commandType}-{DateTimeOffset.UtcNow.ToUnixTimeMilliseconds()}";
        
        var command = new
        {
            id = commandId,
            type = commandType,
            payload = payload,
            timestamp = DateTime.UtcNow.ToString("yyyy-MM-ddTHH:mm:ssZ"),
            auth_token = _authToken
        };
        
        var json = JsonSerializer.Serialize(command);
        var bytes = Encoding.UTF8.GetBytes(json);
        
        await _webSocket.SendAsync(
            new ArraySegment<byte>(bytes),
            WebSocketMessageType.Text,
            true,
            CancellationToken.None);
        
        return await ReceiveResponseAsync();
    }
    
    private async Task<JsonDocument> ReceiveResponseAsync()
    {
        var buffer = new byte[4096];
        var result = await _webSocket.ReceiveAsync(
            new ArraySegment<byte>(buffer),
            CancellationToken.None);
        
        var json = Encoding.UTF8.GetString(buffer, 0, result.Count);
        return JsonDocument.Parse(json);
    }
    
    public async Task<bool> AuthenticateAsync(string username, string password)
    {
        var response = await SendCommandAsync("authenticate", new
        {
            username = username,
            password = password
        });
        
        if (response.RootElement.GetProperty("status").GetString() == "success")
        {
            _authToken = response.RootElement
                .GetProperty("data")
                .GetProperty("token")
                .GetString();
            return true;
        }
        
        return false;
    }
    
    public async Task<JsonDocument> MouseMoveAsync(int x, int y)
    {
        return await SendCommandAsync("mouse_move", new
        {
            type = "mouse_move",
            x = x,
            y = y
        });
    }
    
    public async Task<JsonDocument> MouseClickAsync(string button, int x, int y)
    {
        return await SendCommandAsync("mouse_click", new
        {
            type = "mouse_click",
            button = button,
            x = x,
            y = y
        });
    }
    
    public async Task<JsonDocument> TypeTextAsync(string text)
    {
        return await SendCommandAsync("type_text", new
        {
            type = "type_text",
            text = text
        });
    }
    
    public void Disconnect()
    {
        _webSocket?.Dispose();
    }
}

// Usage example
class Program
{
    static async Task Main(string[] args)
    {
        var client = new SystemControllerClient();
        
        try
        {
            await client.ConnectAsync();
            
            if (await client.AuthenticateAsync("admin", "changeme123!"))
            {
                Console.WriteLine("Authentication successful");
                
                // Move mouse and click
                await client.MouseMoveAsync(100, 100);
                await Task.Delay(100);
                await client.MouseClickAsync("Left", 100, 100);
                
                // Type some text
                await client.TypeTextAsync("Hello from C#!");
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error: {ex.Message}");
        }
        finally
        {
            client.Disconnect();
        }
    }
}
```

This comprehensive API documentation provides all the necessary information for integrating with the System Controller server, including detailed command references, response formats, error handling, and complete client implementation examples in multiple programming languages.
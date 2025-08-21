# System Controller WebSocket Client

A feature-rich WebSocket client for the System Controller with beautiful CLI interface, real-time monitoring, and comprehensive testing capabilities.

## 🚀 Quick Start

```bash
# Install dependencies
npm install

# Run demo (default)
npm start

# Or use specific modes
npm run demo         # Full feature demonstration
npm run interactive  # Interactive command mode
npm run monitor      # Real-time system monitoring
npm run benchmark    # Performance benchmarking
```

## 📦 Features

- **🎨 Beautiful CLI Interface** - Colored output, progress indicators, and formatted tables
- **🔄 Automatic Reconnection** - Robust connection handling with exponential backoff
- **📊 Real-time Monitoring** - System health monitoring with performance metrics
- **🏃 Performance Benchmarking** - Comprehensive performance testing suite
- **🎮 Interactive Mode** - Manual command execution with tab completion
- **📈 Metrics Tracking** - Built-in performance and usage analytics
- **⚙️ Configurable** - Flexible configuration via JSON files and CLI options

## 🎯 Available Modes

### Demo Mode (Default)
```bash
npm start
# or
npm run demo
```
Runs a comprehensive demonstration of all System Controller features including:
- Connection and authentication
- Mouse and keyboard operations
- Display and window management
- Screen capture
- Performance metrics

### Interactive Mode
```bash
npm run interactive
```
Interactive command-line interface for manual control:
```
📝 Command: move 100 200        # Move mouse to coordinates
📝 Command: click Left          # Click mouse button  
📝 Command: type Hello World    # Type text
📝 Command: key Enter          # Press key
📝 Command: capture            # Take screenshot
📝 Command: displays           # List displays
📝 Command: windows            # List windows
📝 Command: help               # Show all commands
```

### Monitor Mode
```bash
npm run monitor
# Monitor every 10 seconds
node client.js monitor --interval 10
```
Real-time system monitoring with:
- System responsiveness checks
- Display configuration monitoring
- Performance metrics tracking
- Health status indicators

### Benchmark Mode
```bash
npm run benchmark
# Run 200 iterations per test
node client.js benchmark --iterations 200
```
Performance testing suite measuring:
- Mouse operation latency
- Keyboard operation speed
- Display query performance  
- Network round-trip time
- Success rates and error handling

## ⚙️ Configuration

### Command Line Options
```bash
node client.js [command] [options]

Options:
  -h, --host <host>      Server hostname (default: localhost)
  -p, --port <port>      Server port (default: 8080)
  --no-tls              Disable TLS encryption
  --verify-ssl          Verify SSL certificates
  -u, --username <user>  Username for authentication
  -P, --password <pass>  Password for authentication
  -v, --verbose         Enable verbose logging
  -q, --quiet          Suppress output
```

### Configuration File
Edit `config/default.json` to customize default settings:

```json
{
  "server": {
    "host": "localhost",
    "port": 8080,
    "use_tls": true,
    "verify_ssl": false
  },
  "auth": {
    "username": "admin",
    "password": "changeme123!"
  },
  "client": {
    "reconnect": true,
    "reconnect_interval": 1000,
    "max_reconnect_attempts": 5,
    "command_timeout": 30000
  }
}
```

### Environment Variables
```bash
export SYSTEM_CONTROLLER_HOST=remote-server.com
export SYSTEM_CONTROLLER_PORT=9090
export SYSTEM_CONTROLLER_USERNAME=operator
export SYSTEM_CONTROLLER_PASSWORD=secret
```

## 📋 Interactive Commands

| Command | Arguments | Description |
|---------|-----------|-------------|
| `move` | X Y | Move mouse to coordinates |
| `click` | BUTTON | Click mouse button (Left/Right/Middle) |
| `type` | TEXT | Type text string |
| `key` | KEY | Press key (Enter, Space, Tab, etc.) |
| `capture` | | Capture screenshot |
| `displays` | | List available displays |
| `windows` | | List open windows |
| `ping` | | Test server connection |
| `metrics` | | Show client performance metrics |
| `clear` | | Clear screen |
| `help` | | Show command help |
| `quit` | | Exit interactive mode |

## 📊 Sample Output

### Demo Mode
```
🚀 System Controller WebSocket Client Demo
============================================================
✅ Connected to System Controller
✅ Authenticated as admin

💓 Testing Connection Health
✅ Heartbeat: 2024-01-15T10:30:01.234Z

📊 System Information
✅ Found 2 display(s)

┌─────┬────────────────────┬───────────────┬───────────────┬─────────┐
│ ID  │ Name               │ Resolution    │ Position      │ Primary │
├─────┼────────────────────┼───────────────┼───────────────┼─────────┤
│ 0   │ Primary Display    │ 1920×1080     │ (0, 0)        │ ✓       │
│ 1   │ Secondary Display  │ 1680×1050     │ (1920, 0)     │         │
└─────┴────────────────────┴───────────────┴───────────────┴─────────┘

🖱️  Mouse Operations Demo
✅ Moving to (150, 150)
✅ Left click at (150, 150)
✅ Scrolling down

⌨️  Keyboard Operations Demo
✅ Pressing key "a"
✅ Typing text
✅ Pressing Enter
```

### Monitor Mode
```
📊 System Controller Real-time Monitor
Monitoring interval: 5 seconds
Press Ctrl+C to stop
==================================================

📊 Monitor Update #1 - 10:30:15
--------------------------------------------------
✅ System responsive (45ms)
   Displays: 2
   Primary: Primary Display (1920×1080)
   Commands: 1 sent, 100% success
   Performance: 45ms avg response
   Uptime: 15s
   Health: Excellent
```

### Benchmark Results
```
📊 Benchmark Results
================================================================================
┌───────────────┬────────────┬──────────┬──────────┬──────────┬──────────┬──────────────┐
│ Test          │ Iterations │ Avg (ms) │ Min (ms) │ Max (ms) │ P95 (ms) │ Success %    │
├───────────────┼────────────┼──────────┼──────────┼──────────┼──────────┼──────────────┤
│ Mouse Move    │ 100        │ 12.3     │ 8        │ 45       │ 18       │ 100.0%       │
│ Key Press     │ 100        │ 15.7     │ 10       │ 38       │ 22       │ 100.0%       │
│ Get Displays  │ 50         │ 28.4     │ 20       │ 65       │ 42       │ 100.0%       │
│ Type Text     │ 25         │ 35.2     │ 25       │ 78       │ 55       │ 100.0%       │
│ Ping          │ 100        │ 8.1      │ 5        │ 25       │ 12       │ 100.0%       │
└───────────────┴────────────┴──────────┴──────────┴──────────┴──────────┴──────────────┘
```

## 🔧 Development

### Project Structure
```
websocket-client/
├── package.json           # NPM configuration
├── client.js             # Main CLI application
├── lib/
│   └── SystemControllerClient.js  # Core client library
├── config/
│   └── default.json      # Default configuration
├── tests/
│   └── client.test.js    # Unit tests
└── README.md            # This file
```

### Running Tests
```bash
# Run unit tests
npm test

# Run with coverage
npm run test:coverage

# Run linting
npm run lint
```

### Adding New Commands

1. Add the command handler to `client.js`:
```javascript
} else if (cmd === 'newcommand' && parts.length >= 2) {
  await client.newOperation(parts[1]);
  console.log(chalk.green('✅ New command executed'));
```

2. Add the method to `SystemControllerClient.js`:
```javascript
async newOperation(parameter) {
  return this.sendCommand('new_operation', {
    type: 'new_operation',
    parameter: parameter
  });
}
```

3. Update the help text in the `showHelp()` function.

### Error Handling

The client includes comprehensive error handling:
- Connection failures with automatic retry
- Command timeouts with configurable limits
- Authentication errors with clear messages
- Network errors with reconnection logic

### Performance Optimization

For high-performance scenarios:
```json
{
  "client": {
    "command_timeout": 5000,
    "reconnect_interval": 500,
    "enable_compression": true
  }
}
```

## 🐛 Troubleshooting

### Connection Issues
```bash
# Test with verbose logging
node client.js --verbose

# Disable TLS for testing
node client.js --no-tls

# Test specific host/port
node client.js --host 192.168.1.100 --port 9090
```

### Authentication Problems
```bash
# Use different credentials
node client.js --username user --password pass

# Check server logs for authentication errors
```

### Performance Issues
```bash
# Run benchmark to identify bottlenecks
npm run benchmark

# Monitor in real-time
npm run monitor
```

### SSL Certificate Issues
```bash
# Disable SSL verification for self-signed certificates
node client.js --no-verify-ssl

# Or set in config:
{
  "server": {
    "verify_ssl": false
  }
}
```

## 📄 License

This WebSocket client is part of the System Controller project and is licensed under the MIT License.
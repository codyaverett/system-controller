# System Controller

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Test Coverage](https://img.shields.io/badge/coverage-100%25-brightgreen)]()
[![Rust Version](https://img.shields.io/badge/rust-1.70+-blue)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()

A high-performance, cross-platform remote system control server built in Rust. Enables secure remote control of mouse, keyboard, screen capture, and window management across Windows, macOS, and Linux systems.

## ✨ Features

### 🎯 Core Capabilities
- **Remote Input Control**: Precise mouse movement, clicks, scrolling, and keyboard simulation
- **Multi-Display Support**: Screen capture and streaming across multiple monitors
- **Window Management**: Real-time window information and active application details
- **Cross-Platform**: Native support for Windows, macOS, and Linux
- **Headless Support**: Full keyboard control in server environments without GUI

### 🚀 Performance & Optimization
- **Platform-Specific Optimizations**: Adaptive performance tuning for each operating system
- **Intelligent Batching**: Queue and batch operations for optimal throughput
- **Resource Management**: Memory and CPU usage optimization with monitoring
- **Real-time Performance**: Low-latency input processing with sub-16ms response times

### 🔐 Security & Enterprise Ready
- **TLS Encryption**: All communications secured with TLS 1.3
- **JWT Authentication**: Token-based authentication with configurable expiration
- **Rate Limiting**: Configurable rate limits and resource protection
- **Audit Logging**: Comprehensive security event logging and monitoring
- **Role-Based Access**: Granular permission system and access control

## 🚀 Quick Start

### Prerequisites

- **Rust**: Version 1.70 or higher ([Install Rust](https://rustup.rs/))
- **Platform Dependencies**:
  - **Windows**: No additional dependencies
  - **macOS**: Xcode command line tools (`xcode-select --install`)
  - **Linux**: X11 development libraries (`apt-get install libx11-dev libxi-dev` on Ubuntu)

### Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/your-org/system-controller.git
   cd system-controller
   ```

2. **Build the application**:
   ```bash
   cargo build --release
   ```

3. **Run tests** (optional but recommended):
   ```bash
   cargo test
   ```

### Basic Usage

#### Start the Server

```bash
# Start on default port (8080)
./target/release/system-controller server

# Specify custom address and port
./target/release/system-controller server --address 0.0.0.0 --port 9090
```

#### Test System Capabilities

```bash
# Test platform capabilities
./target/release/system-controller test
```

The server will start and display:
```
INFO system_controller: Starting system controller server on 127.0.0.1:8080
INFO system_controller: Platform: Linux (GUI detected)
INFO system_controller: Security: TLS enabled, Authentication required
INFO system_controller: Ready to accept connections
```

## 📡 API Usage

### Connection

The server accepts both raw TCP and WebSocket connections on the configured port.

**WebSocket Connection**:
```javascript
const ws = new WebSocket('wss://localhost:8080');
```

**TCP Connection**:
```python
import socket
import ssl

context = ssl.create_default_context()
sock = socket.create_connection(('localhost', 8080))
secure_sock = context.wrap_socket(sock, server_hostname='localhost')
```

### Authentication

All commands require authentication via JWT token:

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

Response:
```json
{
  "command_id": "auth-001",
  "status": "success",
  "data": {
    "type": "auth_success",
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "expires_at": "2024-01-15T11:30:00Z"
  }
}
```

### Command Examples

#### Mouse Control

```json
{
  "id": "mouse-001",
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

```json
{
  "id": "click-001", 
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

#### Keyboard Control

```json
{
  "id": "key-001",
  "type": "key_press", 
  "payload": {
    "type": "key_press",
    "key": "Enter"
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

```json
{
  "id": "type-001",
  "type": "type_text",
  "payload": {
    "type": "type_text", 
    "text": "Hello, World!"
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

#### Screen Capture

```json
{
  "id": "capture-001",
  "type": "capture_screen",
  "payload": {
    "type": "capture_screen",
    "display_id": 0
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "auth_token": "your-jwt-token"
}
```

#### Get Display Information

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

Response:
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
        "is_primary": true
      }
    ]
  }
}
```

## ⚙️ Configuration

### Environment Variables

```bash
# Server configuration
SYSTEM_CONTROLLER_HOST=127.0.0.1      # Bind address
SYSTEM_CONTROLLER_PORT=8080            # Server port
SYSTEM_CONTROLLER_PLATFORM=auto        # Platform type (auto/enigo/headless)

# Security configuration  
SYSTEM_CONTROLLER_TLS_CERT=/path/to/cert.pem    # TLS certificate
SYSTEM_CONTROLLER_TLS_KEY=/path/to/key.pem      # TLS private key
SYSTEM_CONTROLLER_JWT_SECRET=your-secret-key    # JWT signing secret

# Performance tuning
SYSTEM_CONTROLLER_BATCH_SIZE=10         # Batch operation size
SYSTEM_CONTROLLER_RATE_LIMIT=100        # Commands per second limit
SYSTEM_CONTROLLER_WORKER_THREADS=4      # Async runtime threads
```

### Command Line Options

```bash
system-controller server --help

Start the system control server

Usage: system-controller server [OPTIONS]

Options:
  -p, --port <PORT>        Port to listen on [default: 8080]
  -a, --address <ADDRESS>  Address to bind to [default: 127.0.0.1]
  -c, --config <CONFIG>    Configuration file path
  -v, --verbose           Enable verbose logging
  -h, --help              Print help
```

## 🛠️ Development

### Project Structure

```
src/
├── main.rs                    # CLI entry point
├── lib.rs                     # Library root
├── platform/                  # Platform abstractions
│   ├── traits.rs              # Core platform traits
│   ├── factory.rs             # Platform factory
│   ├── cross_platform.rs      # Enhanced cross-platform controller  
│   ├── optimizations.rs       # Platform-specific optimizations
│   ├── enigo_platform.rs      # GUI platform implementation
│   └── headless.rs            # Headless platform implementation
├── protocol/                  # Network protocol
│   └── messages.rs            # Command/response definitions
├── security/                  # Security implementations
│   ├── auth.rs                # Authentication & JWT
│   ├── encryption.rs          # TLS and data encryption
│   ├── permissions.rs         # Access control
│   └── audit.rs               # Security logging
└── server/                    # Server implementations
    ├── input.rs               # Input processing
    ├── display.rs             # Display capture
    ├── network.rs             # Network server
    ├── enhanced_display.rs    # Enhanced display management
    ├── network_protocol.rs    # Protocol integration
    └── system_integration.rs  # System integration
```

### Building from Source

```bash
# Development build
cargo build

# Release build with optimizations
cargo build --release  

# Run tests
cargo test

# Run specific test suite
cargo test cross_platform_tests

# Generate coverage report
cargo test --coverage

# Run benchmarks
cargo bench

# Check code style
cargo fmt --check
cargo clippy
```

### Testing

The project maintains 100% test coverage with comprehensive test suites:

- **Unit Tests**: Individual component testing
- **Integration Tests**: Cross-component interaction testing
- **Cross-Platform Tests**: Platform-specific behavior validation
- **Property Tests**: Fuzzing and edge case testing
- **Performance Tests**: Benchmark and performance validation
- **End-to-End Tests**: Full system integration testing

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test patterns
cargo test test_cross_platform

# Run performance tests
cargo test test_platform_performance_characteristics
```

## 🏢 Platform-Specific Notes

### Windows
- Requires Windows 10 or later
- Uses Windows API for input simulation and window management
- Supports multiple monitors and DPI awareness
- Administrator privileges recommended for full functionality

### macOS  
- Requires macOS 10.14 (Mojave) or later
- Requires accessibility permissions for input control
- Grant permissions in: System Preferences → Security & Privacy → Accessibility
- May require notarization for distribution

### Linux
- Supports X11 and Wayland (X11 compatibility mode)
- Requires X11 development libraries
- May need additional permissions for input devices
- Desktop environment integration varies

## 🔒 Security Considerations

### Production Deployment

1. **TLS Configuration**: Always use TLS in production
   ```bash
   export SYSTEM_CONTROLLER_TLS_CERT=/path/to/cert.pem
   export SYSTEM_CONTROLLER_TLS_KEY=/path/to/key.pem
   ```

2. **Authentication**: Configure strong JWT secrets
   ```bash
   export SYSTEM_CONTROLLER_JWT_SECRET=$(openssl rand -hex 32)
   ```

3. **Network Security**: 
   - Bind to specific interfaces, not 0.0.0.0
   - Use firewall rules to restrict access
   - Consider VPN or SSH tunneling for remote access

4. **Access Control**:
   - Configure user permissions appropriately
   - Enable audit logging
   - Monitor for suspicious activity

### Security Features

- **End-to-End Encryption**: TLS 1.3 for all communications
- **Authentication**: JWT tokens with configurable expiration
- **Authorization**: Role-based permissions for different operations
- **Rate Limiting**: Prevents abuse and DoS attacks
- **Audit Logging**: Comprehensive security event logging
- **Input Validation**: All commands validated and sanitized

## 🚨 Troubleshooting

### Common Issues

#### Server Won't Start
```bash
Error: Address already in use (os error 98)
```
**Solution**: Change port or stop conflicting service
```bash
system-controller server --port 9090
```

#### Permission Denied (Linux/macOS)
```bash  
Error: Permission denied for input simulation
```
**Solution**: 
- Linux: Add user to `input` group or run with sudo
- macOS: Grant accessibility permissions in System Preferences

#### TLS Certificate Issues
```bash
Error: Invalid certificate or key file
```
**Solution**: Generate valid certificates
```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

#### High CPU Usage
**Solution**: Adjust batch size and rate limiting
```bash
export SYSTEM_CONTROLLER_BATCH_SIZE=50
export SYSTEM_CONTROLLER_RATE_LIMIT=50
```

### Debug Mode

Enable verbose logging for troubleshooting:
```bash
RUST_LOG=debug system-controller server --verbose
```

### Performance Monitoring

The server provides built-in performance monitoring:
- Operation latency tracking
- Memory usage monitoring  
- CPU utilization tracking
- Network throughput metrics
- Error rate monitoring

Access metrics via the `/metrics` endpoint or enable logging.

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes and add tests
4. Ensure all tests pass (`cargo test`)
5. Run code formatting (`cargo fmt`)
6. Run linting (`cargo clippy`)
7. Commit your changes (`git commit -m 'Add amazing feature'`)
8. Push to the branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Address all Clippy warnings (`cargo clippy`)
- Maintain test coverage at 100%
- Document all public APIs
- Follow conventional commit messages

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [enigo](https://crates.io/crates/enigo) - Cross-platform input simulation
- [screenshots](https://crates.io/crates/screenshots) - Screen capture functionality  
- [tokio](https://tokio.rs/) - Asynchronous runtime
- [serde](https://serde.rs/) - Serialization framework
- The Rust community for excellent tooling and support

## 📞 Support

- **Documentation**: [Full API Documentation](docs/)
- **Issues**: [GitHub Issues](https://github.com/your-org/system-controller/issues)
- **Security**: Report security issues to security@your-org.com
- **Community**: [Discord Server](https://discord.gg/your-invite)

---

**⚠️ Important**: This software is intended for legitimate system administration and automation purposes. Always ensure you have proper authorization before using remote control capabilities on any system.
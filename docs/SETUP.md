# System Controller - Detailed Setup Guide

This guide provides comprehensive setup instructions for the System Controller application across all supported platforms.

## Table of Contents

- [System Requirements](#system-requirements)
- [Platform-Specific Setup](#platform-specific-setup)
  - [Windows Setup](#windows-setup)
  - [macOS Setup](#macos-setup) 
  - [Linux Setup](#linux-setup)
- [Environment Configuration](#environment-configuration)
- [Authentication Setup](#authentication-setup)
- [Network Configuration](#network-configuration)
- [TLS/SSL Configuration](#tlsssl-configuration)
- [Performance Tuning](#performance-tuning)
- [Docker Deployment](#docker-deployment)
- [Troubleshooting](#troubleshooting)

## System Requirements

### Minimum Requirements
- **CPU**: 2 cores, 2.0 GHz
- **RAM**: 512 MB available memory
- **Disk**: 100 MB free space
- **Network**: TCP port access for server operation

### Recommended Requirements
- **CPU**: 4+ cores, 3.0+ GHz
- **RAM**: 2 GB available memory
- **Disk**: 1 GB free space (for logs and temporary files)
- **Network**: Dedicated network interface with stable connection

### Platform Support Matrix

| Platform | Version | Input Control | Screen Capture | Window Management | Notes |
|----------|---------|---------------|----------------|-------------------|-------|
| Windows | 10+ | ✅ | ✅ | ✅ | Full support |
| Windows | Server 2019+ | ⚠️ | ⚠️ | ⚠️ | Headless mode recommended |
| macOS | 10.14+ | ✅ | ✅ | ✅ | Requires accessibility permissions |
| Linux (X11) | Ubuntu 18.04+ | ✅ | ✅ | ✅ | Full support |
| Linux (Wayland) | Ubuntu 20.04+ | ⚠️ | ⚠️ | ⚠️ | Limited support via XWayland |
| Linux (Server) | Any | ⚠️ | ❌ | ❌ | Headless mode only |

## Platform-Specific Setup

### Windows Setup

#### Prerequisites
```powershell
# Verify Windows version (Windows 10 or later required)
winver

# Check PowerShell version (5.0+ recommended)
$PSVersionTable.PSVersion
```

#### Install Rust
```powershell
# Download and install Rustup
Invoke-WebRequest -Uri "https://win.rustup.rs/" -OutFile "rustup-init.exe"
.\rustup-init.exe

# Restart terminal and verify installation
rustc --version
cargo --version
```

#### Install Build Tools
```powershell
# Install Visual Studio Build Tools (if not already installed)
# Download from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio

# Or install via Chocolatey
choco install visualstudio2019buildtools
```

#### Windows-Specific Configuration
```powershell
# Enable developer mode (optional, for enhanced debugging)
# Settings > Update & Security > For Developers > Developer Mode

# Configure Windows Defender exclusions (if needed)
Add-MpPreference -ExclusionPath "C:\path\to\system-controller"
```

#### Build Application
```powershell
git clone https://github.com/your-org/system-controller.git
cd system-controller

# Build with Windows optimizations
cargo build --release --target x86_64-pc-windows-msvc

# Test installation
.\target\release\system-controller.exe test
```

#### Running as Windows Service
```powershell
# Install as Windows service using sc
sc create SystemController binPath="C:\path\to\system-controller.exe server --port 8080" start=auto

# Start the service
sc start SystemController

# Check service status
sc query SystemController
```

### macOS Setup

#### Prerequisites
```bash
# Verify macOS version (10.14+ required)
sw_vers

# Install Xcode Command Line Tools
xcode-select --install

# Verify installation
xcode-select -p
```

#### Install Rust
```bash
# Install Rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source the environment
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

#### macOS Permissions Setup
```bash
# The application requires accessibility permissions for input control
# Navigate to: System Preferences → Security & Privacy → Privacy → Accessibility
# Add the system-controller binary to the allowed applications list

# For automated setup, you can request permissions programmatically:
# The app will prompt for permissions on first input operation
```

#### Build Application
```bash
git clone https://github.com/your-org/system-controller.git
cd system-controller

# Build with macOS optimizations
cargo build --release --target x86_64-apple-darwin

# For Apple Silicon Macs
cargo build --release --target aarch64-apple-darwin

# Test installation
./target/release/system-controller test
```

#### Running as macOS Service (launchd)
```bash
# Create plist file
cat > ~/Library/LaunchAgents/com.yourorg.system-controller.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.yourorg.system-controller</string>
    <key>ProgramArguments</key>
    <array>
        <string>/path/to/system-controller</string>
        <string>server</string>
        <string>--port</string>
        <string>8080</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
EOF

# Load and start the service
launchctl load ~/Library/LaunchAgents/com.yourorg.system-controller.plist
launchctl start com.yourorg.system-controller
```

### Linux Setup

#### Ubuntu/Debian Setup
```bash
# Update package manager
sudo apt update && sudo apt upgrade -y

# Install dependencies
sudo apt install -y \
    curl \
    build-essential \
    pkg-config \
    libx11-dev \
    libxi-dev \
    libxtst-dev \
    libxrandr-dev \
    libxss-dev \
    libglib2.0-dev \
    libgtk-3-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### RHEL/CentOS/Fedora Setup
```bash
# RHEL/CentOS
sudo yum groupinstall "Development Tools" -y
sudo yum install -y \
    curl \
    pkg-config \
    libX11-devel \
    libXi-devel \
    libXtst-devel \
    libXrandr-devel \
    libXScrnSaver-devel \
    glib2-devel \
    gtk3-devel

# Fedora
sudo dnf groupinstall "Development Tools" -y  
sudo dnf install -y \
    curl \
    pkg-config \
    libX11-devel \
    libXi-devel \
    libXtst-devel \
    libXrandr-devel \
    libXScrnSaver-devel \
    glib2-devel \
    gtk3-devel

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Arch Linux Setup
```bash
# Install dependencies
sudo pacman -S \
    base-devel \
    curl \
    pkg-config \
    libx11 \
    libxi \
    libxtst \
    libxrandr \
    libxss \
    glib2 \
    gtk3

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Build Application
```bash
git clone https://github.com/your-org/system-controller.git
cd system-controller

# Build with Linux optimizations
cargo build --release --target x86_64-unknown-linux-gnu

# Test installation
./target/release/system-controller test
```

#### Permissions Setup
```bash
# Add user to input group (if exists)
sudo usermod -a -G input $USER

# Create udev rules for device access (if needed)
sudo tee /etc/udev/rules.d/99-system-controller.rules << EOF
# Allow access to input devices
SUBSYSTEM=="input", GROUP="input", MODE="0664"
KERNEL=="uinput", GROUP="input", MODE="0664"
EOF

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

#### Running as systemd Service
```bash
# Create service file
sudo tee /etc/systemd/system/system-controller.service << EOF
[Unit]
Description=System Controller Remote Control Server
After=network.target

[Service]
Type=simple
User=system-controller
Group=system-controller
ExecStart=/usr/local/bin/system-controller server --address 127.0.0.1 --port 8080
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

# Create dedicated user
sudo useradd -r -s /bin/false system-controller

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable system-controller
sudo systemctl start system-controller

# Check status
sudo systemctl status system-controller
```

## Environment Configuration

### Environment Variables

Create a `.env` file in your project directory or set system environment variables:

```bash
# Server Configuration
SYSTEM_CONTROLLER_HOST=127.0.0.1
SYSTEM_CONTROLLER_PORT=8080
SYSTEM_CONTROLLER_PLATFORM=auto              # auto, enigo, headless, headless-silent

# Security Configuration
SYSTEM_CONTROLLER_TLS_CERT=/path/to/cert.pem
SYSTEM_CONTROLLER_TLS_KEY=/path/to/key.pem
SYSTEM_CONTROLLER_JWT_SECRET=your-256-bit-secret-key
SYSTEM_CONTROLLER_JWT_EXPIRATION=3600        # Token expiration in seconds

# Performance Configuration
SYSTEM_CONTROLLER_BATCH_SIZE=10              # Batch operation size
SYSTEM_CONTROLLER_RATE_LIMIT=100             # Commands per second
SYSTEM_CONTROLLER_WORKER_THREADS=4           # Async runtime threads
SYSTEM_CONTROLLER_MAX_CONNECTIONS=100        # Maximum concurrent connections

# Logging Configuration
RUST_LOG=info                                # Logging level (error, warn, info, debug, trace)
SYSTEM_CONTROLLER_LOG_FILE=/var/log/system-controller.log
SYSTEM_CONTROLLER_AUDIT_LOG=/var/log/system-controller-audit.log

# Platform-Specific Configuration
SYSTEM_CONTROLLER_ENABLE_GPU_ACCELERATION=true
SYSTEM_CONTROLLER_DISPLAY_CACHE_SIZE=64      # MB
SYSTEM_CONTROLLER_INPUT_BUFFER_SIZE=1000     # Number of queued input events
```

### Configuration File

Create a `config.toml` file for more complex configurations:

```toml
[server]
host = "127.0.0.1"
port = 8080
platform = "auto"
max_connections = 100

[security]
tls_cert = "/path/to/cert.pem"
tls_key = "/path/to/key.pem"
jwt_secret = "your-secret-key"
jwt_expiration = 3600
enable_audit_log = true

[performance]
batch_size = 10
rate_limit = 100
worker_threads = 4
enable_optimizations = true

[logging]
level = "info"
log_file = "/var/log/system-controller.log"
audit_file = "/var/log/system-controller-audit.log"
enable_structured_logging = true

[platforms.windows]
enable_hardware_acceleration = true
use_direct_input = false

[platforms.macos]
use_core_graphics = true
enable_accessibility_api = true

[platforms.linux]
prefer_x11 = true
enable_wayland_support = false
```

## Authentication Setup

### Default Authentication

The server starts with a default admin user:
- **Username**: `admin`  
- **Password**: `changeme123!`

**⚠️ Important**: Change the default password immediately in production!

### Creating Users

```bash
# Create a new user via API call
curl -X POST https://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_ADMIN_TOKEN" \
  -d '{
    "username": "operator",
    "password": "secure-password",
    "permissions": ["input_control", "screen_capture"]
  }'
```

### User Permissions

Available permissions:
- `input_control`: Mouse and keyboard control
- `screen_capture`: Screen capture and streaming
- `window_management`: Window information and management
- `system_info`: System information access
- `admin`: Full administrative access

### JWT Configuration

```bash
# Generate a secure JWT secret
openssl rand -hex 32
export SYSTEM_CONTROLLER_JWT_SECRET="your-generated-secret"

# Configure token expiration (in seconds)
export SYSTEM_CONTROLLER_JWT_EXPIRATION=3600  # 1 hour
```

## Network Configuration

### Firewall Configuration

#### Windows Firewall
```powershell
# Allow inbound connections on port 8080
netsh advfirewall firewall add rule name="System Controller" dir=in action=allow protocol=TCP localport=8080

# For specific IP ranges
netsh advfirewall firewall add rule name="System Controller LAN" dir=in action=allow protocol=TCP localport=8080 remoteip=192.168.1.0/24
```

#### Linux iptables
```bash
# Allow inbound connections on port 8080
sudo iptables -A INPUT -p tcp --dport 8080 -j ACCEPT

# For specific IP ranges
sudo iptables -A INPUT -p tcp --dport 8080 -s 192.168.1.0/24 -j ACCEPT

# Save rules (Ubuntu/Debian)
sudo iptables-save | sudo tee /etc/iptables/rules.v4
```

#### Linux ufw (Ubuntu)
```bash
# Allow port 8080
sudo ufw allow 8080

# Allow from specific subnet
sudo ufw allow from 192.168.1.0/24 to any port 8080
```

#### macOS pf
```bash
# Edit /etc/pf.conf to add:
pass in on en0 proto tcp from 192.168.1.0/24 to any port 8080

# Reload rules
sudo pfctl -f /etc/pf.conf
```

### Reverse Proxy Configuration

#### Nginx Configuration
```nginx
server {
    listen 443 ssl http2;
    server_name system-controller.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        
        # WebSocket support
        proxy_set_header Sec-WebSocket-Extensions $http_sec_websocket_extensions;
        proxy_set_header Sec-WebSocket-Key $http_sec_websocket_key;
        proxy_set_header Sec-WebSocket-Version $http_sec_websocket_version;
    }
}
```

## TLS/SSL Configuration

### Generate Self-Signed Certificates

```bash
# Generate private key
openssl genrsa -out key.pem 4096

# Generate certificate signing request
openssl req -new -key key.pem -out csr.pem

# Generate self-signed certificate (valid for 1 year)
openssl x509 -req -days 365 -in csr.pem -signkey key.pem -out cert.pem

# Set appropriate permissions
chmod 600 key.pem
chmod 644 cert.pem
```

### Let's Encrypt Certificate

```bash
# Install certbot
sudo apt install certbot  # Ubuntu/Debian
sudo yum install certbot  # RHEL/CentOS

# Generate certificate
sudo certbot certonly --standalone -d system-controller.example.com

# Configure automatic renewal
sudo crontab -e
# Add: 0 12 * * * /usr/bin/certbot renew --quiet
```

### Configure TLS in Application

```bash
export SYSTEM_CONTROLLER_TLS_CERT="/etc/letsencrypt/live/system-controller.example.com/fullchain.pem"
export SYSTEM_CONTROLLER_TLS_KEY="/etc/letsencrypt/live/system-controller.example.com/privkey.pem"
```

## Performance Tuning

### System-Level Optimizations

#### Linux
```bash
# Increase file descriptor limits
echo "* soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65536" | sudo tee -a /etc/security/limits.conf

# Optimize network parameters
echo "net.core.rmem_max = 16777216" | sudo tee -a /etc/sysctl.conf
echo "net.core.wmem_max = 16777216" | sudo tee -a /etc/sysctl.conf
echo "net.ipv4.tcp_rmem = 4096 87380 16777216" | sudo tee -a /etc/sysctl.conf
echo "net.ipv4.tcp_wmem = 4096 65536 16777216" | sudo tee -a /etc/sysctl.conf

sudo sysctl -p
```

#### Windows
```powershell
# Increase TCP receive window
netsh int tcp set global autotuninglevel=normal

# Enable TCP window scaling
netsh int tcp set global windowscalingheuristics=enabled
```

### Application Performance Tuning

```bash
# Configure optimal thread count (usually CPU cores * 2)
export SYSTEM_CONTROLLER_WORKER_THREADS=8

# Adjust batch processing
export SYSTEM_CONTROLLER_BATCH_SIZE=20

# Configure memory limits
export SYSTEM_CONTROLLER_MAX_MEMORY_MB=512

# Enable platform optimizations
export SYSTEM_CONTROLLER_ENABLE_OPTIMIZATIONS=true
```

### High-Performance Configuration

For high-throughput scenarios:

```toml
[performance]
batch_size = 50
rate_limit = 1000
worker_threads = 16
enable_optimizations = true
max_memory_mb = 2048
enable_gpu_acceleration = true

[platforms.optimizations]
enable_batch_operations = true
cache_ttl_seconds = 60
min_operation_interval_ms = 1
max_cpu_usage_percent = 90
```

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libx11-6 \
    libxi6 \
    libxtst6 \
    libxrandr2 \
    libxss1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/system-controller /usr/local/bin/

EXPOSE 8080

CMD ["system-controller", "server", "--address", "0.0.0.0", "--port", "8080"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  system-controller:
    build: .
    ports:
      - "8080:8080"
    environment:
      - SYSTEM_CONTROLLER_HOST=0.0.0.0
      - SYSTEM_CONTROLLER_PORT=8080
      - SYSTEM_CONTROLLER_PLATFORM=headless
      - RUST_LOG=info
    volumes:
      - ./certs:/certs:ro
      - ./logs:/var/log
    restart: unless-stopped
    
  nginx:
    image: nginx:alpine
    ports:
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./certs:/certs:ro
    depends_on:
      - system-controller
    restart: unless-stopped
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: system-controller
spec:
  replicas: 3
  selector:
    matchLabels:
      app: system-controller
  template:
    metadata:
      labels:
        app: system-controller
    spec:
      containers:
      - name: system-controller
        image: your-registry/system-controller:latest
        ports:
        - containerPort: 8080
        env:
        - name: SYSTEM_CONTROLLER_HOST
          value: "0.0.0.0"
        - name: SYSTEM_CONTROLLER_PORT
          value: "8080"
        - name: SYSTEM_CONTROLLER_PLATFORM
          value: "headless"
        resources:
          limits:
            memory: "512Mi"
            cpu: "500m"
          requests:
            memory: "256Mi"
            cpu: "250m"
---
apiVersion: v1
kind: Service
metadata:
  name: system-controller-service
spec:
  selector:
    app: system-controller
  ports:
  - protocol: TCP
    port: 8080
    targetPort: 8080
  type: LoadBalancer
```

## Troubleshooting

### Common Issues

#### Build Failures

**Error**: `linker 'cc' not found`
```bash
# Linux: Install build essentials
sudo apt install build-essential

# macOS: Install Xcode command line tools
xcode-select --install
```

**Error**: `failed to run custom build command for 'openssl-sys'`
```bash
# Ubuntu/Debian
sudo apt install libssl-dev pkg-config

# RHEL/CentOS
sudo yum install openssl-devel pkgconfig
```

#### Runtime Issues

**Error**: `Permission denied (os error 13)`
```bash
# Linux: Check file permissions and user groups
ls -la /usr/local/bin/system-controller
sudo chmod +x /usr/local/bin/system-controller

# Add user to appropriate groups
sudo usermod -a -G input,video $USER
```

**Error**: `Address already in use (os error 98)`
```bash
# Find process using the port
sudo netstat -tulpn | grep :8080
sudo lsof -i :8080

# Kill the process or use a different port
sudo kill -9 PID
# or
system-controller server --port 9090
```

#### Platform-Specific Issues

**macOS**: Accessibility permissions denied
```bash
# Navigate to System Preferences → Security & Privacy → Privacy → Accessibility
# Add system-controller to the list of allowed applications
# May need to restart the application after granting permissions
```

**Linux**: X11 display issues in headless environment
```bash
# Install and configure Xvfb for virtual display
sudo apt install xvfb
export DISPLAY=:99
Xvfb :99 -screen 0 1024x768x24 &

# Or force headless mode
export SYSTEM_CONTROLLER_PLATFORM=headless
```

**Windows**: Windows Defender blocking application
```powershell
# Add exclusion for the application
Add-MpPreference -ExclusionPath "C:\path\to\system-controller.exe"

# Or temporarily disable real-time protection
Set-MpPreference -DisableRealtimeMonitoring $true
```

### Debug Mode

Enable detailed logging for troubleshooting:

```bash
# Enable debug logging
export RUST_LOG=debug

# Enable trace logging (very verbose)
export RUST_LOG=trace

# Module-specific logging
export RUST_LOG=system_controller::platform=debug,system_controller::security=info

# Run with verbose output
system-controller server --verbose
```

### Performance Issues

**High CPU Usage**:
```bash
# Reduce batch size and rate limits
export SYSTEM_CONTROLLER_BATCH_SIZE=5
export SYSTEM_CONTROLLER_RATE_LIMIT=50

# Limit worker threads
export SYSTEM_CONTROLLER_WORKER_THREADS=2
```

**High Memory Usage**:
```bash
# Reduce cache sizes
export SYSTEM_CONTROLLER_DISPLAY_CACHE_SIZE=32
export SYSTEM_CONTROLLER_MAX_MEMORY_MB=256

# Enable memory optimization
export SYSTEM_CONTROLLER_ENABLE_MEMORY_OPTIMIZATION=true
```

**Network Latency**:
```bash
# Reduce batch intervals
export SYSTEM_CONTROLLER_MIN_OPERATION_INTERVAL_MS=1

# Enable compression
export SYSTEM_CONTROLLER_ENABLE_COMPRESSION=true

# Use faster serialization
export SYSTEM_CONTROLLER_ENABLE_BINARY_PROTOCOL=true
```

### Getting Help

If you encounter issues not covered in this guide:

1. **Check the logs**: Enable debug logging and review the output
2. **Search existing issues**: Check the GitHub issues for similar problems
3. **Create a new issue**: Include system information, error messages, and logs
4. **Join the community**: Connect with other users on Discord or forums

For immediate support with critical issues, contact: support@your-org.com
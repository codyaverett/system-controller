# System Controller Examples

This directory contains example implementations and usage scenarios for the System Controller API.

## Quick Reference

| Example | Language | Description |
|---------|----------|-------------|
| [basic-client.py](#python-basic-client) | Python | Simple TCP client with authentication |
| [websocket-client.js](#nodejs-websocket-client) | Node.js | WebSocket client with event handling |
| [automation-script.py](#python-automation-script) | Python | Complete automation workflow example |
| [monitoring-dashboard.js](#monitoring-dashboard) | Node.js | Real-time monitoring dashboard |
| [docker-compose.yml](#docker-deployment) | Docker | Container deployment example |
| [client.cs](#c-client) | C# | .NET WebSocket client implementation |

## Getting Started

1. **Start the Server**:
   ```bash
   cd ..
   cargo run -- server --port 8080
   ```

2. **Run Examples**:
   ```bash
   # Python examples
   python3 basic-client.py
   python3 automation-script.py
   
   # Node.js examples
   npm install ws uuid
   node websocket-client.js
   node monitoring-dashboard.js
   
   # Docker deployment
   docker-compose up -d
   ```

## Example Files

### Python Basic Client
**File**: `basic-client.py`

Simple TCP client demonstrating basic operations:
- Secure TLS connection
- JWT authentication
- Mouse and keyboard control
- Screen capture
- Error handling

### Node.js WebSocket Client
**File**: `websocket-client.js`

WebSocket client with advanced features:
- Persistent connection management
- Event-driven architecture
- Automatic reconnection
- Promise-based API
- Real-time communication

### Python Automation Script
**File**: `automation-script.py`

Complete automation workflow:
- Multi-step task automation
- Error recovery
- Progress reporting
- Configuration management
- Logging and debugging

### Monitoring Dashboard
**File**: `monitoring-dashboard.js`

Real-time monitoring interface:
- System performance metrics
- Connection status
- Command execution statistics
- Alert notifications
- Historical data visualization

### Docker Deployment
**File**: `docker-compose.yml`

Production-ready container deployment:
- Multi-service architecture
- TLS termination with nginx
- Persistent storage
- Health checks
- Environment configuration

### C# Client
**File**: `client.cs`

.NET implementation:
- Async/await patterns
- WebSocket communication
- JSON serialization
- Exception handling
- Configuration management

## Configuration Files

### Client Configuration
**File**: `config.json`

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
    "password": "changeme123!",
    "token_refresh_threshold": 300
  },
  "timeouts": {
    "connection": 30000,
    "command": 5000,
    "authentication": 10000
  },
  "logging": {
    "level": "info",
    "file": "client.log"
  }
}
```

### Environment Configuration
**File**: `.env`

```bash
# Server connection
SYSTEM_CONTROLLER_HOST=localhost
SYSTEM_CONTROLLER_PORT=8080
SYSTEM_CONTROLLER_USE_TLS=true

# Authentication
SYSTEM_CONTROLLER_USERNAME=admin
SYSTEM_CONTROLLER_PASSWORD=changeme123!

# Client settings
CLIENT_TIMEOUT=30000
CLIENT_RETRY_ATTEMPTS=3
CLIENT_LOG_LEVEL=info
```

## Usage Scenarios

### Remote Desktop Automation

```python
#!/usr/bin/env python3
"""
Remote desktop automation example
Automates common desktop tasks like opening applications,
filling forms, and capturing screenshots.
"""

from examples.basic_client import SystemControllerClient
import time

def automate_desktop_tasks():
    client = SystemControllerClient()
    client.connect()
    
    if client.authenticate("admin", "changeme123!"):
        # Open application (Windows: Start menu)
        client.key_press("Meta")
        time.sleep(0.5)
        client.type_text("notepad")
        time.sleep(0.5)
        client.key_press("Enter")
        time.sleep(2)
        
        # Type some text
        client.type_text("Hello from System Controller!")
        
        # Save file
        client.key_press("Control+s")
        time.sleep(1)
        client.type_text("automation-test.txt")
        client.key_press("Enter")
        
        # Capture screenshot
        response = client.capture_screen(0)
        print(f"Screenshot captured: {response}")
        
    client.disconnect()

if __name__ == "__main__":
    automate_desktop_tasks()
```

### System Monitoring

```javascript
// monitoring-example.js
const SystemControllerClient = require('./websocket-client');

class SystemMonitor {
    constructor() {
        this.client = new SystemControllerClient();
        this.metrics = {
            commands_executed: 0,
            errors_encountered: 0,
            average_response_time: 0,
            start_time: Date.now()
        };
    }
    
    async start() {
        await this.client.connect();
        
        if (await this.client.authenticate('admin', 'changeme123!')) {
            console.log('✅ Connected to System Controller');
            this.startMonitoring();
        }
    }
    
    startMonitoring() {
        setInterval(async () => {
            try {
                const start = Date.now();
                const displays = await this.client.getDisplays();
                const responseTime = Date.now() - start;
                
                this.updateMetrics(responseTime, true);
                this.logStatus(displays);
            } catch (error) {
                this.updateMetrics(0, false);
                console.error('❌ Monitoring error:', error.message);
            }
        }, 5000); // Monitor every 5 seconds
        
        // Print metrics every minute
        setInterval(() => {
            this.printMetrics();
        }, 60000);
    }
    
    updateMetrics(responseTime, success) {
        this.metrics.commands_executed++;
        
        if (success) {
            this.metrics.average_response_time = 
                (this.metrics.average_response_time + responseTime) / 2;
        } else {
            this.metrics.errors_encountered++;
        }
    }
    
    logStatus(displays) {
        console.log(`📊 System Status - Displays: ${displays.data?.displays?.length || 0}, ` +
                   `Response: ${this.metrics.average_response_time.toFixed(0)}ms`);
    }
    
    printMetrics() {
        const uptime = Math.floor((Date.now() - this.metrics.start_time) / 1000);
        const errorRate = (this.metrics.errors_encountered / this.metrics.commands_executed * 100).toFixed(1);
        
        console.log('\n📈 Monitoring Metrics:');
        console.log(`   Uptime: ${uptime}s`);
        console.log(`   Commands: ${this.metrics.commands_executed}`);
        console.log(`   Errors: ${this.metrics.errors_encountered} (${errorRate}%)`);
        console.log(`   Avg Response: ${this.metrics.average_response_time.toFixed(0)}ms\n`);
    }
}

// Start monitoring
const monitor = new SystemMonitor();
monitor.start().catch(console.error);
```

### Batch Operations

```python
#!/usr/bin/env python3
"""
Batch operations example
Demonstrates efficient batch processing of multiple commands
"""

import asyncio
import time
from examples.basic_client import SystemControllerClient

class BatchOperationClient(SystemControllerClient):
    def __init__(self):
        super().__init__()
        self.command_queue = []
    
    def queue_command(self, command_type, payload):
        """Queue a command for batch execution"""
        self.command_queue.append((command_type, payload))
    
    async def execute_batch(self):
        """Execute all queued commands efficiently"""
        if not self.command_queue:
            return []
        
        results = []
        batch_start = time.time()
        
        print(f"🚀 Executing batch of {len(self.command_queue)} commands...")
        
        for i, (command_type, payload) in enumerate(self.command_queue):
            try:
                start_time = time.time()
                result = self.send_command(command_type, payload)
                execution_time = (time.time() - start_time) * 1000
                
                results.append({
                    'command': command_type,
                    'status': result.get('status'),
                    'execution_time_ms': execution_time
                })
                
                print(f"  ✅ {i+1}/{len(self.command_queue)}: {command_type} ({execution_time:.0f}ms)")
                
                # Small delay to respect rate limits
                await asyncio.sleep(0.01)
                
            except Exception as e:
                results.append({
                    'command': command_type,
                    'status': 'error',
                    'error': str(e)
                })
                print(f"  ❌ {i+1}/{len(self.command_queue)}: {command_type} failed - {e}")
        
        batch_time = (time.time() - batch_start) * 1000
        print(f"📊 Batch completed in {batch_time:.0f}ms")
        
        # Clear the queue
        self.command_queue = []
        
        return results

def demonstrate_batch_operations():
    client = BatchOperationClient()
    client.connect()
    
    if client.authenticate("admin", "changeme123!"):
        # Queue multiple mouse movements
        for i in range(10):
            client.queue_command("mouse_move", {
                "type": "mouse_move",
                "x": 100 + i * 10,
                "y": 100 + i * 10
            })
        
        # Queue some keyboard operations
        client.queue_command("type_text", {
            "type": "type_text",
            "text": "Batch operation test"
        })
        
        client.queue_command("key_press", {
            "type": "key_press", 
            "key": "Enter"
        })
        
        # Execute all queued commands
        results = asyncio.run(client.execute_batch())
        
        # Print summary
        successful = sum(1 for r in results if r['status'] == 'success')
        print(f"\n📈 Summary: {successful}/{len(results)} commands successful")
    
    client.disconnect()

if __name__ == "__main__":
    demonstrate_batch_operations()
```

## Advanced Examples

### Multi-Display Screenshot Capture

```python
#!/usr/bin/env python3
"""
Multi-display screenshot capture example
Captures screenshots from all available displays
"""

import os
import base64
from datetime import datetime
from examples.basic_client import SystemControllerClient

def capture_all_displays():
    client = SystemControllerClient()
    client.connect()
    
    if client.authenticate("admin", "changeme123!"):
        # Get list of displays
        displays_response = client.get_displays()
        
        if displays_response['status'] == 'success':
            displays = displays_response['data']['displays']
            print(f"📺 Found {len(displays)} display(s)")
            
            # Create screenshots directory
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            screenshot_dir = f"screenshots_{timestamp}"
            os.makedirs(screenshot_dir, exist_ok=True)
            
            # Capture each display
            for display in displays:
                display_id = display['id']
                display_name = display['name'].replace(' ', '_')
                
                print(f"📸 Capturing display {display_id}: {display['name']} ({display['width']}x{display['height']})")
                
                capture_response = client.capture_screen(display_id)
                
                if capture_response['status'] == 'success':
                    # Note: In a real implementation, you would receive binary data
                    # This is a simplified example
                    filename = f"{screenshot_dir}/display_{display_id}_{display_name}.png"
                    print(f"   ✅ Saved to {filename}")
                else:
                    print(f"   ❌ Failed to capture: {capture_response.get('error')}")
            
            print(f"\n🎯 Screenshots saved to {screenshot_dir}/")
        
    client.disconnect()

if __name__ == "__main__":
    capture_all_displays()
```

### Performance Benchmark

```python
#!/usr/bin/env python3
"""
Performance benchmark example
Tests the performance characteristics of various operations
"""

import time
import statistics
from examples.basic_client import SystemControllerClient

def benchmark_operations():
    client = SystemControllerClient()
    client.connect()
    
    if client.authenticate("admin", "changeme123!"):
        benchmarks = {
            'mouse_move': [],
            'key_press': [],
            'get_displays': [],
            'type_text': []
        }
        
        iterations = 100
        print(f"🏃 Running performance benchmark ({iterations} iterations each)...")
        
        # Benchmark mouse movements
        print("  Testing mouse_move...")
        for i in range(iterations):
            start = time.time()
            client.mouse_move(100 + i, 100 + i)
            benchmarks['mouse_move'].append((time.time() - start) * 1000)
        
        # Benchmark key presses
        print("  Testing key_press...")
        for i in range(iterations):
            start = time.time()
            client.send_command("key_press", {
                "type": "key_press",
                "key": "a"
            })
            benchmarks['key_press'].append((time.time() - start) * 1000)
        
        # Benchmark display queries
        print("  Testing get_displays...")
        for i in range(20):  # Fewer iterations for heavier operations
            start = time.time()
            client.get_displays()
            benchmarks['get_displays'].append((time.time() - start) * 1000)
        
        # Benchmark text typing
        print("  Testing type_text...")
        for i in range(50):
            start = time.time()
            client.type_text("test")
            benchmarks['type_text'].append((time.time() - start) * 1000)
        
        # Print results
        print("\n📊 Benchmark Results:")
        print("=" * 60)
        
        for operation, times in benchmarks.items():
            if times:
                avg_time = statistics.mean(times)
                min_time = min(times)
                max_time = max(times)
                p95_time = statistics.quantiles(times, n=20)[18]  # 95th percentile
                
                print(f"{operation:15} | Avg: {avg_time:6.1f}ms | "
                      f"Min: {min_time:6.1f}ms | Max: {max_time:6.1f}ms | "
                      f"P95: {p95_time:6.1f}ms")
        
        print("=" * 60)
        
    client.disconnect()

if __name__ == "__main__":
    benchmark_operations()
```

## Testing Examples

### Integration Tests

```python
#!/usr/bin/env python3
"""
Integration test example
Comprehensive testing of system functionality
"""

import unittest
import time
from examples.basic_client import SystemControllerClient

class SystemControllerIntegrationTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        """Set up test client"""
        cls.client = SystemControllerClient()
        cls.client.connect()
        cls.assertTrue(
            cls.client.authenticate("admin", "changeme123!"),
            "Failed to authenticate"
        )
    
    @classmethod
    def tearDownClass(cls):
        """Clean up test client"""
        cls.client.disconnect()
    
    def test_mouse_operations(self):
        """Test mouse control operations"""
        # Test mouse movement
        response = self.client.mouse_move(100, 100)
        self.assertEqual(response['status'], 'success')
        
        # Test mouse click
        response = self.client.mouse_click("Left", 100, 100)
        self.assertEqual(response['status'], 'success')
        
        # Test mouse scroll
        response = self.client.send_command("mouse_scroll", {
            "type": "mouse_scroll",
            "x": 0,
            "y": -3
        })
        self.assertEqual(response['status'], 'success')
    
    def test_keyboard_operations(self):
        """Test keyboard control operations"""
        # Test key press
        response = self.client.send_command("key_press", {
            "type": "key_press",
            "key": "a"
        })
        self.assertEqual(response['status'], 'success')
        
        # Test text typing
        response = self.client.type_text("Integration test")
        self.assertEqual(response['status'], 'success')
    
    def test_display_operations(self):
        """Test display management operations"""
        # Test get displays
        response = self.client.get_displays()
        self.assertEqual(response['status'], 'success')
        self.assertIn('displays', response['data'])
        self.assertGreater(len(response['data']['displays']), 0)
        
        # Test screen capture
        display_id = response['data']['displays'][0]['id']
        capture_response = self.client.capture_screen(display_id)
        self.assertEqual(capture_response['status'], 'success')
    
    def test_error_handling(self):
        """Test error handling"""
        # Test invalid coordinates
        response = self.client.mouse_move(-100, -100)
        # Should either succeed (platform allows) or fail gracefully
        self.assertIn(response['status'], ['success', 'error'])
        
        # Test invalid key
        response = self.client.send_command("key_press", {
            "type": "key_press",
            "key": ""
        })
        self.assertEqual(response['status'], 'error')
    
    def test_rate_limiting(self):
        """Test rate limiting behavior"""
        # Send many rapid requests
        responses = []
        for i in range(150):  # Above typical rate limit
            response = self.client.mouse_move(i, i)
            responses.append(response)
            if i > 100 and response['status'] == 'error':
                break
        
        # Should eventually hit rate limit
        error_responses = [r for r in responses if r['status'] == 'error']
        self.assertGreater(len(error_responses), 0, "Rate limiting not triggered")

if __name__ == "__main__":
    unittest.main(verbosity=2)
```

## Deployment Examples

### Kubernetes Deployment

```yaml
# k8s-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: system-controller
  labels:
    app: system-controller
spec:
  replicas: 1
  selector:
    matchLabels:
      app: system-controller
  template:
    metadata:
      labels:
        app: system-controller
    spec:
      serviceAccountName: system-controller
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000
      containers:
      - name: system-controller
        image: system-controller:latest
        ports:
        - containerPort: 8080
          name: api
        env:
        - name: SYSTEM_CONTROLLER_HOST
          value: "0.0.0.0"
        - name: SYSTEM_CONTROLLER_PORT
          value: "8080"
        - name: SYSTEM_CONTROLLER_JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: system-controller-secrets
              key: jwt-secret
        resources:
          limits:
            memory: "512Mi"
            cpu: "500m"
          requests:
            memory: "256Mi"
            cpu: "250m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
            scheme: HTTPS
          initialDelaySeconds: 30
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
            scheme: HTTPS
          initialDelaySeconds: 5
          periodSeconds: 10
        volumeMounts:
        - name: tls-certs
          mountPath: /etc/ssl/certs/
          readOnly: true
      volumes:
      - name: tls-certs
        secret:
          secretName: system-controller-tls

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
    name: api
  type: ClusterIP

---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: system-controller-ingress
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
    nginx.ingress.kubernetes.io/backend-protocol: "HTTPS"
spec:
  tls:
  - hosts:
    - system-controller.example.com
    secretName: system-controller-tls
  rules:
  - host: system-controller.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: system-controller-service
            port:
              number: 8080
```

### Terraform Infrastructure

```hcl
# main.tf
provider "aws" {
  region = "us-west-2"
}

# Security Group
resource "aws_security_group" "system_controller" {
  name_prefix = "system-controller-"
  description = "Security group for System Controller"

  ingress {
    description = "HTTPS API"
    from_port   = 8080
    to_port     = 8080
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/8"]
  }

  ingress {
    description = "SSH"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/8"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "system-controller-sg"
  }
}

# EC2 Instance
resource "aws_instance" "system_controller" {
  ami           = "ami-0c55b159cbfafe1d0"  # Amazon Linux 2
  instance_type = "t3.medium"
  
  vpc_security_group_ids = [aws_security_group.system_controller.id]
  
  user_data = file("install-script.sh")
  
  tags = {
    Name = "system-controller"
  }
}

# Elastic IP
resource "aws_eip" "system_controller" {
  instance = aws_instance.system_controller.id
  domain   = "vpc"

  tags = {
    Name = "system-controller-eip"
  }
}

output "instance_ip" {
  value = aws_eip.system_controller.public_ip
}
```

## Contributing

To contribute new examples:

1. Create a new file in the appropriate language directory
2. Follow the existing naming convention
3. Include comprehensive comments and error handling
4. Add configuration examples where applicable
5. Update this README with a description of your example

## Support

For questions about these examples:
- Check the main [README](../README.md) for basic setup
- Review the [API Documentation](../docs/API.md) for detailed API reference
- Join our community Discord for real-time help
- Open an issue on GitHub for bug reports

## License

These examples are provided under the same license as the main System Controller project.
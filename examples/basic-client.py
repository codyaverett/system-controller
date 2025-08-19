#!/usr/bin/env python3
"""
System Controller Basic Python Client

A simple Python client demonstrating basic operations with the System Controller API.
This example shows how to:
- Establish a secure TLS connection
- Authenticate using JWT tokens
- Perform mouse and keyboard operations
- Capture screenshots
- Handle errors gracefully

Dependencies:
    pip install requests

Usage:
    python3 basic-client.py
"""

import json
import socket
import ssl
import time
import struct
from datetime import datetime
from typing import Dict, Any, Optional


class SystemControllerClient:
    """Simple System Controller client implementation"""
    
    def __init__(self, host: str = 'localhost', port: int = 8080, 
                 verify_ssl: bool = False):
        """
        Initialize the client
        
        Args:
            host: Server hostname or IP address
            port: Server port number
            verify_ssl: Whether to verify SSL certificates (False for self-signed)
        """
        self.host = host
        self.port = port
        self.verify_ssl = verify_ssl
        self.sock: Optional[ssl.SSLSocket] = None
        self.auth_token: Optional[str] = None
        self.command_counter = 0
    
    def connect(self) -> None:
        """Establish secure connection to the server"""
        try:
            # Create SSL context
            context = ssl.create_default_context()
            if not self.verify_ssl:
                context.check_hostname = False
                context.verify_mode = ssl.CERT_NONE
            
            # Create and wrap socket
            sock = socket.create_connection((self.host, self.port), timeout=30)
            self.sock = context.wrap_socket(sock, server_hostname=self.host)
            
            print(f"✅ Connected to {self.host}:{self.port}")
            
        except Exception as e:
            print(f"❌ Connection failed: {e}")
            raise
    
    def disconnect(self) -> None:
        """Close the connection"""
        if self.sock:
            try:
                self.sock.close()
                print("🔌 Disconnected")
            except:
                pass
            finally:
                self.sock = None
    
    def send_command(self, command_type: str, payload: Dict[str, Any], 
                    command_id: Optional[str] = None) -> Dict[str, Any]:
        """
        Send a command to the server and return the response
        
        Args:
            command_type: Type of command to send
            payload: Command payload data
            command_id: Optional command ID (auto-generated if not provided)
            
        Returns:
            Server response as dictionary
        """
        if not self.sock:
            raise RuntimeError("Not connected to server")
        
        # Generate command ID if not provided
        if not command_id:
            self.command_counter += 1
            command_id = f"{command_type}-{self.command_counter}"
        
        # Build command
        command = {
            "id": command_id,
            "type": command_type,
            "payload": payload,
            "timestamp": datetime.now().isoformat() + "Z"
        }
        
        # Add auth token if available
        if self.auth_token:
            command["auth_token"] = self.auth_token
        
        try:
            # Serialize and send command
            message = json.dumps(command).encode('utf-8')
            length = len(message).to_bytes(4, 'little')
            
            self.sock.send(length + message)
            
            # Receive response
            return self._receive_response()
            
        except Exception as e:
            print(f"❌ Command '{command_type}' failed: {e}")
            raise
    
    def _receive_response(self) -> Dict[str, Any]:
        """Receive and parse response from server"""
        try:
            # Read response length
            length_bytes = self.sock.recv(4)
            if len(length_bytes) != 4:
                raise RuntimeError("Failed to read response length")
            
            length = int.from_bytes(length_bytes, 'little')
            
            # Read response data
            response_data = b''
            while len(response_data) < length:
                chunk = self.sock.recv(length - len(response_data))
                if not chunk:
                    raise RuntimeError("Connection closed while reading response")
                response_data += chunk
            
            # Parse JSON response
            response = json.loads(response_data.decode('utf-8'))
            return response
            
        except Exception as e:
            print(f"❌ Failed to receive response: {e}")
            raise
    
    def authenticate(self, username: str, password: str) -> bool:
        """
        Authenticate with the server
        
        Args:
            username: Username
            password: Password
            
        Returns:
            True if authentication successful, False otherwise
        """
        try:
            response = self.send_command("authenticate", {
                "username": username,
                "password": password
            })
            
            if response['status'] == 'success':
                self.auth_token = response['data']['token']
                expires_at = response['data']['expires_at']
                permissions = response['data'].get('permissions', [])
                
                print(f"🔑 Authentication successful")
                print(f"   Token expires: {expires_at}")
                print(f"   Permissions: {', '.join(permissions)}")
                return True
            else:
                error = response.get('error', 'Unknown error')
                print(f"❌ Authentication failed: {error}")
                return False
                
        except Exception as e:
            print(f"❌ Authentication error: {e}")
            return False
    
    def mouse_move(self, x: int, y: int) -> Dict[str, Any]:
        """Move mouse to specified coordinates"""
        return self.send_command("mouse_move", {
            "type": "mouse_move",
            "x": x,
            "y": y
        })
    
    def mouse_click(self, button: str, x: int, y: int) -> Dict[str, Any]:
        """Click mouse button at specified coordinates"""
        return self.send_command("mouse_click", {
            "type": "mouse_click",
            "button": button,
            "x": x,
            "y": y
        })
    
    def mouse_scroll(self, x: int, y: int) -> Dict[str, Any]:
        """Scroll mouse wheel"""
        return self.send_command("mouse_scroll", {
            "type": "mouse_scroll", 
            "x": x,
            "y": y
        })
    
    def key_press(self, key: str) -> Dict[str, Any]:
        """Press a key"""
        return self.send_command("key_press", {
            "type": "key_press",
            "key": key
        })
    
    def key_release(self, key: str) -> Dict[str, Any]:
        """Release a key"""
        return self.send_command("key_release", {
            "type": "key_release",
            "key": key
        })
    
    def type_text(self, text: str) -> Dict[str, Any]:
        """Type text"""
        return self.send_command("type_text", {
            "type": "type_text",
            "text": text
        })
    
    def capture_screen(self, display_id: int = 0) -> Dict[str, Any]:
        """Capture screenshot of specified display"""
        return self.send_command("capture_screen", {
            "type": "capture_screen",
            "display_id": display_id
        })
    
    def get_displays(self) -> Dict[str, Any]:
        """Get list of available displays"""
        return self.send_command("get_displays", {
            "type": "get_displays"
        })
    
    def list_windows(self) -> Dict[str, Any]:
        """Get list of all windows"""
        return self.send_command("list_windows", {
            "type": "list_windows"
        })
    
    def get_window_at_position(self, x: int, y: int) -> Dict[str, Any]:
        """Get window information at specified position"""
        return self.send_command("get_window_info", {
            "type": "get_window_info",
            "x": x,
            "y": y
        })


def demonstrate_basic_operations():
    """Demonstrate basic System Controller operations"""
    
    print("🚀 System Controller Basic Client Demo")
    print("=" * 50)
    
    # Create and connect client
    client = SystemControllerClient(
        host='localhost',
        port=8080,
        verify_ssl=False  # Use True for production with valid certificates
    )
    
    try:
        # Connect to server
        client.connect()
        
        # Authenticate
        if not client.authenticate("admin", "changeme123!"):
            print("❌ Failed to authenticate. Please check credentials.")
            return
        
        print("\n📱 Testing mouse operations...")
        
        # Test mouse movement
        print("  🖱️  Moving mouse to (100, 100)")
        response = client.mouse_move(100, 100)
        print(f"     Result: {response['status']}")
        time.sleep(0.5)
        
        # Test mouse click
        print("  🖱️  Clicking left button at (100, 100)")
        response = client.mouse_click("Left", 100, 100)
        print(f"     Result: {response['status']}")
        time.sleep(0.5)
        
        # Test mouse scroll
        print("  🖱️  Scrolling down")
        response = client.mouse_scroll(0, -3)
        print(f"     Result: {response['status']}")
        
        print("\n⌨️  Testing keyboard operations...")
        
        # Test key press
        print("  ⌨️  Pressing 'a' key")
        response = client.key_press("a")
        print(f"     Result: {response['status']}")
        time.sleep(0.2)
        
        # Test text typing
        print("  ⌨️  Typing 'Hello, World!'")
        response = client.type_text("Hello, World!")
        print(f"     Result: {response['status']}")
        time.sleep(0.5)
        
        # Test Enter key
        print("  ⌨️  Pressing Enter")
        response = client.key_press("Enter")
        print(f"     Result: {response['status']}")
        
        print("\n📺 Testing display operations...")
        
        # Get displays
        print("  📺 Getting display information")
        response = client.get_displays()
        if response['status'] == 'success':
            displays = response['data']['displays']
            print(f"     Found {len(displays)} display(s):")
            for display in displays:
                print(f"       - {display['name']}: {display['width']}x{display['height']} "
                      f"at ({display['x']}, {display['y']}) {'(Primary)' if display['is_primary'] else ''}")
        else:
            print(f"     Error: {response.get('error')}")
        
        # Capture screenshot
        print("  📸 Capturing screenshot")
        response = client.capture_screen(0)
        if response['status'] == 'success':
            print(f"     Screenshot captured successfully")
            print(f"     Format: {response['data'].get('format', 'unknown')}")
            print(f"     Size: {response['data'].get('data_size', 0)} bytes")
        else:
            print(f"     Error: {response.get('error')}")
        
        print("\n🪟 Testing window operations...")
        
        # List windows
        print("  🪟 Getting window list")
        response = client.list_windows()
        if response['status'] == 'success':
            windows = response['data']['windows']
            print(f"     Found {len(windows)} window(s):")
            for window in windows[:5]:  # Show first 5 windows
                print(f"       - {window['title'][:50]}... ({window['width']}x{window['height']})")
        else:
            print(f"     Error: {response.get('error')}")
        
        # Get window at position
        print("  🪟 Getting window at (200, 200)")
        response = client.get_window_at_position(200, 200)
        if response['status'] == 'success':
            if response['data']:
                window = response['data']['window']
                print(f"     Window: {window['title']}")
            else:
                print("     No window at that position")
        else:
            print(f"     Error: {response.get('error')}")
        
        print(f"\n✅ Demo completed successfully!")
        
    except KeyboardInterrupt:
        print("\n⚠️  Demo interrupted by user")
    except Exception as e:
        print(f"\n❌ Demo failed: {e}")
    finally:
        client.disconnect()


def interactive_mode():
    """Interactive mode for manual testing"""
    
    print("🎮 System Controller Interactive Mode")
    print("Available commands:")
    print("  move X Y       - Move mouse to coordinates")
    print("  click BUTTON   - Click mouse button (Left/Right/Middle)")
    print("  type TEXT      - Type text")
    print("  key KEY        - Press key")
    print("  capture        - Capture screenshot")
    print("  displays       - List displays")
    print("  windows        - List windows")
    print("  quit           - Exit")
    print()
    
    client = SystemControllerClient(verify_ssl=False)
    
    try:
        client.connect()
        
        if not client.authenticate("admin", "changeme123!"):
            print("❌ Authentication failed")
            return
        
        while True:
            try:
                command = input("📝 Enter command: ").strip().split()
                if not command:
                    continue
                
                cmd = command[0].lower()
                
                if cmd == 'quit':
                    break
                elif cmd == 'move' and len(command) >= 3:
                    x, y = int(command[1]), int(command[2])
                    response = client.mouse_move(x, y)
                    print(f"   {response['status']}")
                elif cmd == 'click' and len(command) >= 2:
                    button = command[1].capitalize()
                    # Default position
                    response = client.mouse_click(button, 100, 100)
                    print(f"   {response['status']}")
                elif cmd == 'type' and len(command) >= 2:
                    text = ' '.join(command[1:])
                    response = client.type_text(text)
                    print(f"   {response['status']}")
                elif cmd == 'key' and len(command) >= 2:
                    key = command[1]
                    response = client.key_press(key)
                    print(f"   {response['status']}")
                elif cmd == 'capture':
                    response = client.capture_screen(0)
                    if response['status'] == 'success':
                        print(f"   Screenshot captured ({response['data'].get('data_size', 0)} bytes)")
                    else:
                        print(f"   Error: {response.get('error')}")
                elif cmd == 'displays':
                    response = client.get_displays()
                    if response['status'] == 'success':
                        for display in response['data']['displays']:
                            print(f"   Display {display['id']}: {display['name']} "
                                  f"({display['width']}x{display['height']})")
                    else:
                        print(f"   Error: {response.get('error')}")
                elif cmd == 'windows':
                    response = client.list_windows()
                    if response['status'] == 'success':
                        windows = response['data']['windows'][:10]  # Show first 10
                        for window in windows:
                            print(f"   {window['id']}: {window['title'][:60]}")
                    else:
                        print(f"   Error: {response.get('error')}")
                else:
                    print("   Unknown command or missing arguments")
                    
            except KeyboardInterrupt:
                break
            except ValueError:
                print("   Invalid arguments")
            except Exception as e:
                print(f"   Error: {e}")
    
    finally:
        client.disconnect()


if __name__ == "__main__":
    import sys
    
    if len(sys.argv) > 1 and sys.argv[1] == "--interactive":
        interactive_mode()
    else:
        demonstrate_basic_operations()
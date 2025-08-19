#!/usr/bin/env node
/**
 * System Controller WebSocket Client
 * 
 * Advanced WebSocket client demonstrating:
 * - Persistent connection management
 * - Event-driven architecture  
 * - Automatic reconnection
 * - Promise-based API
 * - Real-time communication
 * - Performance monitoring
 * 
 * Dependencies:
 *   npm install ws uuid
 * 
 * Usage:
 *   node websocket-client.js
 */

const WebSocket = require('ws');
const { v4: uuidv4 } = require('uuid');
const https = require('https');

class SystemControllerClient {
    constructor(url = 'wss://localhost:8080', options = {}) {
        this.url = url;
        this.options = {
            rejectUnauthorized: false, // For self-signed certificates
            ...options
        };
        
        this.ws = null;
        this.authToken = null;
        this.pendingCommands = new Map();
        this.eventListeners = new Map();
        this.connectionState = 'disconnected';
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 1000; // Start with 1 second
        
        // Performance metrics
        this.metrics = {
            commandsSent: 0,
            commandsSucceeded: 0,
            commandsFailed: 0,
            averageResponseTime: 0,
            connectionStartTime: null,
            lastCommandTime: null
        };
    }
    
    /**
     * Connect to the WebSocket server
     */
    connect() {
        return new Promise((resolve, reject) => {
            if (this.connectionState === 'connected') {
                resolve();
                return;
            }
            
            console.log(`🔌 Connecting to ${this.url}...`);
            this.connectionState = 'connecting';
            
            this.ws = new WebSocket(this.url, {
                rejectUnauthorized: this.options.rejectUnauthorized
            });
            
            this.ws.on('open', () => {
                console.log('✅ Connected to System Controller');
                this.connectionState = 'connected';
                this.reconnectAttempts = 0;
                this.reconnectDelay = 1000;
                this.metrics.connectionStartTime = Date.now();
                this.emit('connected');
                resolve();
            });
            
            this.ws.on('error', (error) => {
                console.error('❌ WebSocket error:', error.message);
                this.connectionState = 'error';
                this.emit('error', error);
                
                if (this.reconnectAttempts === 0) {
                    reject(error);
                }
            });
            
            this.ws.on('close', () => {
                console.log('🔌 Connection closed');
                this.connectionState = 'disconnected';
                this.emit('disconnected');
                this.handleReconnection();
            });
            
            this.ws.on('message', (data) => {
                try {
                    const message = JSON.parse(data.toString());
                    this.handleMessage(message);
                } catch (error) {
                    console.error('❌ Failed to parse message:', error);
                }
            });
        });
    }
    
    /**
     * Handle incoming messages
     */
    handleMessage(message) {
        // Handle command responses
        if (message.command_id && this.pendingCommands.has(message.command_id)) {
            const { resolve, reject, startTime } = this.pendingCommands.get(message.command_id);
            this.pendingCommands.delete(message.command_id);
            
            // Update metrics
            const responseTime = Date.now() - startTime;
            this.updateMetrics(responseTime, message.status === 'success');
            
            if (message.status === 'success') {
                resolve(message);
            } else {
                const error = new Error(message.error || 'Command failed');
                error.response = message;
                reject(error);
            }
            
            return;
        }
        
        // Handle server events
        if (message.type === 'event') {
            this.emit('serverEvent', message);
            return;
        }
        
        // Handle heartbeat responses
        if (message.type === 'pong') {
            this.emit('pong', message);
            return;
        }
        
        // Handle other messages
        this.emit('message', message);
    }
    
    /**
     * Send command to server
     */
    sendCommand(commandType, payload, commandId = null, timeout = 30000) {
        return new Promise((resolve, reject) => {
            if (this.connectionState !== 'connected') {
                reject(new Error('Not connected to server'));
                return;
            }
            
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
            
            // Store command for response handling
            const startTime = Date.now();
            this.pendingCommands.set(commandId, { resolve, reject, startTime });
            
            // Set timeout
            const timeoutHandle = setTimeout(() => {
                if (this.pendingCommands.has(commandId)) {
                    this.pendingCommands.delete(commandId);
                    this.updateMetrics(timeout, false);
                    reject(new Error(`Command timeout after ${timeout}ms`));
                }
            }, timeout);
            
            // Clear timeout when command completes
            const originalResolve = resolve;
            const originalReject = reject;
            
            resolve = (result) => {
                clearTimeout(timeoutHandle);
                originalResolve(result);
            };
            
            reject = (error) => {
                clearTimeout(timeoutHandle);
                originalReject(error);
            };
            
            // Update stored handlers
            this.pendingCommands.set(commandId, { resolve, reject, startTime });
            
            // Send command
            this.ws.send(JSON.stringify(command));
            this.metrics.commandsSent++;
            this.metrics.lastCommandTime = Date.now();
        });
    }
    
    /**
     * Authenticate with the server
     */
    async authenticate(username, password) {
        try {
            const response = await this.sendCommand('authenticate', {
                username: username,
                password: password
            });
            
            this.authToken = response.data.token;
            const expiresAt = response.data.expires_at;
            const permissions = response.data.permissions || [];
            
            console.log('🔑 Authentication successful');
            console.log(`   Token expires: ${expiresAt}`);
            console.log(`   Permissions: ${permissions.join(', ')}`);
            
            this.emit('authenticated', response.data);
            return true;
            
        } catch (error) {
            console.error('❌ Authentication failed:', error.message);
            return false;
        }
    }
    
    /**
     * Mouse movement
     */
    async mouseMove(x, y) {
        return this.sendCommand('mouse_move', {
            type: 'mouse_move',
            x: x,
            y: y
        });
    }
    
    /**
     * Mouse click
     */
    async mouseClick(button, x, y) {
        return this.sendCommand('mouse_click', {
            type: 'mouse_click',
            button: button,
            x: x,
            y: y
        });
    }
    
    /**
     * Mouse scroll
     */
    async mouseScroll(x, y) {
        return this.sendCommand('mouse_scroll', {
            type: 'mouse_scroll',
            x: x,
            y: y
        });
    }
    
    /**
     * Key press
     */
    async keyPress(key) {
        return this.sendCommand('key_press', {
            type: 'key_press',
            key: key
        });
    }
    
    /**
     * Key release
     */
    async keyRelease(key) {
        return this.sendCommand('key_release', {
            type: 'key_release',
            key: key
        });
    }
    
    /**
     * Type text
     */
    async typeText(text) {
        return this.sendCommand('type_text', {
            type: 'type_text',
            text: text
        });
    }
    
    /**
     * Capture screen
     */
    async captureScreen(displayId = 0, format = 'png', quality = 85) {
        return this.sendCommand('capture_screen', {
            type: 'capture_screen',
            display_id: displayId,
            format: format,
            quality: quality
        });
    }
    
    /**
     * Get displays
     */
    async getDisplays() {
        return this.sendCommand('get_displays', {
            type: 'get_displays'
        });
    }
    
    /**
     * List windows
     */
    async listWindows(includeMinimized = true) {
        return this.sendCommand('list_windows', {
            type: 'list_windows',
            include_minimized: includeMinimized
        });
    }
    
    /**
     * Get window at position
     */
    async getWindowAtPosition(x, y) {
        return this.sendCommand('get_window_info', {
            type: 'get_window_info',
            x: x,
            y: y
        });
    }
    
    /**
     * Send heartbeat/ping
     */
    async ping() {
        return this.sendCommand('ping', {});
    }
    
    /**
     * Subscribe to server events
     */
    async subscribe(events) {
        return this.sendCommand('subscribe', {
            events: events
        });
    }
    
    /**
     * Get system status
     */
    async getSystemStatus() {
        return this.sendCommand('system_status', {
            type: 'system_status'
        });
    }
    
    /**
     * Get performance metrics
     */
    async getPerformanceMetrics(includeHistory = false) {
        return this.sendCommand('performance_metrics', {
            type: 'performance_metrics',
            include_history: includeHistory
        });
    }
    
    /**
     * Handle reconnection logic
     */
    handleReconnection() {
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            console.error(`❌ Max reconnection attempts (${this.maxReconnectAttempts}) reached`);
            return;
        }
        
        this.reconnectAttempts++;
        const delay = Math.min(this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1), 30000);
        
        console.log(`🔄 Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);
        
        setTimeout(() => {
            this.connect().catch(error => {
                console.error('❌ Reconnection failed:', error.message);
            });
        }, delay);
    }
    
    /**
     * Update performance metrics
     */
    updateMetrics(responseTime, success) {
        if (success) {
            this.metrics.commandsSucceeded++;
            this.metrics.averageResponseTime = 
                (this.metrics.averageResponseTime + responseTime) / 2;
        } else {
            this.metrics.commandsFailed++;
        }
    }
    
    /**
     * Get client metrics
     */
    getMetrics() {
        const now = Date.now();
        const uptime = this.metrics.connectionStartTime ? 
            Math.floor((now - this.metrics.connectionStartTime) / 1000) : 0;
        const timeSinceLastCommand = this.metrics.lastCommandTime ?
            Math.floor((now - this.metrics.lastCommandTime) / 1000) : null;
        
        return {
            ...this.metrics,
            uptime: uptime,
            timeSinceLastCommand: timeSinceLastCommand,
            successRate: this.metrics.commandsSent > 0 ?
                (this.metrics.commandsSucceeded / this.metrics.commandsSent * 100).toFixed(1) : 0
        };
    }
    
    /**
     * Event emitter functionality
     */
    on(event, listener) {
        if (!this.eventListeners.has(event)) {
            this.eventListeners.set(event, []);
        }
        this.eventListeners.get(event).push(listener);
    }
    
    emit(event, data) {
        if (this.eventListeners.has(event)) {
            this.eventListeners.get(event).forEach(listener => {
                try {
                    listener(data);
                } catch (error) {
                    console.error(`❌ Error in event listener for '${event}':`, error);
                }
            });
        }
    }
    
    /**
     * Disconnect from server
     */
    disconnect() {
        if (this.ws) {
            this.connectionState = 'disconnecting';
            this.ws.close();
            this.ws = null;
        }
    }
    
    /**
     * Cleanup resources
     */
    destroy() {
        this.disconnect();
        this.pendingCommands.clear();
        this.eventListeners.clear();
        this.authToken = null;
    }
}

/**
 * Demo function showing various client capabilities
 */
async function demonstrateWebSocketClient() {
    console.log('🚀 System Controller WebSocket Client Demo');
    console.log('=' .repeat(50));
    
    const client = new SystemControllerClient('wss://localhost:8080');
    
    // Set up event listeners
    client.on('connected', () => {
        console.log('📡 Client connected');
    });
    
    client.on('disconnected', () => {
        console.log('📡 Client disconnected');
    });
    
    client.on('authenticated', (data) => {
        console.log('📡 Client authenticated:', data.permissions);
    });
    
    client.on('serverEvent', (event) => {
        console.log('📡 Server event:', event.event_type, event.data);
    });
    
    try {
        // Connect to server
        await client.connect();
        
        // Authenticate
        if (!await client.authenticate('admin', 'changeme123!')) {
            console.error('❌ Authentication failed');
            return;
        }
        
        // Test heartbeat
        console.log('\n💓 Testing heartbeat...');
        const pingResponse = await client.ping();
        console.log('   Pong received:', pingResponse.data.server_time);
        
        // Subscribe to events
        console.log('\n📡 Subscribing to events...');
        try {
            await client.subscribe(['window_focus_changed', 'display_configuration_changed']);
            console.log('   ✅ Subscribed to events');
        } catch (error) {
            console.log('   ⚠️  Event subscription not supported');
        }
        
        // Test mouse operations
        console.log('\n🖱️  Testing mouse operations...');
        await client.mouseMove(150, 150);
        console.log('   ✅ Mouse moved to (150, 150)');
        
        await new Promise(resolve => setTimeout(resolve, 200));
        
        await client.mouseClick('Left', 150, 150);
        console.log('   ✅ Left click at (150, 150)');
        
        await client.mouseScroll(0, -2);
        console.log('   ✅ Scrolled down');
        
        // Test keyboard operations
        console.log('\n⌨️  Testing keyboard operations...');
        await client.keyPress('a');
        console.log('   ✅ Pressed key "a"');
        
        await new Promise(resolve => setTimeout(resolve, 100));
        
        await client.typeText('Hello from WebSocket client!');
        console.log('   ✅ Typed text');
        
        await client.keyPress('Enter');
        console.log('   ✅ Pressed Enter');
        
        // Test display operations
        console.log('\n📺 Testing display operations...');
        const displaysResponse = await client.getDisplays();
        const displays = displaysResponse.data.displays;
        console.log(`   ✅ Found ${displays.length} display(s):`);
        
        displays.forEach((display, index) => {
            console.log(`      ${index + 1}. ${display.name}: ${display.width}x${display.height} ` +
                       `${display.is_primary ? '(Primary)' : ''}`);
        });
        
        // Test screen capture
        console.log('\n📸 Testing screen capture...');
        const captureResponse = await client.captureScreen(0, 'png', 75);
        console.log(`   ✅ Screenshot captured: ${captureResponse.data.data_size} bytes`);
        
        // Test window operations
        console.log('\n🪟 Testing window operations...');
        const windowsResponse = await client.listWindows();
        const windows = windowsResponse.data.windows;
        console.log(`   ✅ Found ${windows.length} window(s):`);
        
        windows.slice(0, 5).forEach((window, index) => {
            console.log(`      ${index + 1}. ${window.title.substring(0, 50)}...`);
        });
        
        // Test system status
        console.log('\n📊 Getting system status...');
        try {
            const statusResponse = await client.getSystemStatus();
            console.log('   ✅ System status retrieved');
            console.log(`      Status: ${statusResponse.data.status}`);
            console.log(`      Uptime: ${statusResponse.data.uptime}ms`);
        } catch (error) {
            console.log('   ⚠️  System status not available');
        }
        
        // Show client metrics
        console.log('\n📈 Client Performance Metrics:');
        const metrics = client.getMetrics();
        console.log(`   Commands sent: ${metrics.commandsSent}`);
        console.log(`   Success rate: ${metrics.successRate}%`);
        console.log(`   Average response time: ${metrics.averageResponseTime.toFixed(0)}ms`);
        console.log(`   Uptime: ${metrics.uptime}s`);
        
        console.log('\n✅ WebSocket demo completed successfully!');
        
    } catch (error) {
        console.error('❌ Demo failed:', error.message);
    } finally {
        // Cleanup
        await new Promise(resolve => setTimeout(resolve, 1000));
        client.destroy();
    }
}

/**
 * Interactive mode for manual testing
 */
async function interactiveMode() {
    const readline = require('readline');
    const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout
    });
    
    const question = (prompt) => new Promise(resolve => rl.question(prompt, resolve));
    
    console.log('🎮 Interactive WebSocket Mode');
    console.log('Commands: move, click, type, key, capture, displays, windows, ping, status, metrics, quit');
    
    const client = new SystemControllerClient('wss://localhost:8080');
    
    try {
        await client.connect();
        
        if (!await client.authenticate('admin', 'changeme123!')) {
            console.error('❌ Authentication failed');
            return;
        }
        
        while (true) {
            const input = await question('\n📝 Command: ');
            const parts = input.trim().split(' ');
            const cmd = parts[0].toLowerCase();
            
            try {
                if (cmd === 'quit') {
                    break;
                } else if (cmd === 'move' && parts.length >= 3) {
                    await client.mouseMove(parseInt(parts[1]), parseInt(parts[2]));
                    console.log('✅ Mouse moved');
                } else if (cmd === 'click' && parts.length >= 2) {
                    await client.mouseClick(parts[1].charAt(0).toUpperCase() + parts[1].slice(1), 100, 100);
                    console.log('✅ Mouse clicked');
                } else if (cmd === 'type' && parts.length >= 2) {
                    await client.typeText(parts.slice(1).join(' '));
                    console.log('✅ Text typed');
                } else if (cmd === 'key' && parts.length >= 2) {
                    await client.keyPress(parts[1]);
                    console.log('✅ Key pressed');
                } else if (cmd === 'capture') {
                    const response = await client.captureScreen();
                    console.log(`✅ Screenshot: ${response.data.data_size} bytes`);
                } else if (cmd === 'displays') {
                    const response = await client.getDisplays();
                    response.data.displays.forEach(d => 
                        console.log(`   ${d.id}: ${d.name} (${d.width}x${d.height})`));
                } else if (cmd === 'windows') {
                    const response = await client.listWindows();
                    response.data.windows.slice(0, 10).forEach(w => 
                        console.log(`   ${w.id}: ${w.title.substring(0, 60)}`));
                } else if (cmd === 'ping') {
                    const response = await client.ping();
                    console.log(`✅ Pong: ${response.data.server_time}`);
                } else if (cmd === 'status') {
                    const response = await client.getSystemStatus();
                    console.log(`✅ Status: ${response.data.status}, Uptime: ${response.data.uptime}ms`);
                } else if (cmd === 'metrics') {
                    const metrics = client.getMetrics();
                    console.log(`✅ Metrics: ${metrics.commandsSent} sent, ${metrics.successRate}% success, ${metrics.averageResponseTime.toFixed(0)}ms avg`);
                } else {
                    console.log('❓ Unknown command or missing arguments');
                }
            } catch (error) {
                console.error('❌ Command failed:', error.message);
            }
        }
        
    } finally {
        rl.close();
        client.destroy();
    }
}

// Main execution
if (require.main === module) {
    const args = process.argv.slice(2);
    
    if (args.includes('--interactive')) {
        interactiveMode().catch(console.error);
    } else {
        demonstrateWebSocketClient().catch(console.error);
    }
}

// Export for use as module
module.exports = SystemControllerClient;
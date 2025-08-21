/**
 * System Controller WebSocket Client Library
 * 
 * Core WebSocket client implementation with:
 * - Connection management and automatic reconnection
 * - Event-driven architecture
 * - Performance monitoring
 * - Error handling and recovery
 */

const WebSocket = require('ws');
const { v4: uuidv4 } = require('uuid');
const EventEmitter = require('events');

class SystemControllerClient extends EventEmitter {
    constructor(url = 'wss://localhost:8080', options = {}) {
        super();
        
        this.url = url;
        this.options = {
            rejectUnauthorized: false, // For self-signed certificates
            reconnect: true,
            reconnectInterval: 1000,
            maxReconnectAttempts: 5,
            commandTimeout: 30000,
            ...options
        };
        
        this.ws = null;
        this.authToken = null;
        this.pendingCommands = new Map();
        this.connectionState = 'disconnected';
        this.reconnectAttempts = 0;
        this.reconnectTimer = null;
        
        // Performance metrics
        this.metrics = {
            commandsSent: 0,
            commandsSucceeded: 0,
            commandsFailed: 0,
            averageResponseTime: 0,
            connectionStartTime: null,
            lastCommandTime: null,
            totalResponseTime: 0
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
            
            this.connectionState = 'connecting';
            this.emit('connecting');
            
            try {
                this.ws = new WebSocket(this.url, {
                    rejectUnauthorized: this.options.rejectUnauthorized
                });
                
                this.ws.on('open', () => {
                    this.connectionState = 'connected';
                    this.reconnectAttempts = 0;
                    this.metrics.connectionStartTime = Date.now();
                    this.emit('connected');
                    resolve();
                });
                
                this.ws.on('error', (error) => {
                    this.connectionState = 'error';
                    this.emit('error', error);
                    
                    if (this.reconnectAttempts === 0) {
                        reject(error);
                    }
                });
                
                this.ws.on('close', (code, reason) => {
                    this.connectionState = 'disconnected';
                    this.emit('disconnected', { code, reason: reason?.toString() });
                    
                    // Clear any pending commands
                    this.pendingCommands.forEach(({ reject }) => {
                        reject(new Error('Connection closed'));
                    });
                    this.pendingCommands.clear();
                    
                    // Handle reconnection
                    if (this.options.reconnect) {
                        this.handleReconnection();
                    }
                });
                
                this.ws.on('message', (data) => {
                    try {
                        const message = JSON.parse(data.toString());
                        this.handleMessage(message);
                    } catch (error) {
                        this.emit('error', new Error(`Failed to parse message: ${error.message}`));
                    }
                });
                
            } catch (error) {
                this.connectionState = 'error';
                reject(error);
            }
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
    sendCommand(commandType, payload, commandId = null, timeout = null) {
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
            const commandTimeout = timeout || this.options.commandTimeout;
            
            this.pendingCommands.set(commandId, { resolve, reject, startTime });
            
            // Set timeout
            const timeoutHandle = setTimeout(() => {
                if (this.pendingCommands.has(commandId)) {
                    this.pendingCommands.delete(commandId);
                    this.updateMetrics(commandTimeout, false);
                    reject(new Error(`Command timeout after ${commandTimeout}ms`));
                }
            }, commandTimeout);
            
            // Wrap handlers to clear timeout
            const originalResolve = resolve;
            const originalReject = reject;
            
            const wrappedResolve = (result) => {
                clearTimeout(timeoutHandle);
                originalResolve(result);
            };
            
            const wrappedReject = (error) => {
                clearTimeout(timeoutHandle);
                originalReject(error);
            };
            
            // Update stored handlers
            this.pendingCommands.set(commandId, { 
                resolve: wrappedResolve, 
                reject: wrappedReject, 
                startTime 
            });
            
            // Send command
            try {
                this.ws.send(JSON.stringify(command));
                this.metrics.commandsSent++;
                this.metrics.lastCommandTime = Date.now();
            } catch (error) {
                this.pendingCommands.delete(commandId);
                clearTimeout(timeoutHandle);
                reject(error);
            }
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
            this.emit('authenticated', response.data);
            return true;
            
        } catch (error) {
            this.emit('authenticationFailed', error);
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
        if (this.reconnectAttempts >= this.options.maxReconnectAttempts) {
            this.emit('reconnectionFailed');
            return;
        }
        
        this.reconnectAttempts++;
        const delay = Math.min(
            this.options.reconnectInterval * Math.pow(2, this.reconnectAttempts - 1),
            30000
        );
        
        this.emit('reconnecting', this.reconnectAttempts, delay);
        
        this.reconnectTimer = setTimeout(() => {
            this.connect().catch(error => {
                this.emit('reconnectionError', error);
            });
        }, delay);
    }
    
    /**
     * Update performance metrics
     */
    updateMetrics(responseTime, success) {
        if (success) {
            this.metrics.commandsSucceeded++;
            this.metrics.totalResponseTime += responseTime;
            this.metrics.averageResponseTime = 
                this.metrics.totalResponseTime / this.metrics.commandsSucceeded;
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
                (this.metrics.commandsSucceeded / this.metrics.commandsSent * 100) : 0,
            connectionState: this.connectionState,
            pendingCommands: this.pendingCommands.size,
            reconnectAttempts: this.reconnectAttempts
        };
    }
    
    /**
     * Reset metrics
     */
    resetMetrics() {
        this.metrics = {
            commandsSent: 0,
            commandsSucceeded: 0,
            commandsFailed: 0,
            averageResponseTime: 0,
            connectionStartTime: this.metrics.connectionStartTime,
            lastCommandTime: null,
            totalResponseTime: 0
        };
    }
    
    /**
     * Disconnect from server
     */
    disconnect() {
        this.options.reconnect = false; // Disable auto-reconnection
        
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }
        
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.connectionState = 'disconnecting';
            this.ws.close();
        }
        
        this.connectionState = 'disconnected';
    }
    
    /**
     * Destroy client and cleanup resources
     */
    destroy() {
        this.disconnect();
        
        // Clear all pending commands
        this.pendingCommands.forEach(({ reject }) => {
            reject(new Error('Client destroyed'));
        });
        this.pendingCommands.clear();
        
        // Remove all event listeners
        this.removeAllListeners();
        
        // Clear references
        this.ws = null;
        this.authToken = null;
    }
    
    /**
     * Get connection state
     */
    getConnectionState() {
        return this.connectionState;
    }
    
    /**
     * Check if client is connected
     */
    isConnected() {
        return this.connectionState === 'connected';
    }
    
    /**
     * Check if client is authenticated
     */
    isAuthenticated() {
        return this.authToken !== null;
    }
}

module.exports = SystemControllerClient;
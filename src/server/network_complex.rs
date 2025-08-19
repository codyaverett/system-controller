use crate::platform::traits::PlatformController;
use crate::protocol::messages::*;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;
use tokio::time::Duration;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use chrono::Utc;

/// Network server configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub bind_address: String,
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub enable_websocket: bool,
    pub buffer_size: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".to_string(),
            max_connections: 100,
            connection_timeout: Duration::from_secs(60),
            enable_websocket: false,
            buffer_size: 8192,
        }
    }
}

/// Network server for handling remote control connections
pub struct NetworkServer {
    platform: Arc<RwLock<Box<dyn PlatformController + Send + Sync>>>,
    config: NetworkConfig,
    listener: Option<TcpListener>,
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    is_running: Arc<RwLock<bool>>,
}

#[derive(Debug)]
struct ConnectionInfo {
    addr: SocketAddr,
    connected_at: chrono::DateTime<Utc>,
    is_websocket: bool,
}

impl NetworkServer {
    /// Create a new network server with default configuration
    pub fn new(
        platform: Box<dyn PlatformController + Send + Sync>,
        bind_address: String,
    ) -> Self {
        let config = NetworkConfig {
            bind_address,
            ..Default::default()
        };
        
        Self {
            platform: Arc::new(RwLock::new(platform)),
            config,
            listener: None,
            connections: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Create a new network server with custom configuration
    pub fn with_config(
        platform: Box<dyn PlatformController + Send + Sync>,
        config: NetworkConfig,
    ) -> Self {
        Self {
            platform: Arc::new(RwLock::new(platform)),
            config,
            listener: None,
            connections: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the network server
    pub async fn start(&mut self) -> Result<()> {
        let listener = TcpListener::bind(&self.config.bind_address).await
            .map_err(|e| anyhow!("Failed to bind to {}: {}", self.config.bind_address, e))?;
        
        tracing::info!("Server started on {}", self.config.bind_address);
        
        self.listener = Some(listener);
        *self.is_running.write().await = true;
        
        Ok(())
    }

    /// Get the local address the server is bound to
    pub async fn local_addr(&self) -> Result<SocketAddr> {
        match &self.listener {
            Some(listener) => Ok(listener.local_addr()?),
            None => Err(anyhow!("Server not started")),
        }
    }

    /// Get the number of active connections
    pub async fn active_connections(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Get server configuration
    pub async fn get_config(&self) -> NetworkConfig {
        self.config.clone()
    }

    /// Shutdown the server gracefully
    pub async fn shutdown(&self) -> Result<()> {
        *self.is_running.write().await = false;
        self.connections.write().await.clear();
        tracing::info!("Server shutdown completed");
        Ok(())
    }

    /// Handle incoming client connections (this would be called in a loop)
    pub async fn accept_connections(&self) -> Result<()> {
        let listener = self.listener.as_ref()
            .ok_or_else(|| anyhow!("Server not started"))?;

        loop {
            if !*self.is_running.read().await {
                break;
            }

            match listener.accept().await {
                Ok((stream, addr)) => {
                    // Check connection limit
                    let current_connections = self.active_connections().await;
                    if current_connections >= self.config.max_connections {
                        tracing::warn!("Connection limit reached, rejecting connection from {}", addr);
                        // Send rejection message and close
                        self.send_connection_rejected(stream).await?;
                        continue;
                    }

                    // Add connection
                    let connection_id = format!("{}:{}", addr.ip(), addr.port());
                    self.connections.write().await.insert(
                        connection_id.clone(),
                        ConnectionInfo {
                            addr,
                            connected_at: Utc::now(),
                            is_websocket: false,
                        },
                    );

                    // Handle connection in separate task
                    let platform_clone = self.platform.clone();
                    let connections_clone = self.connections.clone();
                    let config_clone = self.config.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(
                            stream,
                            addr,
                            platform_clone,
                            connections_clone,
                            config_clone,
                        ).await {
                            tracing::error!("Connection error for {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to accept connection: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Handle a single client connection
    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        platform: Arc<RwLock<Box<dyn PlatformController + Send + Sync>>>,
        connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
        config: NetworkConfig,
    ) -> Result<()> {
        let connection_id = format!("{}:{}", addr.ip(), addr.port());
        tracing::info!("Handling connection from {}", addr);

        let (mut reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(&mut reader);
        let mut buffer = String::new();

        loop {
            buffer.clear();
            
            match buf_reader.read_line(&mut buffer).await {
                Ok(0) => {
                    // Client disconnected
                    tracing::info!("Client {} disconnected", addr);
                    break;
                }
                Ok(_) => {
                    let message = buffer.trim();
                    
                    // Check for WebSocket upgrade
                    if config.enable_websocket && message.starts_with("GET") && message.contains("websocket") {
                        Self::handle_websocket_upgrade(&mut writer, message).await?;
                        continue;
                    }

                    // Handle regular command
                    match Self::process_command(message, &platform).await {
                        Ok(response) => {
                            let response_json = serde_json::to_string(&response)?;
                            writer.write_all(response_json.as_bytes()).await?;
                            writer.write_all(b"\n").await?;
                            
                            // Handle binary data if needed
                            if let Some(ResponseData::ScreenCapture { size, .. }) = &response.data {
                                // Send binary data (placeholder implementation)
                                let binary_data = vec![0u8; *size];
                                writer.write_all(&binary_data).await?;
                            }
                        }
                        Err(e) => {
                            let error_response = Response {
                                command_id: "unknown".to_string(),
                                status: ResponseStatus::Error,
                                error: Some(format!("Invalid JSON: {}", e)),
                                data: None,
                                timestamp: Utc::now().to_rfc3339(),
                            };
                            let response_json = serde_json::to_string(&error_response)?;
                            writer.write_all(response_json.as_bytes()).await?;
                            writer.write_all(b"\n").await?;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error reading from client {}: {}", addr, e);
                    break;
                }
            }
        }

        // Clean up connection
        connections.write().await.remove(&connection_id);
        Ok(())
    }

    /// Process a command and return a response
    async fn process_command(
        message: &str,
        platform: &Arc<RwLock<Box<dyn PlatformController + Send + Sync>>>,
    ) -> Result<Response> {
        let command: Command = serde_json::from_str(message)?;
        command.validate()?;

        let platform_guard = platform.read().await;
        
        let result = match command.payload {
            CommandPayload::MouseMove { x, y } => {
                platform_guard.mouse_move(x, y)
            }
            CommandPayload::MouseClick { button, x, y } => {
                platform_guard.mouse_click(button, x, y)
            }
            CommandPayload::MouseScroll { x, y } => {
                platform_guard.mouse_scroll(x, y)
            }
            CommandPayload::KeyPress { key } => {
                platform_guard.key_press(key)
            }
            CommandPayload::KeyRelease { key } => {
                platform_guard.key_release(key)
            }
            CommandPayload::TypeText { text } => {
                platform_guard.type_text(text)
            }
            CommandPayload::CaptureScreen { display_id } => {
                match platform_guard.capture_screen(display_id) {
                    Ok(data) => {
                        return Ok(Response {
                            command_id: command.id,
                            status: ResponseStatus::Success,
                            error: None,
                            data: Some(ResponseData::ScreenCapture {
                                size: data.len(),
                                format: "png".to_string(),
                            }),
                            timestamp: Utc::now().to_rfc3339(),
                        });
                    }
                    Err(e) => Err(e),
                }
            }
            _ => {
                // TODO: Implement other command types
                Err(anyhow!("Command not implemented"))
            }
        };

        match result {
            Ok(_) => Ok(Response {
                command_id: command.id,
                status: ResponseStatus::Success,
                error: None,
                data: None,
                timestamp: Utc::now().to_rfc3339(),
            }),
            Err(e) => Ok(Response {
                command_id: command.id,
                status: ResponseStatus::Error,
                error: Some(e.to_string()),
                data: None,
                timestamp: Utc::now().to_rfc3339(),
            }),
        }
    }

    /// Handle WebSocket upgrade request
    async fn handle_websocket_upgrade(writer: &mut tokio::net::tcp::OwnedWriteHalf, request: &str) -> Result<()> {
        // Simple WebSocket upgrade response (basic implementation)
        if request.contains("websocket") {
            let response = 
                "HTTP/1.1 101 Switching Protocols\r\n\
                 Upgrade: websocket\r\n\
                 Connection: Upgrade\r\n\
                 Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
            
            writer.write_all(response.as_bytes()).await?;
        }
        
        Ok(())
    }

    /// Send connection rejected message
    async fn send_connection_rejected(&self, mut stream: TcpStream) -> Result<()> {
        let rejection_response = Response {
            command_id: "connection".to_string(),
            status: ResponseStatus::Error,
            error: Some("Connection limit reached".to_string()),
            data: None,
            timestamp: Utc::now().to_rfc3339(),
        };
        
        let response_json = serde_json::to_string(&rejection_response)?;
        stream.write_all(response_json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        
        Ok(())
    }
}

// Ensure thread safety
unsafe impl Send for NetworkServer {}
unsafe impl Sync for NetworkServer {}
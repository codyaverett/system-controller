use anyhow::{Result, anyhow};
use crate::protocol::messages::{Command, CommandType};
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct Role {
    pub name: String,
    pub permissions: Vec<String>,
    pub valid_from: Option<SystemTime>,
    pub valid_until: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct Permission {
    pub name: String,
    pub description: String,
    pub resource_pattern: String,
}

pub struct PermissionManager {
    roles: RwLock<HashMap<String, Role>>,
    user_roles: RwLock<HashMap<String, Vec<String>>>,
    permissions: RwLock<HashMap<String, Permission>>,
}

impl PermissionManager {
    pub async fn new() -> Result<Self> {
        let manager = Self {
            roles: RwLock::new(HashMap::new()),
            user_roles: RwLock::new(HashMap::new()),
            permissions: RwLock::new(HashMap::new()),
        };
        
        // Initialize default permissions
        manager.initialize_default_permissions().await?;
        
        Ok(manager)
    }

    async fn initialize_default_permissions(&self) -> Result<()> {
        let mut permissions = self.permissions.write().await;
        
        permissions.insert("mouse_control".to_string(), Permission {
            name: "mouse_control".to_string(),
            description: "Control mouse movement and clicks".to_string(),
            resource_pattern: "input:mouse:*".to_string(),
        });
        
        permissions.insert("keyboard_control".to_string(), Permission {
            name: "keyboard_control".to_string(),
            description: "Control keyboard input".to_string(),
            resource_pattern: "input:keyboard:*".to_string(),
        });
        
        permissions.insert("screen_capture".to_string(), Permission {
            name: "screen_capture".to_string(),
            description: "Capture screen content".to_string(),
            resource_pattern: "display:capture:*".to_string(),
        });
        
        permissions.insert("window_management".to_string(), Permission {
            name: "window_management".to_string(),
            description: "Manage windows and displays".to_string(),
            resource_pattern: "display:window:*".to_string(),
        });
        
        permissions.insert("system_control".to_string(), Permission {
            name: "system_control".to_string(),
            description: "System-level control operations".to_string(),
            resource_pattern: "system:*".to_string(),
        });
        
        Ok(())
    }

    pub async fn create_role(&mut self, name: &str, permissions: Vec<String>) -> Result<()> {
        let role = Role {
            name: name.to_string(),
            permissions,
            valid_from: None,
            valid_until: None,
        };
        
        let mut roles = self.roles.write().await;
        roles.insert(name.to_string(), role);
        
        Ok(())
    }

    pub async fn create_time_restricted_role(
        &mut self,
        name: &str,
        permissions: Vec<String>,
        valid_from: SystemTime,
        valid_until: SystemTime,
    ) -> Result<()> {
        let role = Role {
            name: name.to_string(),
            permissions,
            valid_from: Some(valid_from),
            valid_until: Some(valid_until),
        };
        
        let mut roles = self.roles.write().await;
        roles.insert(name.to_string(), role);
        
        Ok(())
    }

    pub async fn assign_role_to_user(&mut self, username: &str, role_name: &str) -> Result<()> {
        let roles = self.roles.read().await;
        if !roles.contains_key(role_name) {
            return Err(anyhow!("Role '{}' does not exist", role_name));
        }
        drop(roles);
        
        let mut user_roles = self.user_roles.write().await;
        user_roles.entry(username.to_string())
            .or_insert_with(Vec::new)
            .push(role_name.to_string());
        
        Ok(())
    }

    pub async fn check_permission(&self, username: &str, permission: &str) -> Result<bool> {
        let user_roles = self.user_roles.read().await;
        let empty_vec = vec![];
        let user_role_names = user_roles.get(username).unwrap_or(&empty_vec);
        
        let roles = self.roles.read().await;
        let now = SystemTime::now();
        
        for role_name in user_role_names {
            if let Some(role) = roles.get(role_name) {
                // Check time restrictions
                if let Some(valid_from) = role.valid_from {
                    if now < valid_from {
                        continue;
                    }
                }
                
                if let Some(valid_until) = role.valid_until {
                    if now > valid_until {
                        continue;
                    }
                }
                
                // Check permissions
                if role.permissions.contains(&permission.to_string()) || 
                   role.permissions.contains(&"*".to_string()) {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }

    pub async fn get_user_permissions(&self, username: &str) -> Result<Vec<String>> {
        let user_roles = self.user_roles.read().await;
        let empty_vec = vec![];
        let user_role_names = user_roles.get(username).unwrap_or(&empty_vec);
        
        let roles = self.roles.read().await;
        let now = SystemTime::now();
        let mut permissions = Vec::new();
        
        for role_name in user_role_names {
            if let Some(role) = roles.get(role_name) {
                // Check time restrictions
                if let Some(valid_from) = role.valid_from {
                    if now < valid_from {
                        continue;
                    }
                }
                
                if let Some(valid_until) = role.valid_until {
                    if now > valid_until {
                        continue;
                    }
                }
                
                permissions.extend(role.permissions.clone());
            }
        }
        
        permissions.sort();
        permissions.dedup();
        Ok(permissions)
    }

    pub async fn authorize_command(&self, username: &str, command: &Command) -> Result<()> {
        let required_permission = match command.command_type {
            CommandType::MouseMove | CommandType::MouseClick | CommandType::MouseScroll => "mouse_control",
            CommandType::KeyPress | CommandType::KeyRelease | CommandType::TypeText => "keyboard_control",
            CommandType::CaptureScreen => "screen_capture",
            CommandType::GetDisplays | CommandType::GetWindowInfo | CommandType::ListWindows => "window_management",
        };
        
        let has_permission = self.check_permission(username, required_permission).await?;
        
        if has_permission {
            Ok(())
        } else {
            Err(anyhow!("permission denied: user '{}' lacks '{}' permission for command type '{:?}'", 
                       username, required_permission, command.command_type))
        }
    }

    pub async fn get_role_info(&self, role_name: &str) -> Result<Role> {
        let roles = self.roles.read().await;
        roles.get(role_name)
            .cloned()
            .ok_or_else(|| anyhow!("Role '{}' not found", role_name))
    }

    pub async fn remove_role(&mut self, role_name: &str) -> Result<()> {
        let mut roles = self.roles.write().await;
        roles.remove(role_name)
            .ok_or_else(|| anyhow!("Role '{}' not found", role_name))?;
        
        // Remove role from all users
        let mut user_roles = self.user_roles.write().await;
        for user_role_list in user_roles.values_mut() {
            user_role_list.retain(|r| r != role_name);
        }
        
        Ok(())
    }

    pub async fn remove_user_role(&mut self, username: &str, role_name: &str) -> Result<()> {
        let mut user_roles = self.user_roles.write().await;
        if let Some(user_role_list) = user_roles.get_mut(username) {
            user_role_list.retain(|r| r != role_name);
            if user_role_list.is_empty() {
                user_roles.remove(username);
            }
        }
        
        Ok(())
    }
}
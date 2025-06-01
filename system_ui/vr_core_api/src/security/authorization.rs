//! Authorization module for the VR Core API.
//!
//! This module provides authorization functionality for the VR Core API,
//! including role-based access control, permissions, and policy enforcement.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, RwLock};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::authentication::{AuthToken, User};

/// Authorization error
#[derive(Debug, Error)]
pub enum AuthzError {
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Role not found
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    
    /// User not found
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    /// Internal error
    #[error("Internal authorization error: {0}")]
    InternalError(String),
}

/// Authorization result
pub type Result<T> = std::result::Result<T, AuthzError>;

/// Permission
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Resource type
    pub resource_type: String,
    
    /// Resource ID (optional, * for all)
    pub resource_id: String,
    
    /// Action
    pub action: String,
}

impl Permission {
    /// Create a new Permission
    pub fn new(resource_type: &str, resource_id: &str, action: &str) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            action: action.to_string(),
        }
    }
    
    /// Create a permission for all resources of a type
    pub fn for_all(resource_type: &str, action: &str) -> Self {
        Self::new(resource_type, "*", action)
    }
    
    /// Check if this permission includes another permission
    pub fn includes(&self, other: &Permission) -> bool {
        if self.resource_type != other.resource_type {
            return false;
        }
        
        if self.action != other.action && self.action != "*" {
            return false;
        }
        
        if self.resource_id != other.resource_id && self.resource_id != "*" {
            return false;
        }
        
        true
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.resource_type, self.resource_id, self.action)
    }
}

/// Role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Role name
    pub name: String,
    
    /// Role description
    pub description: String,
    
    /// Permissions
    pub permissions: HashSet<Permission>,
}

impl Role {
    /// Create a new Role
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            permissions: HashSet::new(),
        }
    }
    
    /// Add permission
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }
    
    /// Remove permission
    pub fn remove_permission(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }
    
    /// Check if role has permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        for p in &self.permissions {
            if p.includes(permission) {
                return true;
            }
        }
        
        false
    }
}

/// Authorization provider trait
pub trait AuthorizationProvider: Send + Sync {
    /// Check if user has permission
    fn has_permission(&self, user_id: &str, permission: &Permission) -> Result<bool>;
    
    /// Get user roles
    fn get_user_roles(&self, user_id: &str) -> Result<Vec<Role>>;
    
    /// Assign role to user
    fn assign_role(&self, user_id: &str, role_name: &str) -> Result<()>;
    
    /// Remove role from user
    fn remove_role(&self, user_id: &str, role_name: &str) -> Result<()>;
    
    /// Create role
    fn create_role(&self, role: Role) -> Result<()>;
    
    /// Delete role
    fn delete_role(&self, role_name: &str) -> Result<()>;
    
    /// Get role
    fn get_role(&self, role_name: &str) -> Result<Role>;
    
    /// Get all roles
    fn get_all_roles(&self) -> Result<Vec<Role>>;
}

/// Local authorization provider
pub struct LocalAuthzProvider {
    /// Roles
    roles: RwLock<HashMap<String, Role>>,
    
    /// User roles
    user_roles: RwLock<HashMap<String, HashSet<String>>>,
}

impl LocalAuthzProvider {
    /// Create a new LocalAuthzProvider
    pub fn new() -> Self {
        let mut provider = Self {
            roles: RwLock::new(HashMap::new()),
            user_roles: RwLock::new(HashMap::new()),
        };
        
        // Create default roles
        provider.create_default_roles().unwrap();
        
        provider
    }
    
    /// Create default roles
    fn create_default_roles(&self) -> Result<()> {
        // Admin role
        let mut admin_role = Role::new("admin", "Administrator with full access");
        admin_role.add_permission(Permission::new("*", "*", "*"));
        self.create_role(admin_role)?;
        
        // User role
        let mut user_role = Role::new("user", "Standard user");
        user_role.add_permission(Permission::new("profile", "${user_id}", "read"));
        user_role.add_permission(Permission::new("profile", "${user_id}", "update"));
        user_role.add_permission(Permission::new("device", "*", "read"));
        self.create_role(user_role)?;
        
        // Guest role
        let mut guest_role = Role::new("guest", "Guest user with limited access");
        guest_role.add_permission(Permission::new("device", "*", "read"));
        self.create_role(guest_role)?;
        
        Ok(())
    }
}

impl AuthorizationProvider for LocalAuthzProvider {
    fn has_permission(&self, user_id: &str, permission: &Permission) -> Result<bool> {
        // Get user roles
        let user_roles = self.get_user_roles(user_id)?;
        
        // Check if any role has the permission
        for role in user_roles {
            if role.has_permission(permission) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    fn get_user_roles(&self, user_id: &str) -> Result<Vec<Role>> {
        let user_roles = self.user_roles.read().unwrap();
        let roles = self.roles.read().unwrap();
        
        let role_names = user_roles.get(user_id).cloned().unwrap_or_default();
        
        let mut result = Vec::new();
        for role_name in role_names {
            if let Some(role) = roles.get(&role_name) {
                result.push(role.clone());
            }
        }
        
        Ok(result)
    }
    
    fn assign_role(&self, user_id: &str, role_name: &str) -> Result<()> {
        // Check if role exists
        {
            let roles = self.roles.read().unwrap();
            if !roles.contains_key(role_name) {
                return Err(AuthzError::RoleNotFound(role_name.to_string()));
            }
        }
        
        // Assign role to user
        let mut user_roles = self.user_roles.write().unwrap();
        let user_role_set = user_roles.entry(user_id.to_string()).or_insert_with(HashSet::new);
        user_role_set.insert(role_name.to_string());
        
        Ok(())
    }
    
    fn remove_role(&self, user_id: &str, role_name: &str) -> Result<()> {
        let mut user_roles = self.user_roles.write().unwrap();
        
        if let Some(user_role_set) = user_roles.get_mut(user_id) {
            user_role_set.remove(role_name);
            Ok(())
        } else {
            Err(AuthzError::UserNotFound(user_id.to_string()))
        }
    }
    
    fn create_role(&self, role: Role) -> Result<()> {
        let mut roles = self.roles.write().unwrap();
        roles.insert(role.name.clone(), role);
        Ok(())
    }
    
    fn delete_role(&self, role_name: &str) -> Result<()> {
        let mut roles = self.roles.write().unwrap();
        
        if roles.remove(role_name).is_none() {
            return Err(AuthzError::RoleNotFound(role_name.to_string()));
        }
        
        // Remove role from all users
        let mut user_roles = self.user_roles.write().unwrap();
        for user_role_set in user_roles.values_mut() {
            user_role_set.remove(role_name);
        }
        
        Ok(())
    }
    
    fn get_role(&self, role_name: &str) -> Result<Role> {
        let roles = self.roles.read().unwrap();
        
        roles.get(role_name)
            .cloned()
            .ok_or_else(|| AuthzError::RoleNotFound(role_name.to_string()))
    }
    
    fn get_all_roles(&self) -> Result<Vec<Role>> {
        let roles = self.roles.read().unwrap();
        
        Ok(roles.values().cloned().collect())
    }
}

/// Policy enforcer
pub struct PolicyEnforcer {
    /// Authorization provider
    authz_provider: Arc<dyn AuthorizationProvider>,
}

impl PolicyEnforcer {
    /// Create a new PolicyEnforcer
    pub fn new(authz_provider: Arc<dyn AuthorizationProvider>) -> Self {
        Self {
            authz_provider,
        }
    }
    
    /// Enforce policy
    pub fn enforce(&self, user_id: &str, permission: &Permission) -> Result<()> {
        if self.authz_provider.has_permission(user_id, permission)? {
            Ok(())
        } else {
            Err(AuthzError::PermissionDenied(format!(
                "User {} does not have permission {}",
                user_id, permission
            )))
        }
    }
    
    /// Enforce policy with token
    pub fn enforce_with_token(&self, token: &AuthToken, permission: &Permission) -> Result<()> {
        self.enforce(&token.user_id, permission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_includes() {
        let p1 = Permission::new("device", "*", "read");
        let p2 = Permission::new("device", "123", "read");
        let p3 = Permission::new("profile", "123", "write");
        
        assert!(p1.includes(&p2));
        assert!(!p2.includes(&p1));
        assert!(!p1.includes(&p3));
    }
    
    #[test]
    fn test_role_has_permission() {
        let mut role = Role::new("test", "Test role");
        role.add_permission(Permission::new("device", "*", "read"));
        
        assert!(role.has_permission(&Permission::new("device", "123", "read")));
        assert!(!role.has_permission(&Permission::new("device", "123", "write")));
        assert!(!role.has_permission(&Permission::new("profile", "123", "read")));
    }
    
    #[test]
    fn test_local_authz_provider() {
        let provider = LocalAuthzProvider::new();
        
        // Check default roles
        let admin_role = provider.get_role("admin").unwrap();
        assert_eq!(admin_role.name, "admin");
        
        let user_role = provider.get_role("user").unwrap();
        assert_eq!(user_role.name, "user");
        
        let guest_role = provider.get_role("guest").unwrap();
        assert_eq!(guest_role.name, "guest");
        
        // Assign roles to user
        provider.assign_role("user123", "user").unwrap();
        
        // Check user roles
        let roles = provider.get_user_roles("user123").unwrap();
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "user");
        
        // Check permissions
        assert!(provider.has_permission("user123", &Permission::new("profile", "user123", "read")).unwrap());
        assert!(!provider.has_permission("user123", &Permission::new("profile", "other_user", "read")).unwrap());
        
        // Remove role
        provider.remove_role("user123", "user").unwrap();
        
        // Check user roles again
        let roles = provider.get_user_roles("user123").unwrap();
        assert_eq!(roles.len(), 0);
    }
    
    #[test]
    fn test_policy_enforcer() {
        let provider = Arc::new(LocalAuthzProvider::new());
        let enforcer = PolicyEnforcer::new(provider.clone());
        
        // Assign admin role to user
        provider.assign_role("admin123", "admin").unwrap();
        
        // Enforce policy
        assert!(enforcer.enforce("admin123", &Permission::new("device", "123", "read")).is_ok());
        assert!(enforcer.enforce("admin123", &Permission::new("profile", "123", "write")).is_ok());
        
        // Enforce policy for user without permission
        assert!(enforcer.enforce("user123", &Permission::new("device", "123", "read")).is_err());
    }
}

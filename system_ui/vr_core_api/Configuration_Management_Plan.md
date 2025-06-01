# Configuration Management Expansion Plan

This document outlines the detailed implementation plan for expanding the Configuration Management module in the VR Core API layer. The configuration management system will provide robust handling of settings, user profiles, and configuration data for the VR headset.

## 1. Overall Architecture

### 1.1 Design Principles

- **Schema-based validation**: Ensure configuration data conforms to defined schemas
- **Versioning support**: Handle configuration format changes gracefully
- **Multi-user profiles**: Support multiple user profiles with isolated settings
- **Secure storage**: Protect sensitive configuration data
- **Change notifications**: Notify components of configuration changes
- **Backup and restore**: Reliable backup and recovery of configuration data
- **Performance**: Efficient access to frequently used settings

### 1.2 Module Structure

```
config/
├── mod.rs                 # Main module and ConfigManager
├── schema.rs              # Schema definition and validation
├── versioning.rs          # Version management and migration
├── storage.rs             # Configuration storage and I/O
├── profile.rs             # User profile management
├── backup.rs              # Backup and restore functionality
├── notification.rs        # Change notification system
└── tests/                 # Test modules
    ├── test_schema.rs     # Schema validation tests
    ├── test_versioning.rs # Version migration tests
    └── ...
```

## 2. Schema Validation

### 2.1 Schema Definition

```rust
/// Schema for validating configuration values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigSchema {
    /// Schema name
    pub name: String,
    /// Schema version
    pub version: String,
    /// Property definitions
    pub properties: HashMap<String, PropertySchema>,
    /// Required properties
    pub required: Vec<String>,
}

/// Schema for a single property
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertySchema {
    /// Property type
    pub property_type: PropertyType,
    /// Property description
    pub description: String,
    /// Default value
    pub default: Option<ConfigValue>,
    /// Minimum value (for numeric types)
    pub minimum: Option<ConfigValue>,
    /// Maximum value (for numeric types)
    pub maximum: Option<ConfigValue>,
    /// Enum values (for enum types)
    pub enum_values: Option<Vec<ConfigValue>>,
    /// Pattern (for string types)
    pub pattern: Option<String>,
    /// Items schema (for array types)
    pub items: Option<Box<PropertySchema>>,
    /// Properties schema (for object types)
    pub properties: Option<HashMap<String, PropertySchema>>,
}
```

### 2.2 Schema Validation

```rust
/// Schema validator for configuration data
pub struct SchemaValidator {
    schemas: HashMap<String, ConfigSchema>,
}

impl SchemaValidator {
    /// Create a new SchemaValidator
    pub fn new() -> Self;
    
    /// Load schemas from file
    pub fn load_schemas(&mut self, path: &Path) -> Result<()>;
    
    /// Add schema
    pub fn add_schema(&mut self, schema: ConfigSchema) -> Result<()>;
    
    /// Get schema by name
    pub fn get_schema(&self, name: &str) -> Option<&ConfigSchema>;
    
    /// Validate configuration against schema
    pub fn validate(&self, schema_name: &str, config: &HashMap<String, ConfigValue>) -> Result<()>;
    
    /// Generate default configuration from schema
    pub fn generate_default(&self, schema_name: &str) -> Result<HashMap<String, ConfigValue>>;
}
```

### 2.3 Schema Registry

```rust
/// Registry for configuration schemas
pub struct SchemaRegistry {
    validators: HashMap<ConfigCategory, SchemaValidator>,
}

impl SchemaRegistry {
    /// Create a new SchemaRegistry
    pub fn new() -> Self;
    
    /// Register schema for category
    pub fn register_schema(&mut self, category: ConfigCategory, schema: ConfigSchema) -> Result<()>;
    
    /// Get validator for category
    pub fn get_validator(&self, category: ConfigCategory) -> Option<&SchemaValidator>;
    
    /// Validate category configuration
    pub fn validate_category(&self, category: ConfigCategory, config: &HashMap<String, ConfigValue>) -> Result<()>;
    
    /// Generate default configuration for category
    pub fn generate_default(&self, category: ConfigCategory) -> Result<HashMap<String, ConfigValue>>;
}
```

## 3. Configuration Versioning

### 3.1 Version Management

```rust
/// Configuration version information
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ConfigVersion {
    /// Major version (incompatible changes)
    pub major: u32,
    /// Minor version (backwards-compatible additions)
    pub minor: u32,
    /// Patch version (backwards-compatible fixes)
    pub patch: u32,
}

impl ConfigVersion {
    /// Create a new ConfigVersion
    pub fn new(major: u32, minor: u32, patch: u32) -> Self;
    
    /// Parse version string
    pub fn parse(version_str: &str) -> Result<Self>;
    
    /// Convert to string
    pub fn to_string(&self) -> String;
    
    /// Is compatible with other version
    pub fn is_compatible_with(&self, other: &ConfigVersion) -> bool;
}
```

### 3.2 Migration System

```rust
/// Migration function type
pub type MigrationFn = fn(&mut HashMap<String, ConfigValue>) -> Result<()>;

/// Migration definition
pub struct Migration {
    /// Source version
    pub from_version: ConfigVersion,
    /// Target version
    pub to_version: ConfigVersion,
    /// Migration function
    pub migrate: MigrationFn,
}

/// Migration manager for configuration versions
pub struct MigrationManager {
    migrations: HashMap<ConfigCategory, Vec<Migration>>,
}

impl MigrationManager {
    /// Create a new MigrationManager
    pub fn new() -> Self;
    
    /// Register migration
    pub fn register_migration(&mut self, category: ConfigCategory, migration: Migration) -> Result<()>;
    
    /// Get migrations for category
    pub fn get_migrations(&self, category: ConfigCategory) -> Option<&Vec<Migration>>;
    
    /// Migrate configuration from one version to another
    pub fn migrate(&self, category: ConfigCategory, config: &mut HashMap<String, ConfigValue>, 
                  from_version: &ConfigVersion, to_version: &ConfigVersion) -> Result<()>;
    
    /// Find migration path
    pub fn find_migration_path(&self, category: ConfigCategory, 
                              from_version: &ConfigVersion, to_version: &ConfigVersion) -> Result<Vec<&Migration>>;
}
```

## 4. User Profiles

### 4.1 Profile Management

```rust
/// User profile information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
    /// Profile ID
    pub id: String,
    /// Profile name
    pub name: String,
    /// Profile creation timestamp
    pub created_at: u64,
    /// Profile last modified timestamp
    pub modified_at: u64,
    /// Profile avatar (optional)
    pub avatar: Option<String>,
    /// Profile type
    pub profile_type: ProfileType,
}

/// Profile type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileType {
    /// Administrator profile
    Admin,
    /// Standard user profile
    User,
    /// Guest profile
    Guest,
    /// Developer profile
    Developer,
}

/// Profile manager for handling user profiles
pub struct ProfileManager {
    profiles: HashMap<String, UserProfile>,
    active_profile: Option<String>,
    profile_configs: HashMap<String, HashMap<ConfigCategory, HashMap<String, ConfigValue>>>,
    config_path: PathBuf,
}

impl ProfileManager {
    /// Create a new ProfileManager
    pub fn new(config_path: &Path) -> Result<Self>;
    
    /// Load profiles
    pub fn load_profiles(&mut self) -> Result<()>;
    
    /// Save profiles
    pub fn save_profiles(&self) -> Result<()>;
    
    /// Get all profiles
    pub fn get_profiles(&self) -> Vec<&UserProfile>;
    
    /// Get profile by ID
    pub fn get_profile(&self, id: &str) -> Option<&UserProfile>;
    
    /// Create new profile
    pub fn create_profile(&mut self, name: &str, profile_type: ProfileType) -> Result<UserProfile>;
    
    /// Update profile
    pub fn update_profile(&mut self, id: &str, name: Option<&str>, avatar: Option<&str>) -> Result<()>;
    
    /// Delete profile
    pub fn delete_profile(&mut self, id: &str) -> Result<()>;
    
    /// Set active profile
    pub fn set_active_profile(&mut self, id: &str) -> Result<()>;
    
    /// Get active profile
    pub fn active_profile(&self) -> Option<&UserProfile>;
    
    /// Get profile configuration
    pub fn get_profile_config(&self, profile_id: &str, category: ConfigCategory) -> Option<&HashMap<String, ConfigValue>>;
    
    /// Set profile configuration
    pub fn set_profile_config(&mut self, profile_id: &str, category: ConfigCategory, 
                             config: HashMap<String, ConfigValue>) -> Result<()>;
    
    /// Get configuration value for profile
    pub fn get_profile_value(&self, profile_id: &str, category: ConfigCategory, key: &str) -> Result<ConfigValue>;
    
    /// Set configuration value for profile
    pub fn set_profile_value(&mut self, profile_id: &str, category: ConfigCategory, 
                            key: &str, value: ConfigValue) -> Result<()>;
}
```

### 4.2 Profile Storage

```rust
/// Profile storage handler
pub struct ProfileStorage {
    base_path: PathBuf,
}

impl ProfileStorage {
    /// Create a new ProfileStorage
    pub fn new(base_path: &Path) -> Self;
    
    /// Load profile metadata
    pub fn load_metadata(&self) -> Result<HashMap<String, UserProfile>>;
    
    /// Save profile metadata
    pub fn save_metadata(&self, profiles: &HashMap<String, UserProfile>) -> Result<()>;
    
    /// Load profile configuration
    pub fn load_profile_config(&self, profile_id: &str) -> Result<HashMap<ConfigCategory, HashMap<String, ConfigValue>>>;
    
    /// Save profile configuration
    pub fn save_profile_config(&self, profile_id: &str, 
                              config: &HashMap<ConfigCategory, HashMap<String, ConfigValue>>) -> Result<()>;
    
    /// Delete profile
    pub fn delete_profile(&self, profile_id: &str) -> Result<()>;
    
    /// Get profile path
    pub fn get_profile_path(&self, profile_id: &str) -> PathBuf;
}
```

## 5. Configuration Storage

### 5.1 Storage Backend

```rust
/// Configuration storage backend trait
pub trait ConfigStorage {
    /// Load configuration
    fn load(&self) -> Result<HashMap<ConfigCategory, HashMap<String, ConfigValue>>>;
    
    /// Save configuration
    fn save(&self, config: &HashMap<ConfigCategory, HashMap<String, ConfigValue>>) -> Result<()>;
    
    /// Load category
    fn load_category(&self, category: ConfigCategory) -> Result<HashMap<String, ConfigValue>>;
    
    /// Save category
    fn save_category(&self, category: ConfigCategory, config: &HashMap<String, ConfigValue>) -> Result<()>;
}

/// File-based configuration storage
pub struct FileConfigStorage {
    config_path: PathBuf,
}

impl FileConfigStorage {
    /// Create a new FileConfigStorage
    pub fn new(config_path: &Path) -> Self;
}

impl ConfigStorage for FileConfigStorage {
    // Implementation of ConfigStorage trait methods
}
```

### 5.2 Secure Storage

```rust
/// Secure configuration storage for sensitive data
pub struct SecureConfigStorage {
    storage: Box<dyn ConfigStorage>,
    encryption_key: Vec<u8>,
}

impl SecureConfigStorage {
    /// Create a new SecureConfigStorage
    pub fn new(storage: Box<dyn ConfigStorage>, encryption_key: &[u8]) -> Self;
}

impl ConfigStorage for SecureConfigStorage {
    // Implementation of ConfigStorage trait methods with encryption/decryption
}
```

## 6. Configuration Change Notifications

### 6.1 Change Event System

```rust
/// Configuration change event
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigChangeEvent {
    /// Category that changed
    pub category: ConfigCategory,
    /// Key that changed (None if entire category changed)
    pub key: Option<String>,
    /// Old value (None if key was added)
    pub old_value: Option<ConfigValue>,
    /// New value (None if key was removed)
    pub new_value: Option<ConfigValue>,
    /// Timestamp of change
    pub timestamp: u64,
    /// Profile ID (None if system config)
    pub profile_id: Option<String>,
}

/// Configuration change listener trait
pub trait ConfigChangeListener: Send + Sync {
    /// Handle configuration change event
    fn on_config_change(&self, event: &ConfigChangeEvent);
    
    /// Get listener ID
    fn id(&self) -> &str;
}

/// Configuration change notifier
pub struct ConfigChangeNotifier {
    listeners: HashMap<String, Box<dyn ConfigChangeListener>>,
    category_listeners: HashMap<ConfigCategory, HashSet<String>>,
    key_listeners: HashMap<(ConfigCategory, String), HashSet<String>>,
}

impl ConfigChangeNotifier {
    /// Create a new ConfigChangeNotifier
    pub fn new() -> Self;
    
    /// Register listener
    pub fn register_listener(&mut self, listener: Box<dyn ConfigChangeListener>) -> Result<()>;
    
    /// Unregister listener
    pub fn unregister_listener(&mut self, id: &str) -> Result<()>;
    
    /// Register listener for category
    pub fn register_for_category(&mut self, listener_id: &str, category: ConfigCategory) -> Result<()>;
    
    /// Register listener for key
    pub fn register_for_key(&mut self, listener_id: &str, category: ConfigCategory, key: &str) -> Result<()>;
    
    /// Notify change
    pub fn notify_change(&self, event: ConfigChangeEvent) -> Result<()>;
}
```

## 7. Backup and Restore

### 7.1 Backup System

```rust
/// Backup metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Backup ID
    pub id: String,
    /// Backup timestamp
    pub timestamp: u64,
    /// Backup description
    pub description: String,
    /// Configuration version
    pub config_version: ConfigVersion,
    /// Included profiles
    pub profiles: Vec<String>,
    /// Included categories
    pub categories: Vec<ConfigCategory>,
    /// Checksum
    pub checksum: String,
}

/// Backup manager
pub struct BackupManager {
    backup_dir: PathBuf,
    backups: HashMap<String, BackupMetadata>,
}

impl BackupManager {
    /// Create a new BackupManager
    pub fn new(backup_dir: &Path) -> Result<Self>;
    
    /// Load backup metadata
    pub fn load_metadata(&mut self) -> Result<()>;
    
    /// Get all backups
    pub fn get_backups(&self) -> Vec<&BackupMetadata>;
    
    /// Get backup by ID
    pub fn get_backup(&self, id: &str) -> Option<&BackupMetadata>;
    
    /// Create backup
    pub fn create_backup(&mut self, description: &str, profiles: Option<Vec<String>>, 
                        categories: Option<Vec<ConfigCategory>>) -> Result<BackupMetadata>;
    
    /// Restore from backup
    pub fn restore_from_backup(&self, id: &str, profiles: Option<Vec<String>>, 
                              categories: Option<Vec<ConfigCategory>>) -> Result<()>;
    
    /// Delete backup
    pub fn delete_backup(&mut self, id: &str) -> Result<()>;
    
    /// Export backup
    pub fn export_backup(&self, id: &str, destination: &Path) -> Result<()>;
    
    /// Import backup
    pub fn import_backup(&mut self, source: &Path) -> Result<BackupMetadata>;
}
```

### 7.2 Backup Storage

```rust
/// Backup storage handler
pub struct BackupStorage {
    base_path: PathBuf,
}

impl BackupStorage {
    /// Create a new BackupStorage
    pub fn new(base_path: &Path) -> Self;
    
    /// Load backup metadata
    pub fn load_metadata(&self) -> Result<HashMap<String, BackupMetadata>>;
    
    /// Save backup metadata
    pub fn save_metadata(&self, backups: &HashMap<String, BackupMetadata>) -> Result<()>;
    
    /// Create backup file
    pub fn create_backup_file(&self, id: &str, config: &HashMap<ConfigCategory, HashMap<String, ConfigValue>>, 
                             profiles: &HashMap<String, HashMap<ConfigCategory, HashMap<String, ConfigValue>>>) -> Result<()>;
    
    /// Load backup file
    pub fn load_backup_file(&self, id: &str) -> Result<(HashMap<ConfigCategory, HashMap<String, ConfigValue>>, 
                                                      HashMap<String, HashMap<ConfigCategory, HashMap<String, ConfigValue>>>)>;
    
    /// Delete backup file
    pub fn delete_backup_file(&self, id: &str) -> Result<()>;
    
    /// Get backup path
    pub fn get_backup_path(&self, id: &str) -> PathBuf;
}
```

## 8. Enhanced ConfigManager

Update the existing `ConfigManager` to integrate all the new functionality:

```rust
pub struct ConfigManager {
    config_path: PathBuf,
    config_data: Arc<RwLock<HashMap<ConfigCategory, HashMap<String, ConfigValue>>>>,
    schema_registry: SchemaRegistry,
    migration_manager: MigrationManager,
    profile_manager: ProfileManager,
    change_notifier: ConfigChangeNotifier,
    backup_manager: BackupManager,
    storage: Box<dyn ConfigStorage>,
    config_version: ConfigVersion,
    dirty: bool,
}

impl ConfigManager {
    // Existing methods...
    
    /// Get schema registry
    pub fn schema_registry(&self) -> &SchemaRegistry {
        &self.schema_registry
    }
    
    /// Get migration manager
    pub fn migration_manager(&self) -> &MigrationManager {
        &self.migration_manager
    }
    
    /// Get profile manager
    pub fn profile_manager(&self) -> &ProfileManager {
        &self.profile_manager
    }
    
    /// Get mutable profile manager
    pub fn profile_manager_mut(&mut self) -> &mut ProfileManager {
        &mut self.profile_manager
    }
    
    /// Get change notifier
    pub fn change_notifier(&self) -> &ConfigChangeNotifier {
        &self.change_notifier
    }
    
    /// Get backup manager
    pub fn backup_manager(&self) -> &BackupManager {
        &self.backup_manager
    }
    
    /// Get mutable backup manager
    pub fn backup_manager_mut(&mut self) -> &mut BackupManager {
        &mut self.backup_manager
    }
    
    /// Get configuration version
    pub fn version(&self) -> &ConfigVersion {
        &self.config_version
    }
    
    /// Register change listener
    pub fn register_listener(&mut self, listener: Box<dyn ConfigChangeListener>) -> Result<()> {
        self.change_notifier.register_listener(listener)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        let config_data = self.config_data.read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on config data"))?;
            
        for (category, values) in config_data.iter() {
            self.schema_registry.validate_category(*category, values)?;
        }
        
        Ok(())
    }
    
    /// Get configuration for active profile
    pub fn get_profile_config(&self, category: ConfigCategory) -> Result<HashMap<String, ConfigValue>> {
        if let Some(profile) = self.profile_manager.active_profile() {
            if let Some(config) = self.profile_manager.get_profile_config(&profile.id, category) {
                return Ok(config.clone());
            }
        }
        
        // Fall back to system config
        let config_data = self.config_data.read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on config data"))?;
            
        if let Some(category_data) = config_data.get(&category) {
            Ok(category_data.clone())
        } else {
            Ok(HashMap::new())
        }
    }
    
    /// Get value for active profile
    pub fn get_profile_value(&self, category: ConfigCategory, key: &str) -> Result<ConfigValue> {
        if let Some(profile) = self.profile_manager.active_profile() {
            match self.profile_manager.get_profile_value(&profile.id, category, key) {
                Ok(value) => return Ok(value),
                Err(e) => {
                    if !matches!(e.downcast_ref::<ConfigError>(), Some(ConfigError::KeyNotFound(_))) {
                        return Err(e);
                    }
                    // Key not found in profile, fall back to system config
                }
            }
        }
        
        // Fall back to system config
        self.get(category, key)
    }
    
    /// Set value for active profile
    pub fn set_profile_value(&mut self, category: ConfigCategory, key: &str, value: ConfigValue) -> Result<()> {
        if let Some(profile) = self.profile_manager.active_profile() {
            let profile_id = profile.id.clone();
            self.profile_manager.set_profile_value(&profile_id, category, key, value.clone())?;
            
            // Notify change
            let event = ConfigChangeEvent {
                category,
                key: Some(key.to_string()),
                old_value: None, // We don't track the old value here
                new_value: Some(value),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                profile_id: Some(profile_id),
            };
            
            self.change_notifier.notify_change(event)?;
            
            Ok(())
        } else {
            // No active profile, set system config
            self.set(category, key, value)
        }
    }
    
    /// Create backup
    pub fn create_backup(&mut self, description: &str) -> Result<BackupMetadata> {
        self.backup_manager.create_backup(description, None, None)
    }
    
    /// Restore from backup
    pub fn restore_from_backup(&mut self, id: &str) -> Result<()> {
        self.backup_manager.restore_from_backup(id, None, None)
    }
}
```

## 9. Implementation Strategy

### 9.1 Phase 1: Schema Validation

1. Define schema structures and validation logic
2. Implement SchemaValidator and SchemaRegistry
3. Create default schemas for all configuration categories
4. Integrate schema validation with ConfigManager

### 9.2 Phase 2: Versioning and Migration

1. Implement ConfigVersion and version comparison
2. Create MigrationManager for version migrations
3. Define migration paths for configuration changes
4. Integrate versioning with ConfigManager

### 9.3 Phase 3: User Profiles

1. Implement UserProfile and ProfileType
2. Create ProfileManager for profile management
3. Implement profile-specific configuration storage
4. Integrate profile management with ConfigManager

### 9.4 Phase 4: Change Notifications

1. Define ConfigChangeEvent and listener interface
2. Implement ConfigChangeNotifier
3. Add change notification to ConfigManager operations
4. Create standard listeners for common components

### 9.5 Phase 5: Backup and Restore

1. Implement BackupMetadata and backup file format
2. Create BackupManager for backup operations
3. Implement backup storage and file handling
4. Integrate backup and restore with ConfigManager

## 10. Testing Plan

### 10.1 Unit Tests

- Test schema validation with valid and invalid configurations
- Test version migration between different versions
- Test profile management and profile-specific configurations
- Test change notification system
- Test backup and restore functionality

### 10.2 Integration Tests

- Test ConfigManager with all new components
- Test interaction between profiles and system configuration
- Test configuration persistence across restarts
- Test migration of real configuration data

### 10.3 Performance Tests

- Test configuration loading performance
- Test change notification performance with many listeners
- Test backup and restore performance with large configurations

## 11. Documentation Plan

### 11.1 API Documentation

- Document all public traits, structs, and methods
- Include examples for common configuration operations
- Document schema definition format
- Document migration system

### 11.2 User Guide

- Create guide for defining configuration schemas
- Document profile management
- Provide backup and restore instructions
- Include troubleshooting information

## 12. Timeline and Milestones

1. **Week 1**: Implement schema validation and schema registry
2. **Week 2**: Implement versioning and migration system
3. **Week 3**: Implement user profiles and profile-specific configuration
4. **Week 4**: Implement change notification system
5. **Week 5**: Implement backup and restore functionality
6. **Week 6**: Integration, testing, and documentation

## 13. Dependencies and Requirements

- Rust crates:
  - `serde` and `serde_derive` for serialization
  - `toml` for TOML parsing and generation
  - `thiserror` for error handling
  - `log` for logging
  - `regex` for pattern validation
  - `ring` or `sodiumoxide` for secure storage

This implementation plan provides a comprehensive approach to expanding the configuration management system in the VR Core API layer, covering schema validation, versioning, user profiles, change notifications, and backup/restore functionality while ensuring proper integration with the existing codebase.

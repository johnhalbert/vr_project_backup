//! Optimization module for the VR headset system.
//!
//! This module provides performance optimization capabilities for various
//! system components, including CPU, GPU, memory, storage, network, and power.
//! It is designed to maximize performance and efficiency on the Orange Pi CM5 platform.

pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod storage;
pub mod network;
pub mod power;

use std::sync::{Arc, Mutex};
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};

/// Optimization manager that coordinates all optimization strategies.
#[derive(Debug)]
pub struct OptimizationManager {
    /// CPU optimization manager
    cpu_manager: Arc<Mutex<cpu::CpuOptimizationManager>>,
    
    /// GPU optimization manager
    gpu_manager: Option<Arc<Mutex<gpu::GpuOptimizationManager>>>,
    
    /// Memory optimization manager
    memory_manager: Option<Arc<Mutex<memory::MemoryOptimizationManager>>>,
    
    /// Storage optimization manager
    storage_manager: Option<Arc<Mutex<storage::StorageOptimizationManager>>>,
    
    /// Network optimization manager
    network_manager: Option<Arc<Mutex<network::NetworkOptimizationManager>>>,
    
    /// Power optimization manager
    power_manager: Option<Arc<Mutex<power::PowerOptimizationManager>>>,
    
    /// Global optimization settings
    settings: OptimizationSettings,
}

/// Global optimization settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSettings {
    /// Whether optimization is enabled
    pub enabled: bool,
    
    /// Optimization mode
    pub mode: OptimizationMode,
    
    /// Optimization priority
    pub priority: OptimizationPriority,
    
    /// Whether to apply aggressive optimizations
    pub aggressive: bool,
    
    /// Whether to automatically adapt optimizations based on system state
    pub adaptive: bool,
    
    /// Minimum interval between optimization adjustments (in milliseconds)
    pub adjustment_interval_ms: u64,
}

/// Optimization mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationMode {
    /// Balanced mode that tries to optimize all aspects
    Balanced,
    
    /// Performance mode that prioritizes raw performance
    Performance,
    
    /// Efficiency mode that prioritizes power efficiency
    Efficiency,
    
    /// Latency mode that prioritizes low latency
    Latency,
    
    /// Thermal mode that prioritizes thermal management
    Thermal,
    
    /// Custom mode with user-defined settings
    Custom,
}

/// Optimization priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationPriority {
    /// Prioritize CPU optimization
    Cpu,
    
    /// Prioritize GPU optimization
    Gpu,
    
    /// Prioritize memory optimization
    Memory,
    
    /// Prioritize storage optimization
    Storage,
    
    /// Prioritize network optimization
    Network,
    
    /// Prioritize power optimization
    Power,
}

impl Default for OptimizationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: OptimizationMode::Balanced,
            priority: OptimizationPriority::Cpu,
            aggressive: false,
            adaptive: true,
            adjustment_interval_ms: 1000,
        }
    }
}

impl OptimizationManager {
    /// Create a new optimization manager with default settings.
    pub fn new() -> Result<Self> {
        Self::with_settings(OptimizationSettings::default())
    }
    
    /// Create a new optimization manager with the specified settings.
    pub fn with_settings(settings: OptimizationSettings) -> Result<Self> {
        let cpu_manager = Arc::new(Mutex::new(cpu::CpuOptimizationManager::new()?));
        
        Ok(Self {
            cpu_manager,
            gpu_manager: None,
            memory_manager: None,
            storage_manager: None,
            network_manager: None,
            power_manager: None,
            settings,
        })
    }
    
    /// Initialize all optimization managers.
    pub fn initialize(&mut self) -> Result<()> {
        // Initialize CPU optimization
        self.cpu_manager.lock().unwrap().initialize()
            .context("Failed to initialize CPU optimization")?;
        
        // Initialize GPU optimization if available
        if let Some(gpu_manager) = &self.gpu_manager {
            gpu_manager.lock().unwrap().initialize()
                .context("Failed to initialize GPU optimization")?;
        }
        
        // Initialize memory optimization if available
        if let Some(memory_manager) = &self.memory_manager {
            memory_manager.lock().unwrap().initialize()
                .context("Failed to initialize memory optimization")?;
        }
        
        // Initialize storage optimization if available
        if let Some(storage_manager) = &self.storage_manager {
            storage_manager.lock().unwrap().initialize()
                .context("Failed to initialize storage optimization")?;
        }
        
        // Initialize network optimization if available
        if let Some(network_manager) = &self.network_manager {
            network_manager.lock().unwrap().initialize()
                .context("Failed to initialize network optimization")?;
        }
        
        // Initialize power optimization if available
        if let Some(power_manager) = &self.power_manager {
            power_manager.lock().unwrap().initialize()
                .context("Failed to initialize power optimization")?;
        }
        
        Ok(())
    }
    
    /// Apply all optimizations.
    pub fn apply_optimizations(&self) -> Result<()> {
        if !self.settings.enabled {
            return Ok(());
        }
        
        // Apply CPU optimization
        self.cpu_manager.lock().unwrap().apply_optimizations(&self.settings)
            .context("Failed to apply CPU optimization")?;
        
        // Apply GPU optimization if available
        if let Some(gpu_manager) = &self.gpu_manager {
            gpu_manager.lock().unwrap().apply_optimizations(&self.settings)
                .context("Failed to apply GPU optimization")?;
        }
        
        // Apply memory optimization if available
        if let Some(memory_manager) = &self.memory_manager {
            memory_manager.lock().unwrap().apply_optimizations(&self.settings)
                .context("Failed to apply memory optimization")?;
        }
        
        // Apply storage optimization if available
        if let Some(storage_manager) = &self.storage_manager {
            storage_manager.lock().unwrap().apply_optimizations(&self.settings)
                .context("Failed to apply storage optimization")?;
        }
        
        // Apply network optimization if available
        if let Some(network_manager) = &self.network_manager {
            network_manager.lock().unwrap().apply_optimizations(&self.settings)
                .context("Failed to apply network optimization")?;
        }
        
        // Apply power optimization if available
        if let Some(power_manager) = &self.power_manager {
            power_manager.lock().unwrap().apply_optimizations(&self.settings)
                .context("Failed to apply power optimization")?;
        }
        
        Ok(())
    }
    
    /// Reset all optimizations to default values.
    pub fn reset_optimizations(&self) -> Result<()> {
        // Reset CPU optimization
        self.cpu_manager.lock().unwrap().reset_optimizations()
            .context("Failed to reset CPU optimization")?;
        
        // Reset GPU optimization if available
        if let Some(gpu_manager) = &self.gpu_manager {
            gpu_manager.lock().unwrap().reset_optimizations()
                .context("Failed to reset GPU optimization")?;
        }
        
        // Reset memory optimization if available
        if let Some(memory_manager) = &self.memory_manager {
            memory_manager.lock().unwrap().reset_optimizations()
                .context("Failed to reset memory optimization")?;
        }
        
        // Reset storage optimization if available
        if let Some(storage_manager) = &self.storage_manager {
            storage_manager.lock().unwrap().reset_optimizations()
                .context("Failed to reset storage optimization")?;
        }
        
        // Reset network optimization if available
        if let Some(network_manager) = &self.network_manager {
            network_manager.lock().unwrap().reset_optimizations()
                .context("Failed to reset network optimization")?;
        }
        
        // Reset power optimization if available
        if let Some(power_manager) = &self.power_manager {
            power_manager.lock().unwrap().reset_optimizations()
                .context("Failed to reset power optimization")?;
        }
        
        Ok(())
    }
    
    /// Update optimization settings.
    pub fn update_settings(&mut self, settings: OptimizationSettings) -> Result<()> {
        self.settings = settings;
        self.apply_optimizations()?;
        Ok(())
    }
    
    /// Get current optimization settings.
    pub fn get_settings(&self) -> OptimizationSettings {
        self.settings.clone()
    }
    
    /// Get CPU optimization manager.
    pub fn get_cpu_manager(&self) -> Arc<Mutex<cpu::CpuOptimizationManager>> {
        self.cpu_manager.clone()
    }
    
    /// Get GPU optimization manager.
    pub fn get_gpu_manager(&self) -> Option<Arc<Mutex<gpu::GpuOptimizationManager>>> {
        self.gpu_manager.clone()
    }
    
    /// Get memory optimization manager.
    pub fn get_memory_manager(&self) -> Option<Arc<Mutex<memory::MemoryOptimizationManager>>> {
        self.memory_manager.clone()
    }
    
    /// Get storage optimization manager.
    pub fn get_storage_manager(&self) -> Option<Arc<Mutex<storage::StorageOptimizationManager>>> {
        self.storage_manager.clone()
    }
    
    /// Get network optimization manager.
    pub fn get_network_manager(&self) -> Option<Arc<Mutex<network::NetworkOptimizationManager>>> {
        self.network_manager.clone()
    }
    
    /// Get power optimization manager.
    pub fn get_power_manager(&self) -> Option<Arc<Mutex<power::PowerOptimizationManager>>> {
        self.power_manager.clone()
    }
    
    /// Set GPU optimization manager.
    pub fn set_gpu_manager(&mut self, manager: Arc<Mutex<gpu::GpuOptimizationManager>>) {
        self.gpu_manager = Some(manager);
    }
    
    /// Set memory optimization manager.
    pub fn set_memory_manager(&mut self, manager: Arc<Mutex<memory::MemoryOptimizationManager>>) {
        self.memory_manager = Some(manager);
    }
    
    /// Set storage optimization manager.
    pub fn set_storage_manager(&mut self, manager: Arc<Mutex<storage::StorageOptimizationManager>>) {
        self.storage_manager = Some(manager);
    }
    
    /// Set network optimization manager.
    pub fn set_network_manager(&mut self, manager: Arc<Mutex<network::NetworkOptimizationManager>>) {
        self.network_manager = Some(manager);
    }
    
    /// Set power optimization manager.
    pub fn set_power_manager(&mut self, manager: Arc<Mutex<power::PowerOptimizationManager>>) {
        self.power_manager = Some(manager);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimization_settings_default() {
        let settings = OptimizationSettings::default();
        assert!(settings.enabled);
        assert_eq!(settings.mode, OptimizationMode::Balanced);
        assert_eq!(settings.priority, OptimizationPriority::Cpu);
        assert!(!settings.aggressive);
        assert!(settings.adaptive);
        assert_eq!(settings.adjustment_interval_ms, 1000);
    }
    
    #[test]
    fn test_optimization_manager_creation() {
        let manager = OptimizationManager::new();
        assert!(manager.is_ok());
    }
}

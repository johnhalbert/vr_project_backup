//! Dependency resolution module for the VR headset update system.
//!
//! This module provides functionality for managing update package dependencies,
//! ensuring that updates are installed in the correct order and that all
//! required dependencies are satisfied.

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::collections::{HashMap, HashSet, VecDeque};
use anyhow::{Result, Context, anyhow, bail};
use serde::{Serialize, Deserialize};
use semver::{Version, VersionReq};
use log::{info, warn, error, debug};

use super::package::UpdatePackageMetadata;

/// Dependency information for an update package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    /// List of package dependencies.
    pub dependencies: Vec<PackageDependency>,
    
    /// List of component dependencies.
    pub component_dependencies: Vec<ComponentDependency>,
    
    /// List of system requirements.
    pub system_requirements: SystemRequirements,
    
    /// Whether this package conflicts with any other packages.
    pub conflicts: Vec<PackageConflict>,
}

/// A dependency on another package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDependency {
    /// Name of the package.
    pub package_name: String,
    
    /// Version requirement for the package.
    pub version_req: VersionReq,
    
    /// Whether this dependency is optional.
    pub optional: bool,
}

/// A dependency on a specific component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDependency {
    /// Name of the component.
    pub component_name: String,
    
    /// Version requirement for the component.
    pub version_req: VersionReq,
    
    /// Whether this dependency is optional.
    pub optional: bool,
}

/// System requirements for an update package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRequirements {
    /// Minimum CPU requirements.
    pub min_cpu: Option<String>,
    
    /// Minimum RAM requirements in MB.
    pub min_ram_mb: Option<u32>,
    
    /// Minimum storage requirements in MB.
    pub min_storage_mb: Option<u32>,
    
    /// Required hardware features.
    pub required_features: Vec<String>,
    
    /// Required kernel version.
    pub kernel_version: Option<VersionReq>,
}

/// A conflict with another package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConflict {
    /// Name of the conflicting package.
    pub package_name: String,
    
    /// Version range that conflicts.
    pub version_range: VersionReq,
    
    /// Reason for the conflict.
    pub reason: String,
}

/// Information about an installed package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackageInfo {
    /// Name of the package.
    pub package_name: String,
    
    /// Version of the package.
    pub version: Version,
    
    /// Components provided by this package.
    pub components: Vec<InstalledComponent>,
    
    /// Installation date.
    pub installation_date: chrono::DateTime<chrono::Utc>,
}

/// Information about an installed component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledComponent {
    /// Name of the component.
    pub component_name: String,
    
    /// Version of the component.
    pub version: Version,
}

/// Result of dependency resolution.
#[derive(Debug, Clone)]
pub struct DependencyResolutionResult {
    /// Whether all dependencies are satisfied.
    pub satisfied: bool,
    
    /// List of missing dependencies.
    pub missing_dependencies: Vec<PackageDependency>,
    
    /// List of missing component dependencies.
    pub missing_component_dependencies: Vec<ComponentDependency>,
    
    /// List of unsatisfied system requirements.
    pub unsatisfied_system_requirements: Vec<String>,
    
    /// List of package conflicts.
    pub conflicts: Vec<PackageConflict>,
    
    /// Installation order for packages.
    pub installation_order: Vec<String>,
}

/// Check if dependencies are satisfied for an update package.
///
/// # Arguments
///
/// * `metadata` - Metadata for the update package
/// * `installed_packages` - List of installed packages
/// * `system_info` - Information about the system
///
/// # Returns
///
/// Result of dependency resolution.
pub fn check_dependencies(
    metadata: &UpdatePackageMetadata,
    installed_packages: &[InstalledPackageInfo],
    system_info: &SystemInfo,
) -> DependencyResolutionResult {
    debug!("Checking dependencies for update package: {}", metadata.version);
    
    let mut result = DependencyResolutionResult {
        satisfied: true,
        missing_dependencies: Vec::new(),
        missing_component_dependencies: Vec::new(),
        unsatisfied_system_requirements: Vec::new(),
        conflicts: Vec::new(),
        installation_order: Vec::new(),
    };
    
    // Check package dependencies
    if let Some(deps) = &metadata.dependencies {
        for dep in &deps.dependencies {
            let mut satisfied = false;
            
            for pkg in installed_packages {
                if pkg.package_name == dep.package_name && dep.version_req.matches(&pkg.version) {
                    satisfied = true;
                    break;
                }
            }
            
            if !satisfied && !dep.optional {
                result.missing_dependencies.push(dep.clone());
                result.satisfied = false;
            }
        }
        
        // Check component dependencies
        for dep in &deps.component_dependencies {
            let mut satisfied = false;
            
            for pkg in installed_packages {
                for comp in &pkg.components {
                    if comp.component_name == dep.component_name && dep.version_req.matches(&comp.version) {
                        satisfied = true;
                        break;
                    }
                }
                
                if satisfied {
                    break;
                }
            }
            
            if !satisfied && !dep.optional {
                result.missing_component_dependencies.push(dep.clone());
                result.satisfied = false;
            }
        }
        
        // Check system requirements
        if let Some(req) = &deps.system_requirements {
            // Check CPU requirements
            if let Some(min_cpu) = &req.min_cpu {
                if !system_info.cpu_model.contains(min_cpu) {
                    result.unsatisfied_system_requirements.push(
                        format!("CPU requirement not met: requires {}, have {}", 
                                min_cpu, system_info.cpu_model));
                    result.satisfied = false;
                }
            }
            
            // Check RAM requirements
            if let Some(min_ram) = req.min_ram_mb {
                if system_info.ram_mb < min_ram {
                    result.unsatisfied_system_requirements.push(
                        format!("RAM requirement not met: requires {} MB, have {} MB", 
                                min_ram, system_info.ram_mb));
                    result.satisfied = false;
                }
            }
            
            // Check storage requirements
            if let Some(min_storage) = req.min_storage_mb {
                if system_info.available_storage_mb < min_storage {
                    result.unsatisfied_system_requirements.push(
                        format!("Storage requirement not met: requires {} MB, have {} MB", 
                                min_storage, system_info.available_storage_mb));
                    result.satisfied = false;
                }
            }
            
            // Check required hardware features
            for feature in &req.required_features {
                if !system_info.hardware_features.contains(feature) {
                    result.unsatisfied_system_requirements.push(
                        format!("Missing required hardware feature: {}", feature));
                    result.satisfied = false;
                }
            }
            
            // Check kernel version
            if let Some(kernel_req) = &req.kernel_version {
                if !kernel_req.matches(&system_info.kernel_version) {
                    result.unsatisfied_system_requirements.push(
                        format!("Kernel version requirement not met: requires {}, have {}", 
                                kernel_req, system_info.kernel_version));
                    result.satisfied = false;
                }
            }
        }
        
        // Check conflicts
        if let Some(conflicts) = &deps.conflicts {
            for conflict in conflicts {
                for pkg in installed_packages {
                    if pkg.package_name == conflict.package_name && 
                       conflict.version_range.matches(&pkg.version) {
                        result.conflicts.push(conflict.clone());
                        result.satisfied = false;
                    }
                }
            }
        }
    }
    
    // If all dependencies are satisfied, determine installation order
    if result.satisfied {
        result.installation_order = determine_installation_order(metadata, installed_packages);
    }
    
    debug!("Dependency check result: satisfied={}", result.satisfied);
    result
}

/// Determine the installation order for packages.
///
/// # Arguments
///
/// * `metadata` - Metadata for the update package
/// * `installed_packages` - List of installed packages
///
/// # Returns
///
/// List of package names in the order they should be installed.
fn determine_installation_order(
    metadata: &UpdatePackageMetadata,
    installed_packages: &[InstalledPackageInfo],
) -> Vec<String> {
    // Build dependency graph
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    
    // Add the current package
    let package_name = metadata.package_name.clone().unwrap_or_else(|| "system".to_string());
    graph.insert(package_name.clone(), Vec::new());
    in_degree.insert(package_name.clone(), 0);
    
    // Add dependencies
    if let Some(deps) = &metadata.dependencies {
        for dep in &deps.dependencies {
            if !dep.optional {
                graph.entry(package_name.clone())
                    .or_insert_with(Vec::new)
                    .push(dep.package_name.clone());
                
                *in_degree.entry(package_name.clone()).or_insert(0) += 1;
            }
        }
    }
    
    // Add installed packages
    for pkg in installed_packages {
        graph.entry(pkg.package_name.clone()).or_insert_with(Vec::new);
        in_degree.entry(pkg.package_name.clone()).or_insert(0);
    }
    
    // Perform topological sort
    let mut result = Vec::new();
    let mut queue = VecDeque::new();
    
    // Add nodes with no dependencies to the queue
    for (node, degree) in &in_degree {
        if *degree == 0 {
            queue.push_back(node.clone());
        }
    }
    
    while let Some(node) = queue.pop_front() {
        result.push(node.clone());
        
        if let Some(neighbors) = graph.get(&node) {
            for neighbor in neighbors {
                if let Some(degree) = in_degree.get_mut(neighbor) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    }
    
    // Check for cycles
    if result.len() != graph.len() {
        warn!("Dependency cycle detected, using arbitrary order");
        result = graph.keys().cloned().collect();
    }
    
    result
}

/// Information about the system.
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// CPU model.
    pub cpu_model: String,
    
    /// Amount of RAM in MB.
    pub ram_mb: u32,
    
    /// Available storage in MB.
    pub available_storage_mb: u32,
    
    /// Hardware features available on the system.
    pub hardware_features: HashSet<String>,
    
    /// Kernel version.
    pub kernel_version: Version,
}

/// Get information about the system.
///
/// # Returns
///
/// Information about the system.
pub fn get_system_info() -> Result<SystemInfo> {
    debug!("Getting system information");
    
    // Get CPU model
    let cpu_model = std::process::Command::new("sh")
        .arg("-c")
        .arg("cat /proc/cpuinfo | grep 'model name' | head -n1 | cut -d':' -f2 | xargs")
        .output()
        .context("Failed to get CPU model")?;
    
    let cpu_model = String::from_utf8_lossy(&cpu_model.stdout).trim().to_string();
    
    // Get RAM amount
    let mem_info = std::process::Command::new("sh")
        .arg("-c")
        .arg("cat /proc/meminfo | grep 'MemTotal' | awk '{print $2}'")
        .output()
        .context("Failed to get memory info")?;
    
    let mem_kb = String::from_utf8_lossy(&mem_info.stdout).trim().parse::<u32>()
        .context("Failed to parse memory info")?;
    let ram_mb = mem_kb / 1024;
    
    // Get available storage
    let df_output = std::process::Command::new("sh")
        .arg("-c")
        .arg("df -m / | tail -n1 | awk '{print $4}'")
        .output()
        .context("Failed to get storage info")?;
    
    let available_storage_mb = String::from_utf8_lossy(&df_output.stdout).trim().parse::<u32>()
        .context("Failed to parse storage info")?;
    
    // Get hardware features
    let mut hardware_features = HashSet::new();
    
    // Check for TPU
    let tpu_check = std::process::Command::new("sh")
        .arg("-c")
        .arg("lspci | grep -i 'Google Edge TPU'")
        .output()
        .context("Failed to check for TPU")?;
    
    if !tpu_check.stdout.is_empty() {
        hardware_features.insert("tpu".to_string());
    }
    
    // Check for GPU
    let gpu_check = std::process::Command::new("sh")
        .arg("-c")
        .arg("lspci | grep -i 'VGA compatible controller'")
        .output()
        .context("Failed to check for GPU")?;
    
    if !gpu_check.stdout.is_empty() {
        hardware_features.insert("gpu".to_string());
    }
    
    // Check for WiFi
    let wifi_check = std::process::Command::new("sh")
        .arg("-c")
        .arg("lspci | grep -i 'Network controller'")
        .output()
        .context("Failed to check for WiFi")?;
    
    if !wifi_check.stdout.is_empty() {
        hardware_features.insert("wifi".to_string());
    }
    
    // Get kernel version
    let kernel_version = std::process::Command::new("uname")
        .arg("-r")
        .output()
        .context("Failed to get kernel version")?;
    
    let kernel_version_str = String::from_utf8_lossy(&kernel_version.stdout).trim().to_string();
    let kernel_parts: Vec<&str> = kernel_version_str.split('.').collect();
    
    let major = kernel_parts.get(0).unwrap_or(&"0").parse::<u64>().unwrap_or(0);
    let minor = kernel_parts.get(1).unwrap_or(&"0").parse::<u64>().unwrap_or(0);
    let patch = kernel_parts.get(2)
        .map(|s| s.split('-').next().unwrap_or("0"))
        .unwrap_or("0")
        .parse::<u64>()
        .unwrap_or(0);
    
    let kernel_version = Version::new(major, minor, patch);
    
    Ok(SystemInfo {
        cpu_model,
        ram_mb,
        available_storage_mb,
        hardware_features,
        kernel_version,
    })
}

/// Add dependency information to an update package metadata.
///
/// # Arguments
///
/// * `metadata` - Metadata for the update package
/// * `dependencies` - Dependency information to add
///
/// # Returns
///
/// Updated metadata.
pub fn add_dependencies(
    mut metadata: UpdatePackageMetadata,
    dependencies: DependencyInfo,
) -> UpdatePackageMetadata {
    metadata.dependencies = Some(dependencies);
    metadata
}

/// Get installed packages information.
///
/// # Arguments
///
/// * `install_dir` - Directory where packages are installed
///
/// # Returns
///
/// List of installed packages.
pub fn get_installed_packages(install_dir: &Path) -> Result<Vec<InstalledPackageInfo>> {
    debug!("Getting installed packages from: {}", install_dir.display());
    
    let packages_dir = install_dir.join("packages");
    if !packages_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut installed_packages = Vec::new();
    
    for entry in fs::read_dir(&packages_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let info_path = path.join("package_info.json");
            if info_path.exists() {
                let info_file = File::open(&info_path)
                    .context(format!("Failed to open package info file: {}", info_path.display()))?;
                
                let package_info: InstalledPackageInfo = serde_json::from_reader(info_file)
                    .context(format!("Failed to parse package info: {}", info_path.display()))?;
                
                installed_packages.push(package_info);
            }
        }
    }
    
    debug!("Found {} installed packages", installed_packages.len());
    Ok(installed_packages)
}

/// Save installed package information.
///
/// # Arguments
///
/// * `install_dir` - Directory where packages are installed
/// * `package_info` - Information about the installed package
///
/// # Returns
///
/// `Ok(())` if the information was saved successfully.
pub fn save_installed_package(
    install_dir: &Path,
    package_info: &InstalledPackageInfo,
) -> Result<()> {
    debug!("Saving installed package info: {}", package_info.package_name);
    
    let packages_dir = install_dir.join("packages");
    fs::create_dir_all(&packages_dir)
        .context("Failed to create packages directory")?;
    
    let package_dir = packages_dir.join(&package_info.package_name);
    fs::create_dir_all(&package_dir)
        .context("Failed to create package directory")?;
    
    let info_path = package_dir.join("package_info.json");
    let info_file = File::create(&info_path)
        .context("Failed to create package info file")?;
    
    serde_json::to_writer_pretty(info_file, package_info)
        .context("Failed to write package info")?;
    
    Ok(())
}

/// Remove installed package information.
///
/// # Arguments
///
/// * `install_dir` - Directory where packages are installed
/// * `package_name` - Name of the package to remove
///
/// # Returns
///
/// `Ok(())` if the information was removed successfully.
pub fn remove_installed_package(
    install_dir: &Path,
    package_name: &str,
) -> Result<()> {
    debug!("Removing installed package info: {}", package_name);
    
    let packages_dir = install_dir.join("packages");
    if !packages_dir.exists() {
        return Ok(());
    }
    
    let package_dir = packages_dir.join(package_name);
    if package_dir.exists() {
        fs::remove_dir_all(&package_dir)
            .context("Failed to remove package directory")?;
    }
    
    Ok(())
}

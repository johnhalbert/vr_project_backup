//! Validation module for the VR headset system.
//!
//! This module provides comprehensive validation capabilities for the VR headset,
//! including performance benchmarks, stress tests, compatibility tests, security tests,
//! usability tests, and regression tests. All validation components are specifically
//! tailored for the Orange Pi CM5 platform with the RK3588S SoC.

pub mod benchmark;
pub mod stress;
pub mod compatibility;
pub mod security;
pub mod usability;
pub mod regression;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Validation status enum representing the current state of a validation test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationStatus {
    /// Test has not been started
    NotStarted,
    /// Test is currently running
    Running,
    /// Test completed successfully
    Passed,
    /// Test completed with warnings
    Warning,
    /// Test failed
    Failed,
    /// Test was aborted
    Aborted,
}

/// Validation result containing the outcome of a validation test.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Status of the validation test
    pub status: ValidationStatus,
    /// Detailed message about the validation result
    pub message: String,
    /// Timestamp when the validation was performed
    pub timestamp: u64,
    /// Duration of the validation in milliseconds
    pub duration_ms: u64,
    /// Additional metrics collected during validation
    pub metrics: HashMap<String, f64>,
    /// Log entries generated during validation
    pub logs: Vec<String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new(status: ValidationStatus, message: String) -> Self {
        Self {
            status,
            message,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            duration_ms: 0,
            metrics: HashMap::new(),
            logs: Vec::new(),
        }
    }

    /// Add a metric to the validation result
    pub fn add_metric(&mut self, name: &str, value: f64) {
        self.metrics.insert(name.to_string(), value);
    }

    /// Add a log entry to the validation result
    pub fn add_log(&mut self, log: &str) {
        self.logs.push(log.to_string());
    }
}

/// Trait for validation tests
pub trait ValidationTest: Send + Sync {
    /// Get the name of the validation test
    fn name(&self) -> &str;
    
    /// Get the description of the validation test
    fn description(&self) -> &str;
    
    /// Run the validation test
    fn run(&self) -> ValidationResult;
    
    /// Check if the validation test is supported on the current hardware
    fn is_supported(&self) -> bool;
    
    /// Get the estimated duration of the validation test in milliseconds
    fn estimated_duration_ms(&self) -> u64;
    
    /// Get the category of the validation test
    fn category(&self) -> &str;
}

/// Validation manager for running and tracking validation tests
pub struct ValidationManager {
    tests: Vec<Arc<dyn ValidationTest>>,
    results: Mutex<HashMap<String, ValidationResult>>,
}

impl ValidationManager {
    /// Create a new validation manager
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            results: Mutex::new(HashMap::new()),
        }
    }

    /// Register a validation test
    pub fn register_test(&mut self, test: Arc<dyn ValidationTest>) {
        self.tests.push(test);
    }

    /// Run all registered validation tests
    pub fn run_all_tests(&self) -> HashMap<String, ValidationResult> {
        let mut results = HashMap::new();
        
        for test in &self.tests {
            let name = test.name().to_string();
            println!("Running validation test: {}", name);
            
            let result = test.run();
            results.insert(name, result);
        }
        
        // Update the stored results
        let mut stored_results = self.results.lock().unwrap();
        for (name, result) in &results {
            stored_results.insert(name.clone(), result.clone());
        }
        
        results
    }

    /// Run a specific validation test by name
    pub fn run_test(&self, name: &str) -> Option<ValidationResult> {
        for test in &self.tests {
            if test.name() == name {
                let result = test.run();
                
                // Update the stored result
                let mut stored_results = self.results.lock().unwrap();
                stored_results.insert(name.to_string(), result.clone());
                
                return Some(result);
            }
        }
        
        None
    }

    /// Run validation tests by category
    pub fn run_tests_by_category(&self, category: &str) -> HashMap<String, ValidationResult> {
        let mut results = HashMap::new();
        
        for test in &self.tests {
            if test.category() == category {
                let name = test.name().to_string();
                println!("Running validation test: {}", name);
                
                let result = test.run();
                results.insert(name, result);
            }
        }
        
        // Update the stored results
        let mut stored_results = self.results.lock().unwrap();
        for (name, result) in &results {
            stored_results.insert(name.clone(), result.clone());
        }
        
        results
    }

    /// Get the result of a specific validation test
    pub fn get_result(&self, name: &str) -> Option<ValidationResult> {
        let stored_results = self.results.lock().unwrap();
        stored_results.get(name).cloned()
    }

    /// Get all validation test results
    pub fn get_all_results(&self) -> HashMap<String, ValidationResult> {
        let stored_results = self.results.lock().unwrap();
        stored_results.clone()
    }

    /// Get all registered validation tests
    pub fn get_all_tests(&self) -> Vec<(String, String, String, bool, u64)> {
        self.tests
            .iter()
            .map(|test| {
                (
                    test.name().to_string(),
                    test.description().to_string(),
                    test.category().to_string(),
                    test.is_supported(),
                    test.estimated_duration_ms(),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTest {
        name: String,
        description: String,
        category: String,
    }

    impl ValidationTest for MockTest {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn description(&self) -> &str {
            &self.description
        }
        
        fn run(&self) -> ValidationResult {
            ValidationResult::new(ValidationStatus::Passed, "Test passed".to_string())
        }
        
        fn is_supported(&self) -> bool {
            true
        }
        
        fn estimated_duration_ms(&self) -> u64 {
            1000
        }
        
        fn category(&self) -> &str {
            &self.category
        }
    }

    #[test]
    fn test_validation_manager() {
        let mut manager = ValidationManager::new();
        
        let test1 = Arc::new(MockTest {
            name: "test1".to_string(),
            description: "Test 1".to_string(),
            category: "benchmark".to_string(),
        });
        
        let test2 = Arc::new(MockTest {
            name: "test2".to_string(),
            description: "Test 2".to_string(),
            category: "stress".to_string(),
        });
        
        manager.register_test(test1);
        manager.register_test(test2);
        
        assert_eq!(manager.get_all_tests().len(), 2);
        
        let result = manager.run_test("test1").unwrap();
        assert_eq!(result.status, ValidationStatus::Passed);
        
        let results = manager.run_tests_by_category("benchmark");
        assert_eq!(results.len(), 1);
        
        let all_results = manager.run_all_tests();
        assert_eq!(all_results.len(), 2);
    }
}

//! Test harness module for the VR headset system.
//!
//! This module provides a comprehensive testing framework for the VR headset system,
//! supporting both hardware testing on the Orange Pi CM5 and simulated environment testing.
//! The framework is designed to be modular and extensible, supporting unit tests,
//! integration tests, system tests, performance tests, and security tests.

pub mod harness;
pub mod fixtures;
pub mod mocks;
pub mod utils;
pub mod hardware;
pub mod simulation;

use std::fmt;

/// Test environment type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestEnvironment {
    /// Hardware testing environment (requires Orange Pi CM5 hardware)
    Hardware,
    /// Simulated testing environment (no hardware required)
    Simulation,
    /// Hybrid testing environment (some hardware components required)
    Hybrid,
}

impl fmt::Display for TestEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestEnvironment::Hardware => write!(f, "Hardware"),
            TestEnvironment::Simulation => write!(f, "Simulation"),
            TestEnvironment::Hybrid => write!(f, "Hybrid"),
        }
    }
}

/// Test category type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestCategory {
    /// Unit tests
    Unit,
    /// Integration tests
    Integration,
    /// System tests
    System,
    /// Performance tests
    Performance,
    /// Security tests
    Security,
}

impl fmt::Display for TestCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestCategory::Unit => write!(f, "Unit"),
            TestCategory::Integration => write!(f, "Integration"),
            TestCategory::System => write!(f, "System"),
            TestCategory::Performance => write!(f, "Performance"),
            TestCategory::Security => write!(f, "Security"),
        }
    }
}

/// Test result status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    /// Test passed
    Passed,
    /// Test failed
    Failed,
    /// Test skipped
    Skipped,
    /// Test error (test itself had an error)
    Error,
}

impl fmt::Display for TestStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestStatus::Passed => write!(f, "Passed"),
            TestStatus::Failed => write!(f, "Failed"),
            TestStatus::Skipped => write!(f, "Skipped"),
            TestStatus::Error => write!(f, "Error"),
        }
    }
}

/// Test result struct
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Test name
    pub name: String,
    /// Test category
    pub category: TestCategory,
    /// Test environment
    pub environment: TestEnvironment,
    /// Test status
    pub status: TestStatus,
    /// Test message
    pub message: String,
    /// Test duration in milliseconds
    pub duration_ms: u64,
    /// Test metrics
    pub metrics: std::collections::HashMap<String, f64>,
}

impl TestResult {
    /// Create a new test result
    pub fn new(
        name: &str,
        category: TestCategory,
        environment: TestEnvironment,
        status: TestStatus,
        message: &str,
        duration_ms: u64,
    ) -> Self {
        Self {
            name: name.to_string(),
            category,
            environment,
            status,
            message: message.to_string(),
            duration_ms,
            metrics: std::collections::HashMap::new(),
        }
    }

    /// Add a metric to the test result
    pub fn add_metric(&mut self, key: &str, value: f64) {
        self.metrics.insert(key.to_string(), value);
    }

    /// Check if the test passed
    pub fn is_passed(&self) -> bool {
        self.status == TestStatus::Passed
    }

    /// Check if the test failed
    pub fn is_failed(&self) -> bool {
        self.status == TestStatus::Failed
    }

    /// Check if the test was skipped
    pub fn is_skipped(&self) -> bool {
        self.status == TestStatus::Skipped
    }

    /// Check if the test had an error
    pub fn is_error(&self) -> bool {
        self.status == TestStatus::Error
    }
}

/// Test trait for all test implementations
pub trait Test {
    /// Get the test name
    fn name(&self) -> &str;
    
    /// Get the test description
    fn description(&self) -> &str;
    
    /// Get the test category
    fn category(&self) -> TestCategory;
    
    /// Get the test environment
    fn environment(&self) -> TestEnvironment;
    
    /// Check if the test is supported in the current environment
    fn is_supported(&self) -> bool;
    
    /// Run the test
    fn run(&self) -> TestResult;
    
    /// Get the estimated duration of the test in milliseconds
    fn estimated_duration_ms(&self) -> u64;
}

/// Test suite struct
#[derive(Debug)]
pub struct TestSuite {
    /// Test suite name
    pub name: String,
    /// Test suite description
    pub description: String,
    /// Tests in the suite
    pub tests: Vec<Box<dyn Test>>,
}

impl TestSuite {
    /// Create a new test suite
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            tests: Vec::new(),
        }
    }

    /// Add a test to the suite
    pub fn add_test<T: Test + 'static>(&mut self, test: T) {
        self.tests.push(Box::new(test));
    }

    /// Run all tests in the suite
    pub fn run_all(&self) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        for test in &self.tests {
            println!("Running test: {}", test.name());
            let result = test.run();
            println!("Test {} {}", test.name(), result.status);
            results.push(result);
        }
        
        results
    }

    /// Run tests of a specific category
    pub fn run_category(&self, category: TestCategory) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        for test in &self.tests {
            if test.category() == category {
                println!("Running test: {}", test.name());
                let result = test.run();
                println!("Test {} {}", test.name(), result.status);
                results.push(result);
            }
        }
        
        results
    }

    /// Run tests in a specific environment
    pub fn run_environment(&self, environment: TestEnvironment) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        for test in &self.tests {
            if test.environment() == environment {
                println!("Running test: {}", test.name());
                let result = test.run();
                println!("Test {} {}", test.name(), result.status);
                results.push(result);
            }
        }
        
        results
    }

    /// Run supported tests
    pub fn run_supported(&self) -> Vec<TestResult> {
        let mut results = Vec::new();
        
        for test in &self.tests {
            if test.is_supported() {
                println!("Running test: {}", test.name());
                let result = test.run();
                println!("Test {} {}", test.name(), result.status);
                results.push(result);
            } else {
                println!("Skipping unsupported test: {}", test.name());
                results.push(TestResult::new(
                    test.name(),
                    test.category(),
                    test.environment(),
                    TestStatus::Skipped,
                    "Test not supported in current environment",
                    0,
                ));
            }
        }
        
        results
    }

    /// Get test summary
    pub fn get_summary(&self, results: &[TestResult]) -> TestSummary {
        let mut summary = TestSummary {
            total: results.len(),
            passed: 0,
            failed: 0,
            skipped: 0,
            error: 0,
            total_duration_ms: 0,
        };
        
        for result in results {
            match result.status {
                TestStatus::Passed => summary.passed += 1,
                TestStatus::Failed => summary.failed += 1,
                TestStatus::Skipped => summary.skipped += 1,
                TestStatus::Error => summary.error += 1,
            }
            
            summary.total_duration_ms += result.duration_ms;
        }
        
        summary
    }
}

/// Test summary struct
#[derive(Debug, Clone, Copy)]
pub struct TestSummary {
    /// Total number of tests
    pub total: usize,
    /// Number of passed tests
    pub passed: usize,
    /// Number of failed tests
    pub failed: usize,
    /// Number of skipped tests
    pub skipped: usize,
    /// Number of tests with errors
    pub error: usize,
    /// Total duration of all tests in milliseconds
    pub total_duration_ms: u64,
}

impl TestSummary {
    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0 && self.error == 0
    }

    /// Get the pass rate as a percentage
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            100.0 * (self.passed as f64) / (self.total as f64)
        }
    }
}

impl fmt::Display for TestSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Total: {}, Passed: {}, Failed: {}, Skipped: {}, Error: {}, Duration: {}ms, Pass Rate: {:.1}%",
            self.total,
            self.passed,
            self.failed,
            self.skipped,
            self.error,
            self.total_duration_ms,
            self.pass_rate()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTest {
        name: String,
        description: String,
        category: TestCategory,
        environment: TestEnvironment,
        supported: bool,
        status: TestStatus,
    }

    impl Test for MockTest {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn description(&self) -> &str {
            &self.description
        }
        
        fn category(&self) -> TestCategory {
            self.category
        }
        
        fn environment(&self) -> TestEnvironment {
            self.environment
        }
        
        fn is_supported(&self) -> bool {
            self.supported
        }
        
        fn run(&self) -> TestResult {
            TestResult::new(
                &self.name,
                self.category,
                self.environment,
                self.status,
                "Test completed",
                100,
            )
        }
        
        fn estimated_duration_ms(&self) -> u64 {
            100
        }
    }

    #[test]
    fn test_test_suite() {
        let mut suite = TestSuite::new("Test Suite", "Test suite description");
        
        suite.add_test(MockTest {
            name: "Test 1".to_string(),
            description: "Test 1 description".to_string(),
            category: TestCategory::Unit,
            environment: TestEnvironment::Simulation,
            supported: true,
            status: TestStatus::Passed,
        });
        
        suite.add_test(MockTest {
            name: "Test 2".to_string(),
            description: "Test 2 description".to_string(),
            category: TestCategory::Integration,
            environment: TestEnvironment::Hardware,
            supported: false,
            status: TestStatus::Failed,
        });
        
        let results = suite.run_supported();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].status, TestStatus::Passed);
        assert_eq!(results[1].status, TestStatus::Skipped);
        
        let summary = suite.get_summary(&results);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.skipped, 1);
        assert_eq!(summary.error, 0);
    }
}

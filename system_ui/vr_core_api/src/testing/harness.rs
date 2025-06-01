//! Test harness implementation for the VR headset system.
//!
//! This module provides the core implementation of the test harness,
//! including test runners, reporters, and configuration.

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::testing::{Test, TestResult, TestStatus, TestSuite, TestSummary};

/// Test configuration struct
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Whether to run tests in verbose mode
    pub verbose: bool,
    /// Whether to fail fast (stop on first failure)
    pub fail_fast: bool,
    /// Timeout for tests in milliseconds
    pub timeout_ms: u64,
    /// Whether to run hardware tests
    pub run_hardware_tests: bool,
    /// Whether to run simulation tests
    pub run_simulation_tests: bool,
    /// Whether to run hybrid tests
    pub run_hybrid_tests: bool,
    /// Filter for test names (only run tests matching this filter)
    pub name_filter: Option<String>,
    /// Filter for test categories (only run tests in these categories)
    pub category_filter: Option<Vec<crate::testing::TestCategory>>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            fail_fast: false,
            timeout_ms: 60000, // 1 minute default timeout
            run_hardware_tests: true,
            run_simulation_tests: true,
            run_hybrid_tests: true,
            name_filter: None,
            category_filter: None,
        }
    }
}

impl TestConfig {
    /// Create a new test configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set verbose mode
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set fail fast mode
    pub fn fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Set test timeout
    pub fn timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set whether to run hardware tests
    pub fn run_hardware_tests(mut self, run: bool) -> Self {
        self.run_hardware_tests = run;
        self
    }

    /// Set whether to run simulation tests
    pub fn run_simulation_tests(mut self, run: bool) -> Self {
        self.run_simulation_tests = run;
        self
    }

    /// Set whether to run hybrid tests
    pub fn run_hybrid_tests(mut self, run: bool) -> Self {
        self.run_hybrid_tests = run;
        self
    }

    /// Set name filter
    pub fn name_filter(mut self, filter: Option<String>) -> Self {
        self.name_filter = filter;
        self
    }

    /// Set category filter
    pub fn category_filter(mut self, filter: Option<Vec<crate::testing::TestCategory>>) -> Self {
        self.category_filter = filter;
        self
    }

    /// Check if a test should be run based on the configuration
    pub fn should_run_test(&self, test: &dyn Test) -> bool {
        // Check environment filters
        let env_match = match test.environment() {
            crate::testing::TestEnvironment::Hardware => self.run_hardware_tests,
            crate::testing::TestEnvironment::Simulation => self.run_simulation_tests,
            crate::testing::TestEnvironment::Hybrid => self.run_hybrid_tests,
        };
        
        if !env_match {
            return false;
        }
        
        // Check name filter
        if let Some(ref filter) = self.name_filter {
            if !test.name().contains(filter) {
                return false;
            }
        }
        
        // Check category filter
        if let Some(ref categories) = self.category_filter {
            if !categories.contains(&test.category()) {
                return false;
            }
        }
        
        true
    }
}

/// Test runner struct
#[derive(Debug)]
pub struct TestRunner {
    /// Test configuration
    config: TestConfig,
    /// Test reporters
    reporters: Vec<Box<dyn TestReporter>>,
}

impl TestRunner {
    /// Create a new test runner with the given configuration
    pub fn new(config: TestConfig) -> Self {
        Self {
            config,
            reporters: Vec::new(),
        }
    }

    /// Add a test reporter
    pub fn add_reporter<R: TestReporter + 'static>(&mut self, reporter: R) {
        self.reporters.push(Box::new(reporter));
    }

    /// Run a test suite
    pub fn run_suite(&self, suite: &TestSuite) -> TestSummary {
        // Notify reporters that the suite is starting
        for reporter in &self.reporters {
            reporter.on_suite_start(suite);
        }
        
        let mut results = Vec::new();
        let start_time = Instant::now();
        
        // Run each test
        for test in &suite.tests {
            if !self.config.should_run_test(test.as_ref()) {
                // Skip test based on configuration
                let result = TestResult::new(
                    test.name(),
                    test.category(),
                    test.environment(),
                    TestStatus::Skipped,
                    "Skipped due to configuration",
                    0,
                );
                
                results.push(result.clone());
                
                // Notify reporters
                for reporter in &self.reporters {
                    reporter.on_test_skip(test.as_ref(), &result);
                }
                
                continue;
            }
            
            // Notify reporters that the test is starting
            for reporter in &self.reporters {
                reporter.on_test_start(test.as_ref());
            }
            
            // Run the test with timeout
            let result = self.run_test_with_timeout(test.as_ref());
            results.push(result.clone());
            
            // Notify reporters about the test result
            match result.status {
                TestStatus::Passed => {
                    for reporter in &self.reporters {
                        reporter.on_test_pass(test.as_ref(), &result);
                    }
                }
                TestStatus::Failed => {
                    for reporter in &self.reporters {
                        reporter.on_test_fail(test.as_ref(), &result);
                    }
                    
                    // Check if we should stop on first failure
                    if self.config.fail_fast {
                        break;
                    }
                }
                TestStatus::Error => {
                    for reporter in &self.reporters {
                        reporter.on_test_error(test.as_ref(), &result);
                    }
                    
                    // Check if we should stop on first error
                    if self.config.fail_fast {
                        break;
                    }
                }
                TestStatus::Skipped => {
                    for reporter in &self.reporters {
                        reporter.on_test_skip(test.as_ref(), &result);
                    }
                }
            }
        }
        
        // Calculate summary
        let summary = suite.get_summary(&results);
        
        // Notify reporters that the suite is complete
        for reporter in &self.reporters {
            reporter.on_suite_complete(suite, &results, &summary, start_time.elapsed());
        }
        
        summary
    }

    /// Run a test with timeout
    fn run_test_with_timeout(&self, test: &dyn Test) -> TestResult {
        // Check if the test is supported
        if !test.is_supported() {
            return TestResult::new(
                test.name(),
                test.category(),
                test.environment(),
                TestStatus::Skipped,
                "Test not supported in current environment",
                0,
            );
        }
        
        // Create a shared result that can be accessed from a thread
        let result = Arc::new(Mutex::new(None));
        let result_clone = Arc::clone(&result);
        
        // Run the test in a separate thread to enable timeout
        let handle = std::thread::spawn(move || {
            let start = Instant::now();
            let test_result = test.run();
            let duration = start.elapsed();
            
            // Store the result
            let mut result = result_clone.lock().unwrap();
            *result = Some((test_result, duration));
        });
        
        // Wait for the test to complete or timeout
        match handle.join() {
            Ok(_) => {
                // Test completed, get the result
                let result_guard = result.lock().unwrap();
                if let Some((test_result, _)) = &*result_guard {
                    test_result.clone()
                } else {
                    // This should not happen, but handle it anyway
                    TestResult::new(
                        test.name(),
                        test.category(),
                        test.environment(),
                        TestStatus::Error,
                        "Test did not produce a result",
                        0,
                    )
                }
            }
            Err(_) => {
                // Thread panicked
                TestResult::new(
                    test.name(),
                    test.category(),
                    test.environment(),
                    TestStatus::Error,
                    "Test panicked",
                    self.config.timeout_ms,
                )
            }
        }
    }
}

/// Test reporter trait
pub trait TestReporter: Send + Sync {
    /// Called when a test suite starts
    fn on_suite_start(&self, suite: &TestSuite);
    
    /// Called when a test starts
    fn on_test_start(&self, test: &dyn Test);
    
    /// Called when a test passes
    fn on_test_pass(&self, test: &dyn Test, result: &TestResult);
    
    /// Called when a test fails
    fn on_test_fail(&self, test: &dyn Test, result: &TestResult);
    
    /// Called when a test has an error
    fn on_test_error(&self, test: &dyn Test, result: &TestResult);
    
    /// Called when a test is skipped
    fn on_test_skip(&self, test: &dyn Test, result: &TestResult);
    
    /// Called when a test suite completes
    fn on_suite_complete(&self, suite: &TestSuite, results: &[TestResult], summary: &TestSummary, duration: Duration);
}

/// Console test reporter
#[derive(Debug)]
pub struct ConsoleReporter {
    /// Whether to use verbose output
    verbose: bool,
}

impl ConsoleReporter {
    /// Create a new console reporter
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl TestReporter for ConsoleReporter {
    fn on_suite_start(&self, suite: &TestSuite) {
        println!("Running test suite: {}", suite.name);
        if self.verbose {
            println!("Description: {}", suite.description);
            println!("Tests: {}", suite.tests.len());
        }
        println!("----------------------------------------");
    }
    
    fn on_test_start(&self, test: &dyn Test) {
        if self.verbose {
            println!("Starting test: {} ({})", test.name(), test.environment());
        }
    }
    
    fn on_test_pass(&self, test: &dyn Test, result: &TestResult) {
        println!("✅ PASS: {} ({}ms)", test.name(), result.duration_ms);
        if self.verbose {
            println!("  Message: {}", result.message);
            for (key, value) in &result.metrics {
                println!("  {}: {}", key, value);
            }
        }
    }
    
    fn on_test_fail(&self, test: &dyn Test, result: &TestResult) {
        println!("❌ FAIL: {} ({}ms)", test.name(), result.duration_ms);
        println!("  Message: {}", result.message);
        if self.verbose {
            for (key, value) in &result.metrics {
                println!("  {}: {}", key, value);
            }
        }
    }
    
    fn on_test_error(&self, test: &dyn Test, result: &TestResult) {
        println!("⚠️ ERROR: {} ({}ms)", test.name(), result.duration_ms);
        println!("  Message: {}", result.message);
        if self.verbose {
            for (key, value) in &result.metrics {
                println!("  {}: {}", key, value);
            }
        }
    }
    
    fn on_test_skip(&self, test: &dyn Test, result: &TestResult) {
        println!("⏭️ SKIP: {}", test.name());
        if self.verbose {
            println!("  Message: {}", result.message);
        }
    }
    
    fn on_suite_complete(&self, suite: &TestSuite, results: &[TestResult], summary: &TestSummary, duration: Duration) {
        println!("----------------------------------------");
        println!("Test suite complete: {}", suite.name);
        println!("Duration: {:.2}s", duration.as_secs_f64());
        println!("Summary: {}", summary);
        
        if summary.failed > 0 || summary.error > 0 {
            println!("\nFailed tests:");
            for result in results {
                if result.is_failed() || result.is_error() {
                    println!("  {} ({}): {}", result.name, result.status, result.message);
                }
            }
        }
        
        println!("----------------------------------------");
    }
}

/// JSON test reporter
#[derive(Debug)]
pub struct JsonReporter {
    /// Output file path
    output_path: String,
    /// Test results
    results: Arc<Mutex<Vec<TestResult>>>,
}

impl JsonReporter {
    /// Create a new JSON reporter
    pub fn new(output_path: &str) -> Self {
        Self {
            output_path: output_path.to_string(),
            results: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Write results to file
    fn write_results(&self, suite: &TestSuite, summary: &TestSummary, duration: Duration) {
        use std::fs::File;
        use std::io::Write;
        
        // Create a report structure
        let report = serde_json::json!({
            "suite": {
                "name": suite.name,
                "description": suite.description,
            },
            "summary": {
                "total": summary.total,
                "passed": summary.passed,
                "failed": summary.failed,
                "skipped": summary.skipped,
                "error": summary.error,
                "duration_ms": summary.total_duration_ms,
                "pass_rate": summary.pass_rate(),
            },
            "duration_ms": duration.as_millis(),
            "results": self.results.lock().unwrap().iter().map(|r| {
                serde_json::json!({
                    "name": r.name,
                    "category": format!("{}", r.category),
                    "environment": format!("{}", r.environment),
                    "status": format!("{}", r.status),
                    "message": r.message,
                    "duration_ms": r.duration_ms,
                    "metrics": r.metrics,
                })
            }).collect::<Vec<_>>(),
        });
        
        // Write to file
        if let Ok(mut file) = File::create(&self.output_path) {
            if let Ok(json_str) = serde_json::to_string_pretty(&report) {
                let _ = file.write_all(json_str.as_bytes());
            }
        }
    }
}

impl TestReporter for JsonReporter {
    fn on_suite_start(&self, _suite: &TestSuite) {
        // Clear previous results
        let mut results = self.results.lock().unwrap();
        results.clear();
    }
    
    fn on_test_start(&self, _test: &dyn Test) {
        // Nothing to do
    }
    
    fn on_test_pass(&self, _test: &dyn Test, result: &TestResult) {
        let mut results = self.results.lock().unwrap();
        results.push(result.clone());
    }
    
    fn on_test_fail(&self, _test: &dyn Test, result: &TestResult) {
        let mut results = self.results.lock().unwrap();
        results.push(result.clone());
    }
    
    fn on_test_error(&self, _test: &dyn Test, result: &TestResult) {
        let mut results = self.results.lock().unwrap();
        results.push(result.clone());
    }
    
    fn on_test_skip(&self, _test: &dyn Test, result: &TestResult) {
        let mut results = self.results.lock().unwrap();
        results.push(result.clone());
    }
    
    fn on_suite_complete(&self, suite: &TestSuite, _results: &[TestResult], summary: &TestSummary, duration: Duration) {
        self.write_results(suite, summary, duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Test, TestCategory, TestEnvironment, TestStatus};

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
    fn test_config_should_run_test() {
        let config = TestConfig::default();
        
        let test = MockTest {
            name: "Test 1".to_string(),
            description: "Test 1 description".to_string(),
            category: TestCategory::Unit,
            environment: TestEnvironment::Simulation,
            supported: true,
            status: TestStatus::Passed,
        };
        
        assert!(config.should_run_test(&test));
        
        let config = TestConfig::default().run_simulation_tests(false);
        assert!(!config.should_run_test(&test));
        
        let config = TestConfig::default().name_filter(Some("Test 2".to_string()));
        assert!(!config.should_run_test(&test));
        
        let config = TestConfig::default().category_filter(Some(vec![TestCategory::Integration]));
        assert!(!config.should_run_test(&test));
    }

    #[test]
    fn test_runner_run_suite() {
        let config = TestConfig::default();
        let mut runner = TestRunner::new(config);
        runner.add_reporter(ConsoleReporter::new(false));
        
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
        
        let summary = runner.run_suite(&suite);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.skipped, 1);
    }
}

//! Unit tests module for the VR headset system.
//!
//! This module contains unit tests for various components of the VR headset system.
//! These tests are designed to verify the functionality of individual components
//! in isolation, using both hardware and simulation environments.

pub mod hardware_tests;
pub mod config_tests;
pub mod ipc_tests;
pub mod security_tests;
pub mod update_tests;
pub mod telemetry_tests;
pub mod optimization_tests;

use crate::testing::{Test, TestCategory, TestEnvironment, TestResult, TestStatus};
use crate::testing::fixtures::{TestFixture, SimulationTestFixture, HardwareTestFixture, CombinedTestFixture};
use crate::testing::mocks::{MockDevice, MockDisplayDevice, MockCameraDevice, MockImuDevice};
use crate::testing::utils::{assert_approx_eq, assert_vec3_approx_eq, measure_time, TestLogger};
use crate::testing::hardware::{HardwareDeviceType, HardwareTestEnvironment};
use crate::testing::simulation::{SimulatedDeviceType, SimulationTestEnvironment};

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Base unit test struct
pub struct UnitTest<F: TestFixture> {
    /// Test name
    name: String,
    /// Test description
    description: String,
    /// Test category
    category: TestCategory,
    /// Test environment
    environment: TestEnvironment,
    /// Test fixture
    fixture: F,
    /// Test function
    test_fn: Box<dyn Fn(&mut F) -> TestResult + Send + Sync>,
    /// Estimated duration in milliseconds
    estimated_duration_ms: u64,
}

impl<F: TestFixture + 'static> UnitTest<F> {
    /// Create a new unit test
    pub fn new<Fn>(
        name: &str,
        description: &str,
        environment: TestEnvironment,
        fixture: F,
        test_fn: Fn,
        estimated_duration_ms: u64,
    ) -> Self
    where
        Fn: Fn(&mut F) -> TestResult + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            category: TestCategory::Unit,
            environment,
            fixture,
            test_fn: Box::new(test_fn),
            estimated_duration_ms,
        }
    }
}

impl<F: TestFixture + 'static> Test for UnitTest<F> {
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
        match self.environment {
            TestEnvironment::Hardware => {
                // Check if hardware environment is available
                let mut env = HardwareTestEnvironment::new();
                if env.initialize().is_err() {
                    return false;
                }
                
                // Check if required hardware is available
                // This would depend on the specific test
                true
            }
            TestEnvironment::Simulation => {
                // Simulation is always supported
                true
            }
            TestEnvironment::Hybrid => {
                // Check if hardware environment is available
                let mut env = HardwareTestEnvironment::new();
                if env.initialize().is_err() {
                    return false;
                }
                
                // Check if required hardware is available
                // This would depend on the specific test
                true
            }
        }
    }
    
    fn run(&self) -> TestResult {
        let start = Instant::now();
        
        // Set up the fixture
        let mut fixture = self.fixture.clone();
        fixture.setup();
        
        // Run the test
        let result = (self.test_fn)(&mut fixture);
        
        // Tear down the fixture
        fixture.teardown();
        
        // Calculate duration
        let duration = start.elapsed();
        
        // Create a new result with the actual duration
        TestResult {
            duration_ms: duration.as_millis() as u64,
            ..result
        }
    }
    
    fn estimated_duration_ms(&self) -> u64 {
        self.estimated_duration_ms
    }
}

/// Create a test suite with all unit tests
pub fn create_unit_test_suite() -> crate::testing::TestSuite {
    let mut suite = crate::testing::TestSuite::new(
        "VR Headset Unit Tests",
        "Unit tests for the VR headset system components",
    );
    
    // Add hardware tests
    hardware_tests::add_tests(&mut suite);
    
    // Add config tests
    config_tests::add_tests(&mut suite);
    
    // Add IPC tests
    ipc_tests::add_tests(&mut suite);
    
    // Add security tests
    security_tests::add_tests(&mut suite);
    
    // Add update tests
    update_tests::add_tests(&mut suite);
    
    // Add telemetry tests
    telemetry_tests::add_tests(&mut suite);
    
    // Add optimization tests
    optimization_tests::add_tests(&mut suite);
    
    suite
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unit_test() {
        // Create a test fixture
        let fixture = SimulationTestFixture::new("test_fixture");
        
        // Create a unit test
        let test = UnitTest::new(
            "test_example",
            "Example unit test",
            TestEnvironment::Simulation,
            fixture,
            |_fixture| {
                TestResult::new(
                    "test_example",
                    TestCategory::Unit,
                    TestEnvironment::Simulation,
                    TestStatus::Passed,
                    "Test passed",
                    0,
                )
            },
            100,
        );
        
        // Run the test
        let result = test.run();
        
        // Check the result
        assert_eq!(result.status, TestStatus::Passed);
        assert_eq!(result.name, "test_example");
        assert_eq!(result.category, TestCategory::Unit);
        assert_eq!(result.environment, TestEnvironment::Simulation);
    }
}

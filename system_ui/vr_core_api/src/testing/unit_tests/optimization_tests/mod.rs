//! Optimization unit tests module for the VR headset system.
//!
//! This module contains unit tests for the optimization components of the VR headset system,
//! including CPU, GPU, memory, storage, network, and power optimization.

use crate::testing::{Test, TestCategory, TestEnvironment, TestResult, TestStatus};
use crate::testing::fixtures::{TestFixture, SimulationTestFixture, HardwareTestFixture, CombinedTestFixture};
use crate::testing::mocks::{MockDevice, MockDisplayDevice, MockCameraDevice, MockImuDevice};
use crate::testing::utils::{assert_approx_eq, assert_vec3_approx_eq, measure_time, TestLogger};
use crate::testing::unit_tests::UnitTest;

use crate::optimization::cpu::{CpuOptimizer, CpuGovernor, CpuFrequency, CpuCore, CpuAffinity, CpuPriority};
use crate::optimization::gpu::{GpuOptimizer, GpuFrequency, GpuPowerState, ShaderCache, RenderingPipeline};
use crate::optimization::memory::{MemoryOptimizer, MemoryAllocationStrategy, MemoryCompressionLevel, HugePageSupport};
use crate::optimization::storage::{StorageOptimizer, IoScheduler, CachePolicy, ReadAheadBuffer};
use crate::optimization::network::{NetworkOptimizer, QosPolicy, TcpConfiguration, BufferSize};
use crate::optimization::power::{PowerOptimizer, PowerProfile, PowerState, ThermalThrottling};

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::thread;

/// Add optimization tests to the test suite
pub fn add_tests(suite: &mut crate::testing::TestSuite) {
    // Add CPU optimization tests
    add_cpu_tests(suite);
    
    // Add GPU optimization tests
    add_gpu_tests(suite);
    
    // Add memory optimization tests
    add_memory_tests(suite);
    
    // Add storage optimization tests
    add_storage_tests(suite);
    
    // Add network optimization tests
    add_network_tests(suite);
    
    // Add power optimization tests
    add_power_tests(suite);
}

/// Add CPU optimization tests to the test suite
fn add_cpu_tests(suite: &mut crate::testing::TestSuite) {
    // Test CPU governor selection
    let sim_fixture = SimulationTestFixture::new("cpu_governor_sim");
    let cpu_governor_test = UnitTest::new(
        "cpu_governor",
        "Test CPU governor selection",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a CPU optimizer
            let mut optimizer = CpuOptimizer::new();
            
            // Set up mock CPU cores
            optimizer.set_mock_cores(vec![
                CpuCore::new(0, CpuFrequency::new(500, 2000)), // 500MHz - 2GHz
                CpuCore::new(1, CpuFrequency::new(500, 2000)),
                CpuCore::new(2, CpuFrequency::new(500, 2000)),
                CpuCore::new(3, CpuFrequency::new(500, 2000)),
            ]);
            
            // Test performance governor
            optimizer.set_governor(CpuGovernor::Performance);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply performance governor: {:?}", result.err());
            
            // Check that all cores are at maximum frequency
            for i in 0..4 {
                let freq = optimizer.get_core_frequency(i).unwrap();
                assert_eq!(freq.current(), 2000, "Core {} should be at maximum frequency", i);
            }
            
            // Test powersave governor
            optimizer.set_governor(CpuGovernor::PowerSave);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply powersave governor: {:?}", result.err());
            
            // Check that all cores are at minimum frequency
            for i in 0..4 {
                let freq = optimizer.get_core_frequency(i).unwrap();
                assert_eq!(freq.current(), 500, "Core {} should be at minimum frequency", i);
            }
            
            // Test ondemand governor
            optimizer.set_governor(CpuGovernor::OnDemand);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply ondemand governor: {:?}", result.err());
            
            // Simulate high load on core 0
            optimizer.set_mock_core_load(0, 90);
            
            // Apply settings again
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply settings with high load: {:?}", result.err());
            
            // Check that core 0 is at high frequency due to high load
            let freq = optimizer.get_core_frequency(0).unwrap();
            assert!(freq.current() > 1500, "Core 0 should be at high frequency due to high load");
            
            // Create test result
            TestResult::new(
                "cpu_governor",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "CPU governor selection test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(cpu_governor_test);
    
    // Test CPU affinity optimization
    let sim_fixture = SimulationTestFixture::new("cpu_affinity_sim");
    let cpu_affinity_test = UnitTest::new(
        "cpu_affinity",
        "Test CPU affinity optimization",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a CPU optimizer
            let mut optimizer = CpuOptimizer::new();
            
            // Set up mock CPU cores
            optimizer.set_mock_cores(vec![
                CpuCore::new(0, CpuFrequency::new(500, 2000)), // 500MHz - 2GHz
                CpuCore::new(1, CpuFrequency::new(500, 2000)),
                CpuCore::new(2, CpuFrequency::new(500, 2000)),
                CpuCore::new(3, CpuFrequency::new(500, 2000)),
            ]);
            
            // Create process IDs for testing
            let render_process = 1001;
            let tracking_process = 1002;
            let background_process = 1003;
            
            // Set affinity for VR processes
            optimizer.set_process_affinity(render_process, CpuAffinity::new(vec![0, 1]));
            optimizer.set_process_affinity(tracking_process, CpuAffinity::new(vec![2]));
            optimizer.set_process_affinity(background_process, CpuAffinity::new(vec![3]));
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply CPU affinity settings: {:?}", result.err());
            
            // Check process affinities
            let render_affinity = optimizer.get_process_affinity(render_process).unwrap();
            let tracking_affinity = optimizer.get_process_affinity(tracking_process).unwrap();
            let background_affinity = optimizer.get_process_affinity(background_process).unwrap();
            
            assert_eq!(render_affinity.cores(), &[0, 1], "Render process should be on cores 0 and 1");
            assert_eq!(tracking_affinity.cores(), &[2], "Tracking process should be on core 2");
            assert_eq!(background_affinity.cores(), &[3], "Background process should be on core 3");
            
            // Test affinity optimization for VR workload
            optimizer.optimize_for_vr_workload();
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply VR workload optimization: {:?}", result.err());
            
            // Check that render and tracking processes have higher priority cores
            let render_affinity = optimizer.get_process_affinity(render_process).unwrap();
            let tracking_affinity = optimizer.get_process_affinity(tracking_process).unwrap();
            
            // Render and tracking should be on the performance cores (0, 1, 2)
            assert!(render_affinity.cores().iter().all(|&c| c <= 2), "Render process should be on performance cores");
            assert!(tracking_affinity.cores().iter().all(|&c| c <= 2), "Tracking process should be on performance cores");
            
            // Create test result
            TestResult::new(
                "cpu_affinity",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "CPU affinity optimization test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(cpu_affinity_test);
    
    // Test CPU priority optimization
    let sim_fixture = SimulationTestFixture::new("cpu_priority_sim");
    let cpu_priority_test = UnitTest::new(
        "cpu_priority",
        "Test CPU priority optimization",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a CPU optimizer
            let mut optimizer = CpuOptimizer::new();
            
            // Create process IDs for testing
            let render_process = 1001;
            let tracking_process = 1002;
            let background_process = 1003;
            
            // Set priorities for VR processes
            optimizer.set_process_priority(render_process, CpuPriority::High);
            optimizer.set_process_priority(tracking_process, CpuPriority::RealTime);
            optimizer.set_process_priority(background_process, CpuPriority::Low);
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply CPU priority settings: {:?}", result.err());
            
            // Check process priorities
            let render_priority = optimizer.get_process_priority(render_process).unwrap();
            let tracking_priority = optimizer.get_process_priority(tracking_process).unwrap();
            let background_priority = optimizer.get_process_priority(background_process).unwrap();
            
            assert_eq!(render_priority, CpuPriority::High, "Render process should have high priority");
            assert_eq!(tracking_priority, CpuPriority::RealTime, "Tracking process should have real-time priority");
            assert_eq!(background_priority, CpuPriority::Low, "Background process should have low priority");
            
            // Test priority optimization for VR workload
            optimizer.optimize_for_vr_workload();
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply VR workload optimization: {:?}", result.err());
            
            // Check that critical processes have higher priority
            let render_priority = optimizer.get_process_priority(render_process).unwrap();
            let tracking_priority = optimizer.get_process_priority(tracking_process).unwrap();
            let background_priority = optimizer.get_process_priority(background_process).unwrap();
            
            assert!(render_priority >= CpuPriority::High, "Render process should have at least high priority");
            assert_eq!(tracking_priority, CpuPriority::RealTime, "Tracking process should have real-time priority");
            assert!(background_priority <= CpuPriority::Normal, "Background process should have at most normal priority");
            
            // Create test result
            TestResult::new(
                "cpu_priority",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "CPU priority optimization test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(cpu_priority_test);
}

/// Add GPU optimization tests to the test suite
fn add_gpu_tests(suite: &mut crate::testing::TestSuite) {
    // Test GPU frequency scaling
    let sim_fixture = SimulationTestFixture::new("gpu_frequency_sim");
    let gpu_frequency_test = UnitTest::new(
        "gpu_frequency",
        "Test GPU frequency scaling",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a GPU optimizer
            let mut optimizer = GpuOptimizer::new();
            
            // Set up mock GPU
            optimizer.set_mock_gpu_frequency(GpuFrequency::new(200, 1000)); // 200MHz - 1GHz
            
            // Test maximum performance
            optimizer.set_performance_level(1.0); // 100%
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply maximum performance: {:?}", result.err());
            
            // Check that GPU is at maximum frequency
            let freq = optimizer.get_gpu_frequency().unwrap();
            assert_eq!(freq.current(), 1000, "GPU should be at maximum frequency");
            
            // Test minimum performance
            optimizer.set_performance_level(0.0); // 0%
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply minimum performance: {:?}", result.err());
            
            // Check that GPU is at minimum frequency
            let freq = optimizer.get_gpu_frequency().unwrap();
            assert_eq!(freq.current(), 200, "GPU should be at minimum frequency");
            
            // Test intermediate performance
            optimizer.set_performance_level(0.5); // 50%
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply intermediate performance: {:?}", result.err());
            
            // Check that GPU is at intermediate frequency
            let freq = optimizer.get_gpu_frequency().unwrap();
            assert_approx_eq!(freq.current() as f64, 600.0, 10.0, "GPU should be at intermediate frequency");
            
            // Create test result
            TestResult::new(
                "gpu_frequency",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "GPU frequency scaling test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(gpu_frequency_test);
    
    // Test shader cache optimization
    let sim_fixture = SimulationTestFixture::new("shader_cache_sim");
    let shader_cache_test = UnitTest::new(
        "shader_cache",
        "Test shader cache optimization",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a GPU optimizer
            let mut optimizer = GpuOptimizer::new();
            
            // Create a shader cache
            let mut cache = ShaderCache::new();
            
            // Add some shaders to the cache
            cache.add_shader("vertex_shader_1", "shader source 1");
            cache.add_shader("fragment_shader_1", "shader source 2");
            cache.add_shader("vertex_shader_2", "shader source 3");
            cache.add_shader("fragment_shader_2", "shader source 4");
            
            // Set the cache in the optimizer
            optimizer.set_shader_cache(cache);
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply shader cache settings: {:?}", result.err());
            
            // Check cache hit rate (should be 0% initially)
            let hit_rate = optimizer.get_shader_cache_hit_rate().unwrap();
            assert_eq!(hit_rate, 0.0, "Initial hit rate should be 0%");
            
            // Simulate shader lookups
            optimizer.lookup_shader("vertex_shader_1");
            optimizer.lookup_shader("fragment_shader_1");
            optimizer.lookup_shader("vertex_shader_2");
            optimizer.lookup_shader("fragment_shader_2");
            optimizer.lookup_shader("vertex_shader_3"); // Not in cache
            
            // Check cache hit rate (should be 80% now)
            let hit_rate = optimizer.get_shader_cache_hit_rate().unwrap();
            assert_approx_eq!(hit_rate, 0.8, 0.01, "Hit rate should be 80%");
            
            // Test cache optimization
            optimizer.optimize_shader_cache();
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply shader cache optimization: {:?}", result.err());
            
            // Check that frequently used shaders are prioritized
            let priorities = optimizer.get_shader_priorities();
            assert!(priorities.get("vertex_shader_1").unwrap() > priorities.get("vertex_shader_3").unwrap_or(&0),
                   "Frequently used shaders should have higher priority");
            
            // Create test result
            TestResult::new(
                "shader_cache",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Shader cache optimization test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(shader_cache_test);
    
    // Test rendering pipeline optimization
    let sim_fixture = SimulationTestFixture::new("rendering_pipeline_sim");
    let rendering_pipeline_test = UnitTest::new(
        "rendering_pipeline",
        "Test rendering pipeline optimization",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a GPU optimizer
            let mut optimizer = GpuOptimizer::new();
            
            // Create a rendering pipeline
            let mut pipeline = RenderingPipeline::new();
            
            // Add some stages to the pipeline
            pipeline.add_stage("geometry", 5.0); // 5ms
            pipeline.add_stage("lighting", 3.0); // 3ms
            pipeline.add_stage("post_processing", 2.0); // 2ms
            
            // Set the pipeline in the optimizer
            optimizer.set_rendering_pipeline(pipeline);
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply rendering pipeline settings: {:?}", result.err());
            
            // Check total pipeline time
            let total_time = optimizer.get_pipeline_total_time().unwrap();
            assert_approx_eq!(total_time, 10.0, 0.1, "Total pipeline time should be 10ms");
            
            // Test pipeline optimization
            optimizer.optimize_rendering_pipeline();
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply pipeline optimization: {:?}", result.err());
            
            // Check that the pipeline is optimized
            let optimized_time = optimizer.get_pipeline_total_time().unwrap();
            assert!(optimized_time < total_time, "Optimized pipeline should be faster");
            
            // Check stage-specific optimizations
            let stage_times = optimizer.get_pipeline_stage_times();
            assert!(stage_times.get("geometry").unwrap() < &5.0, "Geometry stage should be optimized");
            
            // Create test result
            TestResult::new(
                "rendering_pipeline",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Rendering pipeline optimization test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(rendering_pipeline_test);
}

/// Add memory optimization tests to the test suite
fn add_memory_tests(suite: &mut crate::testing::TestSuite) {
    // Test memory allocation strategy
    let sim_fixture = SimulationTestFixture::new("memory_allocation_sim");
    let memory_allocation_test = UnitTest::new(
        "memory_allocation",
        "Test memory allocation strategy",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a memory optimizer
            let mut optimizer = MemoryOptimizer::new();
            
            // Set up mock memory
            optimizer.set_mock_total_memory(16 * 1024); // 16 GB
            optimizer.set_mock_available_memory(12 * 1024); // 12 GB available
            
            // Test different allocation strategies
            
            // Conservative strategy
            optimizer.set_allocation_strategy(MemoryAllocationStrategy::Conservative);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply conservative strategy: {:?}", result.err());
            
            // Check allocation limits
            let vr_limit = optimizer.get_vr_memory_limit().unwrap();
            let system_limit = optimizer.get_system_memory_limit().unwrap();
            
            assert!(vr_limit < 12 * 1024, "VR memory limit should be less than available memory");
            assert!(system_limit > 0, "System memory limit should be positive");
            assert!(vr_limit + system_limit <= 16 * 1024, "Total limits should not exceed total memory");
            
            // Aggressive strategy
            optimizer.set_allocation_strategy(MemoryAllocationStrategy::Aggressive);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply aggressive strategy: {:?}", result.err());
            
            // Check allocation limits
            let new_vr_limit = optimizer.get_vr_memory_limit().unwrap();
            let new_system_limit = optimizer.get_system_memory_limit().unwrap();
            
            assert!(new_vr_limit > vr_limit, "Aggressive VR memory limit should be higher");
            assert!(new_system_limit < system_limit, "Aggressive system memory limit should be lower");
            
            // Balanced strategy
            optimizer.set_allocation_strategy(MemoryAllocationStrategy::Balanced);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply balanced strategy: {:?}", result.err());
            
            // Check allocation limits
            let balanced_vr_limit = optimizer.get_vr_memory_limit().unwrap();
            
            assert!(balanced_vr_limit > vr_limit, "Balanced VR memory limit should be higher than conservative");
            assert!(balanced_vr_limit < new_vr_limit, "Balanced VR memory limit should be lower than aggressive");
            
            // Create test result
            TestResult::new(
                "memory_allocation",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Memory allocation strategy test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(memory_allocation_test);
    
    // Test huge pages support
    let sim_fixture = SimulationTestFixture::new("huge_pages_sim");
    let huge_pages_test = UnitTest::new(
        "huge_pages",
        "Test huge pages support",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a memory optimizer
            let mut optimizer = MemoryOptimizer::new();
            
            // Check if huge pages are supported
            let supported = optimizer.is_huge_pages_supported();
            
            // If supported, test enabling huge pages
            if supported {
                // Enable huge pages
                optimizer.set_huge_pages_support(HugePageSupport::Enabled);
                let result = optimizer.apply_settings();
                assert!(result.is_ok(), "Failed to enable huge pages: {:?}", result.err());
                
                // Check that huge pages are enabled
                let status = optimizer.get_huge_pages_status().unwrap();
                assert_eq!(status, HugePageSupport::Enabled, "Huge pages should be enabled");
                
                // Check huge page allocation
                let allocated = optimizer.get_huge_pages_allocated().unwrap();
                assert!(allocated > 0, "Some huge pages should be allocated");
                
                // Disable huge pages
                optimizer.set_huge_pages_support(HugePageSupport::Disabled);
                let result = optimizer.apply_settings();
                assert!(result.is_ok(), "Failed to disable huge pages: {:?}", result.err());
                
                // Check that huge pages are disabled
                let status = optimizer.get_huge_pages_status().unwrap();
                assert_eq!(status, HugePageSupport::Disabled, "Huge pages should be disabled");
            }
            
            // Create test result
            TestResult::new(
                "huge_pages",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Huge pages support test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(huge_pages_test);
    
    // Test memory compression
    let sim_fixture = SimulationTestFixture::new("memory_compression_sim");
    let memory_compression_test = UnitTest::new(
        "memory_compression",
        "Test memory compression",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a memory optimizer
            let mut optimizer = MemoryOptimizer::new();
            
            // Set up mock memory
            optimizer.set_mock_total_memory(16 * 1024); // 16 GB
            optimizer.set_mock_available_memory(4 * 1024); // 4 GB available (low memory)
            
            // Test different compression levels
            
            // No compression
            optimizer.set_compression_level(MemoryCompressionLevel::None);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply no compression: {:?}", result.err());
            
            // Check effective memory
            let effective_memory = optimizer.get_effective_memory().unwrap();
            assert_approx_eq!(effective_memory as f64, (4 * 1024) as f64, 10.0, "Effective memory should equal available memory");
            
            // Light compression
            optimizer.set_compression_level(MemoryCompressionLevel::Light);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply light compression: {:?}", result.err());
            
            // Check effective memory (should be slightly higher)
            let light_effective_memory = optimizer.get_effective_memory().unwrap();
            assert!(light_effective_memory > effective_memory, "Light compression should increase effective memory");
            
            // Aggressive compression
            optimizer.set_compression_level(MemoryCompressionLevel::Aggressive);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply aggressive compression: {:?}", result.err());
            
            // Check effective memory (should be significantly higher)
            let aggressive_effective_memory = optimizer.get_effective_memory().unwrap();
            assert!(aggressive_effective_memory > light_effective_memory, "Aggressive compression should further increase effective memory");
            
            // Check compression ratio
            let compression_ratio = optimizer.get_compression_ratio().unwrap();
            assert!(compression_ratio > 1.0, "Compression ratio should be greater than 1.0");
            
            // Create test result
            TestResult::new(
                "memory_compression",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Memory compression test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(memory_compression_test);
}

/// Add storage optimization tests to the test suite
fn add_storage_tests(suite: &mut crate::testing::TestSuite) {
    // Test I/O scheduler optimization
    let sim_fixture = SimulationTestFixture::new("io_scheduler_sim");
    let io_scheduler_test = UnitTest::new(
        "io_scheduler",
        "Test I/O scheduler optimization",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a storage optimizer
            let mut optimizer = StorageOptimizer::new();
            
            // Test different I/O schedulers
            
            // Deadline scheduler
            optimizer.set_io_scheduler(IoScheduler::Deadline);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply deadline scheduler: {:?}", result.err());
            
            // Check that scheduler is set
            let scheduler = optimizer.get_io_scheduler().unwrap();
            assert_eq!(scheduler, IoScheduler::Deadline, "I/O scheduler should be deadline");
            
            // CFQ scheduler
            optimizer.set_io_scheduler(IoScheduler::CFQ);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply CFQ scheduler: {:?}", result.err());
            
            // Check that scheduler is set
            let scheduler = optimizer.get_io_scheduler().unwrap();
            assert_eq!(scheduler, IoScheduler::CFQ, "I/O scheduler should be CFQ");
            
            // NOOP scheduler
            optimizer.set_io_scheduler(IoScheduler::NOOP);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply NOOP scheduler: {:?}", result.err());
            
            // Check that scheduler is set
            let scheduler = optimizer.get_io_scheduler().unwrap();
            assert_eq!(scheduler, IoScheduler::NOOP, "I/O scheduler should be NOOP");
            
            // Test VR workload optimization
            optimizer.optimize_for_vr_workload();
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply VR workload optimization: {:?}", result.err());
            
            // Check that an appropriate scheduler is selected for VR
            let scheduler = optimizer.get_io_scheduler().unwrap();
            assert!(scheduler == IoScheduler::Deadline || scheduler == IoScheduler::CFQ,
                   "VR-optimized scheduler should be deadline or CFQ");
            
            // Create test result
            TestResult::new(
                "io_scheduler",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "I/O scheduler optimization test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(io_scheduler_test);
    
    // Test read-ahead buffer optimization
    let sim_fixture = SimulationTestFixture::new("read_ahead_sim");
    let read_ahead_test = UnitTest::new(
        "read_ahead",
        "Test read-ahead buffer optimization",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a storage optimizer
            let mut optimizer = StorageOptimizer::new();
            
            // Set up mock storage device
            optimizer.set_mock_device("/dev/sda");
            
            // Test different read-ahead buffer sizes
            
            // Small buffer
            optimizer.set_read_ahead_buffer("/dev/sda", ReadAheadBuffer::new(128)); // 128 KB
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply small read-ahead buffer: {:?}", result.err());
            
            // Check that buffer size is set
            let buffer = optimizer.get_read_ahead_buffer("/dev/sda").unwrap();
            assert_eq!(buffer.size_kb(), 128, "Read-ahead buffer should be 128 KB");
            
            // Large buffer
            optimizer.set_read_ahead_buffer("/dev/sda", ReadAheadBuffer::new(4096)); // 4 MB
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply large read-ahead buffer: {:?}", result.err());
            
            // Check that buffer size is set
            let buffer = optimizer.get_read_ahead_buffer("/dev/sda").unwrap();
            assert_eq!(buffer.size_kb(), 4096, "Read-ahead buffer should be 4096 KB");
            
            // Test sequential read performance
            let small_buffer_perf = optimizer.measure_sequential_read_performance(128);
            let large_buffer_perf = optimizer.measure_sequential_read_performance(4096);
            
            // Large buffer should be faster for sequential reads
            assert!(large_buffer_perf > small_buffer_perf, "Large buffer should be faster for sequential reads");
            
            // Test random read performance
            let small_buffer_random_perf = optimizer.measure_random_read_performance(128);
            let large_buffer_random_perf = optimizer.measure_random_read_performance(4096);
            
            // Small buffer might be better for random reads
            assert!(small_buffer_random_perf >= large_buffer_random_perf, "Small buffer should not be worse for random reads");
            
            // Test adaptive buffer sizing
            optimizer.enable_adaptive_read_ahead();
            
            // Simulate sequential workload
            optimizer.set_mock_workload_pattern(0.9); // 90% sequential
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply adaptive read-ahead: {:?}", result.err());
            
            // Check that buffer size is increased for sequential workload
            let buffer = optimizer.get_read_ahead_buffer("/dev/sda").unwrap();
            assert!(buffer.size_kb() > 1024, "Buffer should be large for sequential workload");
            
            // Simulate random workload
            optimizer.set_mock_workload_pattern(0.1); // 10% sequential
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply adaptive read-ahead: {:?}", result.err());
            
            // Check that buffer size is decreased for random workload
            let buffer = optimizer.get_read_ahead_buffer("/dev/sda").unwrap();
            assert!(buffer.size_kb() < 1024, "Buffer should be small for random workload");
            
            // Create test result
            TestResult::new(
                "read_ahead",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Read-ahead buffer optimization test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(read_ahead_test);
    
    // Test cache policy optimization
    let sim_fixture = SimulationTestFixture::new("cache_policy_sim");
    let cache_policy_test = UnitTest::new(
        "cache_policy",
        "Test cache policy optimization",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a storage optimizer
            let mut optimizer = StorageOptimizer::new();
            
            // Set up mock storage device
            optimizer.set_mock_device("/dev/sda");
            
            // Test different cache policies
            
            // Write-through policy
            optimizer.set_cache_policy("/dev/sda", CachePolicy::WriteThrough);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply write-through policy: {:?}", result.err());
            
            // Check that policy is set
            let policy = optimizer.get_cache_policy("/dev/sda").unwrap();
            assert_eq!(policy, CachePolicy::WriteThrough, "Cache policy should be write-through");
            
            // Write-back policy
            optimizer.set_cache_policy("/dev/sda", CachePolicy::WriteBack);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply write-back policy: {:?}", result.err());
            
            // Check that policy is set
            let policy = optimizer.get_cache_policy("/dev/sda").unwrap();
            assert_eq!(policy, CachePolicy::WriteBack, "Cache policy should be write-back");
            
            // Test performance
            let write_through_perf = optimizer.measure_write_performance(CachePolicy::WriteThrough);
            let write_back_perf = optimizer.measure_write_performance(CachePolicy::WriteBack);
            
            // Write-back should be faster
            assert!(write_back_perf > write_through_perf, "Write-back should be faster than write-through");
            
            // Test data safety
            let write_through_safety = optimizer.measure_data_safety(CachePolicy::WriteThrough);
            let write_back_safety = optimizer.measure_data_safety(CachePolicy::WriteBack);
            
            // Write-through should be safer
            assert!(write_through_safety > write_back_safety, "Write-through should be safer than write-back");
            
            // Test VR workload optimization
            optimizer.optimize_for_vr_workload();
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply VR workload optimization: {:?}", result.err());
            
            // Check that an appropriate policy is selected for VR
            let policy = optimizer.get_cache_policy("/dev/sda").unwrap();
            assert_eq!(policy, CachePolicy::WriteBack, "VR-optimized policy should be write-back for performance");
            
            // Create test result
            TestResult::new(
                "cache_policy",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Cache policy optimization test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(cache_policy_test);
}

/// Add network optimization tests to the test suite
fn add_network_tests(suite: &mut crate::testing::TestSuite) {
    // Test QoS policy optimization
    let sim_fixture = SimulationTestFixture::new("qos_policy_sim");
    let qos_policy_test = UnitTest::new(
        "qos_policy",
        "Test QoS policy optimization",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a network optimizer
            let mut optimizer = NetworkOptimizer::new();
            
            // Set up mock network interfaces
            optimizer.set_mock_interface("eth0");
            optimizer.set_mock_interface("wlan0");
            
            // Test different QoS policies
            
            // No QoS
            optimizer.set_qos_policy("eth0", QosPolicy::None);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply no QoS policy: {:?}", result.err());
            
            // Check that policy is set
            let policy = optimizer.get_qos_policy("eth0").unwrap();
            assert_eq!(policy, QosPolicy::None, "QoS policy should be none");
            
            // Priority-based QoS
            optimizer.set_qos_policy("eth0", QosPolicy::Priority);
            
            // Set traffic priorities
            optimizer.set_traffic_priority("VR_DATA", 0); // Highest priority
            optimizer.set_traffic_priority("VOICE", 1);
            optimizer.set_traffic_priority("VIDEO", 2);
            optimizer.set_traffic_priority("BEST_EFFORT", 3);
            
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply priority-based QoS policy: {:?}", result.err());
            
            // Check that policy is set
            let policy = optimizer.get_qos_policy("eth0").unwrap();
            assert_eq!(policy, QosPolicy::Priority, "QoS policy should be priority-based");
            
            // Check traffic priorities
            let vr_priority = optimizer.get_traffic_priority("VR_DATA").unwrap();
            let voice_priority = optimizer.get_traffic_priority("VOICE").unwrap();
            
            assert!(vr_priority < voice_priority, "VR data should have higher priority than voice");
            
            // Bandwidth-based QoS
            optimizer.set_qos_policy("eth0", QosPolicy::Bandwidth);
            
            // Set bandwidth allocations
            optimizer.set_bandwidth_allocation("VR_DATA", 0.5); // 50%
            optimizer.set_bandwidth_allocation("VOICE", 0.2); // 20%
            optimizer.set_bandwidth_allocation("VIDEO", 0.2); // 20%
            optimizer.set_bandwidth_allocation("BEST_EFFORT", 0.1); // 10%
            
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply bandwidth-based QoS policy: {:?}", result.err());
            
            // Check that policy is set
            let policy = optimizer.get_qos_policy("eth0").unwrap();
            assert_eq!(policy, QosPolicy::Bandwidth, "QoS policy should be bandwidth-based");
            
            // Check bandwidth allocations
            let vr_bandwidth = optimizer.get_bandwidth_allocation("VR_DATA").unwrap();
            assert_approx_eq!(vr_bandwidth, 0.5, 0.01, "VR data should have 50% bandwidth allocation");
            
            // Test VR workload optimization
            optimizer.optimize_for_vr_workload();
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply VR workload optimization: {:?}", result.err());
            
            // Check that VR traffic has high priority
            let vr_priority = optimizer.get_traffic_priority("VR_DATA").unwrap();
            assert_eq!(vr_priority, 0, "VR data should have highest priority");
            
            // Create test result
            TestResult::new(
                "qos_policy",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "QoS policy optimization test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(qos_policy_test);
    
    // Test TCP configuration optimization
    let sim_fixture = SimulationTestFixture::new("tcp_config_sim");
    let tcp_config_test = UnitTest::new(
        "tcp_config",
        "Test TCP configuration optimization",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a network optimizer
            let mut optimizer = NetworkOptimizer::new();
            
            // Test different TCP configurations
            
            // Default configuration
            optimizer.set_tcp_configuration(TcpConfiguration::default());
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply default TCP configuration: {:?}", result.err());
            
            // Check that configuration is set
            let config = optimizer.get_tcp_configuration().unwrap();
            assert_eq!(config.congestion_algorithm(), "cubic", "Default congestion algorithm should be cubic");
            
            // Low-latency configuration
            let mut low_latency_config = TcpConfiguration::default();
            low_latency_config.set_congestion_algorithm("bbr");
            low_latency_config.set_initial_congestion_window(10);
            low_latency_config.set_tcp_no_delay(true);
            
            optimizer.set_tcp_configuration(low_latency_config);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply low-latency TCP configuration: {:?}", result.err());
            
            // Check that configuration is set
            let config = optimizer.get_tcp_configuration().unwrap();
            assert_eq!(config.congestion_algorithm(), "bbr", "Congestion algorithm should be bbr");
            assert_eq!(config.initial_congestion_window(), 10, "Initial congestion window should be 10");
            assert_eq!(config.tcp_no_delay(), true, "TCP_NODELAY should be enabled");
            
            // Test latency
            let default_latency = optimizer.measure_tcp_latency(TcpConfiguration::default());
            let low_latency = optimizer.measure_tcp_latency(low_latency_config);
            
            // Low-latency configuration should have lower latency
            assert!(low_latency < default_latency, "Low-latency configuration should have lower latency");
            
            // Test VR workload optimization
            optimizer.optimize_for_vr_workload();
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply VR workload optimization: {:?}", result.err());
            
            // Check that configuration is optimized for VR
            let config = optimizer.get_tcp_configuration().unwrap();
            assert_eq!(config.tcp_no_delay(), true, "TCP_NODELAY should be enabled for VR");
            
            // Create test result
            TestResult::new(
                "tcp_config",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "TCP configuration optimization test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(tcp_config_test);
    
    // Test buffer size optimization
    let sim_fixture = SimulationTestFixture::new("buffer_size_sim");
    let buffer_size_test = UnitTest::new(
        "buffer_size",
        "Test buffer size optimization",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a network optimizer
            let mut optimizer = NetworkOptimizer::new();
            
            // Test different buffer sizes
            
            // Small buffer
            optimizer.set_buffer_size(BufferSize::new(8, 8)); // 8 KB send, 8 KB receive
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply small buffer size: {:?}", result.err());
            
            // Check that buffer size is set
            let buffer = optimizer.get_buffer_size().unwrap();
            assert_eq!(buffer.send_kb(), 8, "Send buffer should be 8 KB");
            assert_eq!(buffer.receive_kb(), 8, "Receive buffer should be 8 KB");
            
            // Large buffer
            optimizer.set_buffer_size(BufferSize::new(256, 256)); // 256 KB send, 256 KB receive
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply large buffer size: {:?}", result.err());
            
            // Check that buffer size is set
            let buffer = optimizer.get_buffer_size().unwrap();
            assert_eq!(buffer.send_kb(), 256, "Send buffer should be 256 KB");
            assert_eq!(buffer.receive_kb(), 256, "Receive buffer should be 256 KB");
            
            // Test throughput
            let small_buffer_throughput = optimizer.measure_throughput(BufferSize::new(8, 8));
            let large_buffer_throughput = optimizer.measure_throughput(BufferSize::new(256, 256));
            
            // Large buffer should have higher throughput
            assert!(large_buffer_throughput > small_buffer_throughput, "Large buffer should have higher throughput");
            
            // Test latency
            let small_buffer_latency = optimizer.measure_latency(BufferSize::new(8, 8));
            let large_buffer_latency = optimizer.measure_latency(BufferSize::new(256, 256));
            
            // Small buffer might have lower latency
            assert!(small_buffer_latency <= large_buffer_latency, "Small buffer should not have higher latency");
            
            // Test auto-tuning
            optimizer.enable_buffer_auto_tuning();
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply buffer auto-tuning: {:?}", result.err());
            
            // Check that auto-tuning is enabled
            assert!(optimizer.is_buffer_auto_tuning_enabled(), "Buffer auto-tuning should be enabled");
            
            // Create test result
            TestResult::new(
                "buffer_size",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Buffer size optimization test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(buffer_size_test);
}

/// Add power optimization tests to the test suite
fn add_power_tests(suite: &mut crate::testing::TestSuite) {
    // Test power profile optimization
    let sim_fixture = SimulationTestFixture::new("power_profile_sim");
    let power_profile_test = UnitTest::new(
        "power_profile",
        "Test power profile optimization",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a power optimizer
            let mut optimizer = PowerOptimizer::new();
            
            // Test different power profiles
            
            // Performance profile
            optimizer.set_power_profile(PowerProfile::Performance);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply performance profile: {:?}", result.err());
            
            // Check that profile is set
            let profile = optimizer.get_power_profile().unwrap();
            assert_eq!(profile, PowerProfile::Performance, "Power profile should be performance");
            
            // Check CPU and GPU settings
            let cpu_governor = optimizer.get_cpu_governor().unwrap();
            let gpu_power_state = optimizer.get_gpu_power_state().unwrap();
            
            assert_eq!(cpu_governor, CpuGovernor::Performance, "CPU governor should be performance");
            assert_eq!(gpu_power_state, GpuPowerState::Performance, "GPU power state should be performance");
            
            // Balanced profile
            optimizer.set_power_profile(PowerProfile::Balanced);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply balanced profile: {:?}", result.err());
            
            // Check that profile is set
            let profile = optimizer.get_power_profile().unwrap();
            assert_eq!(profile, PowerProfile::Balanced, "Power profile should be balanced");
            
            // Check CPU and GPU settings
            let cpu_governor = optimizer.get_cpu_governor().unwrap();
            let gpu_power_state = optimizer.get_gpu_power_state().unwrap();
            
            assert_eq!(cpu_governor, CpuGovernor::OnDemand, "CPU governor should be ondemand");
            assert!(gpu_power_state == GpuPowerState::Balanced || gpu_power_state == GpuPowerState::OnDemand,
                   "GPU power state should be balanced or ondemand");
            
            // Power saving profile
            optimizer.set_power_profile(PowerProfile::PowerSave);
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply power saving profile: {:?}", result.err());
            
            // Check that profile is set
            let profile = optimizer.get_power_profile().unwrap();
            assert_eq!(profile, PowerProfile::PowerSave, "Power profile should be power save");
            
            // Check CPU and GPU settings
            let cpu_governor = optimizer.get_cpu_governor().unwrap();
            let gpu_power_state = optimizer.get_gpu_power_state().unwrap();
            
            assert_eq!(cpu_governor, CpuGovernor::PowerSave, "CPU governor should be powersave");
            assert_eq!(gpu_power_state, GpuPowerState::PowerSave, "GPU power state should be power save");
            
            // Test power consumption
            let performance_power = optimizer.measure_power_consumption(PowerProfile::Performance);
            let balanced_power = optimizer.measure_power_consumption(PowerProfile::Balanced);
            let power_save_power = optimizer.measure_power_consumption(PowerProfile::PowerSave);
            
            // Power consumption should decrease with more conservative profiles
            assert!(performance_power > balanced_power, "Performance profile should consume more power than balanced");
            assert!(balanced_power > power_save_power, "Balanced profile should consume more power than power save");
            
            // Test performance
            let performance_perf = optimizer.measure_performance(PowerProfile::Performance);
            let balanced_perf = optimizer.measure_performance(PowerProfile::Balanced);
            let power_save_perf = optimizer.measure_performance(PowerProfile::PowerSave);
            
            // Performance should decrease with more conservative profiles
            assert!(performance_perf > balanced_perf, "Performance profile should have higher performance than balanced");
            assert!(balanced_perf > power_save_perf, "Balanced profile should have higher performance than power save");
            
            // Create test result
            TestResult::new(
                "power_profile",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Power profile optimization test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(power_profile_test);
    
    // Test thermal throttling
    let sim_fixture = SimulationTestFixture::new("thermal_throttling_sim");
    let thermal_throttling_test = UnitTest::new(
        "thermal_throttling",
        "Test thermal throttling",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a power optimizer
            let mut optimizer = PowerOptimizer::new();
            
            // Set up mock temperature sensors
            optimizer.set_mock_temperature("cpu", 50.0); // 50°C
            optimizer.set_mock_temperature("gpu", 55.0); // 55°C
            
            // Set thermal throttling thresholds
            optimizer.set_thermal_throttling(ThermalThrottling::new(
                70.0, // Warning threshold
                80.0, // Critical threshold
                90.0, // Emergency threshold
            ));
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply thermal throttling settings: {:?}", result.err());
            
            // Check that throttling is set
            let throttling = optimizer.get_thermal_throttling().unwrap();
            assert_eq!(throttling.warning_threshold(), 70.0, "Warning threshold should be 70°C");
            assert_eq!(throttling.critical_threshold(), 80.0, "Critical threshold should be 80°C");
            assert_eq!(throttling.emergency_threshold(), 90.0, "Emergency threshold should be 90°C");
            
            // Check throttling status (should be none at normal temperatures)
            let status = optimizer.get_throttling_status().unwrap();
            assert_eq!(status, PowerState::Normal, "Throttling status should be normal");
            
            // Simulate high CPU temperature
            optimizer.set_mock_temperature("cpu", 75.0); // 75°C (above warning)
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply settings with high CPU temperature: {:?}", result.err());
            
            // Check throttling status
            let status = optimizer.get_throttling_status().unwrap();
            assert_eq!(status, PowerState::Throttled, "Throttling status should be throttled");
            
            // Check CPU frequency (should be reduced)
            let cpu_freq = optimizer.get_cpu_frequency().unwrap();
            assert!(cpu_freq < 100.0, "CPU frequency should be reduced");
            
            // Simulate critical CPU temperature
            optimizer.set_mock_temperature("cpu", 85.0); // 85°C (above critical)
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply settings with critical CPU temperature: {:?}", result.err());
            
            // Check throttling status
            let status = optimizer.get_throttling_status().unwrap();
            assert_eq!(status, PowerState::SeverelyThrottled, "Throttling status should be severely throttled");
            
            // Check CPU frequency (should be significantly reduced)
            let cpu_freq = optimizer.get_cpu_frequency().unwrap();
            assert!(cpu_freq < 50.0, "CPU frequency should be significantly reduced");
            
            // Simulate emergency CPU temperature
            optimizer.set_mock_temperature("cpu", 95.0); // 95°C (above emergency)
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply settings with emergency CPU temperature: {:?}", result.err());
            
            // Check throttling status
            let status = optimizer.get_throttling_status().unwrap();
            assert_eq!(status, PowerState::Emergency, "Throttling status should be emergency");
            
            // Check if emergency measures are taken
            assert!(optimizer.is_emergency_shutdown_requested(), "Emergency shutdown should be requested");
            
            // Create test result
            TestResult::new(
                "thermal_throttling",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Thermal throttling test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(thermal_throttling_test);
    
    // Test peripheral power management
    let sim_fixture = SimulationTestFixture::new("peripheral_power_sim");
    let peripheral_power_test = UnitTest::new(
        "peripheral_power",
        "Test peripheral power management",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a power optimizer
            let mut optimizer = PowerOptimizer::new();
            
            // Set up mock peripherals
            optimizer.add_mock_peripheral("bluetooth", true); // Initially enabled
            optimizer.add_mock_peripheral("wifi", true); // Initially enabled
            optimizer.add_mock_peripheral("camera", true); // Initially enabled
            optimizer.add_mock_peripheral("microphone", true); // Initially enabled
            
            // Check initial power state
            let initial_power = optimizer.measure_peripheral_power();
            
            // Disable unused peripherals
            optimizer.set_peripheral_state("bluetooth", false);
            optimizer.set_peripheral_state("camera", false);
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply peripheral power settings: {:?}", result.err());
            
            // Check peripheral states
            assert!(!optimizer.get_peripheral_state("bluetooth").unwrap(), "Bluetooth should be disabled");
            assert!(!optimizer.get_peripheral_state("camera").unwrap(), "Camera should be disabled");
            assert!(optimizer.get_peripheral_state("wifi").unwrap(), "WiFi should still be enabled");
            assert!(optimizer.get_peripheral_state("microphone").unwrap(), "Microphone should still be enabled");
            
            // Check power consumption
            let reduced_power = optimizer.measure_peripheral_power();
            assert!(reduced_power < initial_power, "Power consumption should be reduced");
            
            // Test auto power management
            optimizer.enable_auto_power_management();
            
            // Simulate inactivity
            optimizer.set_mock_peripheral_activity("wifi", false); // No WiFi activity
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply auto power management: {:?}", result.err());
            
            // Check that inactive peripherals are powered down
            assert!(!optimizer.get_peripheral_state("wifi").unwrap(), "WiFi should be automatically disabled");
            
            // Simulate activity
            optimizer.set_mock_peripheral_activity("wifi", true); // WiFi activity
            
            // Apply settings
            let result = optimizer.apply_settings();
            assert!(result.is_ok(), "Failed to apply auto power management with activity: {:?}", result.err());
            
            // Check that active peripherals are powered up
            assert!(optimizer.get_peripheral_state("wifi").unwrap(), "WiFi should be automatically enabled");
            
            // Create test result
            TestResult::new(
                "peripheral_power",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Peripheral power management test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(peripheral_power_test);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_governor() {
        // Create a CPU optimizer
        let mut optimizer = CpuOptimizer::new();
        
        // Set up mock CPU cores
        optimizer.set_mock_cores(vec![
            CpuCore::new(0, CpuFrequency::new(500, 2000)),
        ]);
        
        // Test performance governor
        optimizer.set_governor(CpuGovernor::Performance);
        let result = optimizer.apply_settings();
        assert!(result.is_ok());
        
        // Check that core is at maximum frequency
        let freq = optimizer.get_core_frequency(0).unwrap();
        assert_eq!(freq.current(), 2000);
    }
    
    #[test]
    fn test_gpu_frequency() {
        // Create a GPU optimizer
        let mut optimizer = GpuOptimizer::new();
        
        // Set up mock GPU
        optimizer.set_mock_gpu_frequency(GpuFrequency::new(200, 1000));
        
        // Test maximum performance
        optimizer.set_performance_level(1.0);
        let result = optimizer.apply_settings();
        assert!(result.is_ok());
        
        // Check that GPU is at maximum frequency
        let freq = optimizer.get_gpu_frequency().unwrap();
        assert_eq!(freq.current(), 1000);
    }
    
    #[test]
    fn test_memory_allocation() {
        // Create a memory optimizer
        let mut optimizer = MemoryOptimizer::new();
        
        // Set up mock memory
        optimizer.set_mock_total_memory(16 * 1024);
        optimizer.set_mock_available_memory(12 * 1024);
        
        // Test conservative strategy
        optimizer.set_allocation_strategy(MemoryAllocationStrategy::Conservative);
        let result = optimizer.apply_settings();
        assert!(result.is_ok());
        
        // Check allocation limits
        let vr_limit = optimizer.get_vr_memory_limit().unwrap();
        assert!(vr_limit < 12 * 1024);
    }
    
    #[test]
    fn test_power_profile() {
        // Create a power optimizer
        let mut optimizer = PowerOptimizer::new();
        
        // Test performance profile
        optimizer.set_power_profile(PowerProfile::Performance);
        let result = optimizer.apply_settings();
        assert!(result.is_ok());
        
        // Check that profile is set
        let profile = optimizer.get_power_profile().unwrap();
        assert_eq!(profile, PowerProfile::Performance);
    }
}

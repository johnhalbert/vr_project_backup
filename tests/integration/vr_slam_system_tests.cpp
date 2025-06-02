#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <memory>
#include <vector>
#include <chrono>
#include <thread>
#include <opencv2/core/core.hpp>

#include "../include/vr_slam_system.hpp"

using namespace ORB_SLAM3;
using namespace testing;
using namespace std::chrono;

class VRSLAMSystemTest : public ::testing::Test {
protected:
    void SetUp() override {
        // Create system configuration
        VRSLAMSystem::Config config;
        config.vocabulary_path = "/path/to/vocabulary.txt";
        config.settings_path = "/path/to/settings.yaml";
        config.calibration_path = "/path/to/calibration.json";
        config.tpu_model_path = "/path/to/model.tflite";
        config.use_imu = true;
        config.enable_mapping = true;
        config.enable_loop_closing = true;
        config.interaction_mode = VRMotionModel::InteractionMode::STANDING;
        config.prediction_horizon_ms = 16.0;
        config.num_threads = 4;
        config.verbose = false;
        
        // Create system
        system_ = std::make_unique<VRSLAMSystem>(config);
    }
    
    void TearDown() override {
        system_.reset();
    }
    
    // Helper method to create test images
    std::vector<cv::Mat> createTestImages(int num_cameras = 4) {
        std::vector<cv::Mat> images;
        
        for (int i = 0; i < num_cameras; ++i) {
            cv::Mat image(480, 640, CV_8UC1);
            cv::randu(image, cv::Scalar(0), cv::Scalar(255));
            images.push_back(image);
        }
        
        return images;
    }
    
    // Helper method to create IMU measurements
    std::pair<Eigen::Vector3f, Eigen::Vector3f> createIMUMeasurement() {
        Eigen::Vector3f gyro(0.1f, 0.2f, 0.3f);
        Eigen::Vector3f accel(0.0f, 0.0f, 9.81f);
        return {gyro, accel};
    }
    
    std::unique_ptr<VRSLAMSystem> system_;
};

// Test system initialization
TEST_F(VRSLAMSystemTest, Initialization) {
    // Initialize system
    bool result = system_->Initialize();
    
    // Verify result (should fail due to missing files)
    EXPECT_FALSE(result);
    
    // Verify status
    EXPECT_EQ(system_->GetStatus(), VRSLAMSystem::Status::UNINITIALIZED);
}

// Test system configuration
TEST_F(VRSLAMSystemTest, Configuration) {
    // Set prediction horizon
    system_->SetPredictionHorizon(20.0);
    
    // Verify prediction horizon
    EXPECT_FLOAT_EQ(system_->GetPredictionHorizon(), 20.0);
    
    // Set interaction mode
    system_->SetInteractionMode(VRMotionModel::InteractionMode::SEATED);
    
    // Verify interaction mode
    EXPECT_EQ(system_->GetInteractionMode(), VRMotionModel::InteractionMode::SEATED);
}

// Test frame processing
TEST_F(VRSLAMSystemTest, FrameProcessing) {
    // Initialize system
    system_->Initialize();
    
    // Create test images
    auto images = createTestImages();
    
    // Process frame
    bool result = system_->ProcessFrame(images, 0.0);
    
    // Verify result (should fail due to uninitialized components)
    EXPECT_FALSE(result);
}

// Test IMU processing
TEST_F(VRSLAMSystemTest, IMUProcessing) {
    // Initialize system
    system_->Initialize();
    
    // Create IMU measurement
    auto [gyro, accel] = createIMUMeasurement();
    
    // Process IMU
    bool result = system_->ProcessIMU(gyro, accel, 0.0);
    
    // Verify result (should fail due to uninitialized components)
    EXPECT_FALSE(result);
}

// Test performance metrics
TEST_F(VRSLAMSystemTest, PerformanceMetrics) {
    // Initialize system
    system_->Initialize();
    
    // Get performance metrics
    auto metrics = system_->GetPerformanceMetrics();
    
    // Verify initial metrics
    EXPECT_FLOAT_EQ(metrics.average_tracking_time_ms, 0.0);
    EXPECT_FLOAT_EQ(metrics.average_feature_extraction_time_ms, 0.0);
    EXPECT_FLOAT_EQ(metrics.average_frame_acquisition_time_ms, 0.0);
    EXPECT_FLOAT_EQ(metrics.average_total_latency_ms, 0.0);
    EXPECT_FLOAT_EQ(metrics.average_fps, 0.0);
    EXPECT_EQ(metrics.frames_processed, 0);
    EXPECT_EQ(metrics.tracking_lost_count, 0);
    EXPECT_FLOAT_EQ(metrics.tracking_percentage, 100.0);
}

// Test system reset
TEST_F(VRSLAMSystemTest, Reset) {
    // Initialize system
    system_->Initialize();
    
    // Reset system
    bool result = system_->Reset();
    
    // Verify result
    EXPECT_TRUE(result);
    
    // Verify status
    EXPECT_EQ(system_->GetStatus(), VRSLAMSystem::Status::INITIALIZING);
}

// Test system shutdown
TEST_F(VRSLAMSystemTest, Shutdown) {
    // Initialize system
    system_->Initialize();
    
    // Shutdown system
    system_->Shutdown();
    
    // Verify status
    EXPECT_EQ(system_->GetStatus(), VRSLAMSystem::Status::SHUTDOWN);
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}

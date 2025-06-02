#include <gtest/gtest.h>
#include <memory>
#include <thread>
#include <chrono>

#include "include/visual_inertial_fusion.hpp"
#include "include/bno085_interface.hpp"
#include "include/multi_camera_tracking.hpp"
#include "include/vr_motion_model.hpp"

namespace ORB_SLAM3 {

// Mock classes for testing
class MockBNO085Interface : public BNO085Interface {
public:
    MockBNO085Interface() : BNO085Interface(Config()) {}
    
    // Override methods to provide test data
    std::vector<IMU::Point> GetMeasurementsInTimeRange(double start_time, double end_time) override {
        std::vector<IMU::Point> measurements;
        
        // Generate synthetic IMU data
        double dt = 0.005; // 200Hz
        for (double t = start_time; t <= end_time; t += dt) {
            // Simple sinusoidal motion pattern
            float ax = 0.1f * sin(t * 2.0);
            float ay = 0.1f * cos(t * 2.0);
            float az = 9.81f; // Gravity
            
            float gx = 0.2f * sin(t * 3.0);
            float gy = 0.2f * cos(t * 3.0);
            float gz = 0.1f * sin(t * 1.5);
            
            measurements.emplace_back(ax, ay, az, gx, gy, gz, t);
        }
        
        return measurements;
    }
    
    IMU::Calib GetCalibration() const override {
        // Create a default calibration
        Sophus::SE3<float> T_bc = Sophus::SE3<float>();
        float ng = 1.7e-4f;  // Gyroscope noise
        float na = 2.0e-3f;  // Accelerometer noise
        float ngw = 1.9e-5f; // Gyroscope random walk
        float naw = 3.0e-3f; // Accelerometer random walk
        
        return IMU::Calib(T_bc, ng, na, ngw, naw);
    }
};

class MockMultiCameraTracking {
public:
    Sophus::SE3<float> GetCurrentPose() const {
        return Sophus::SE3<float>();
    }
};

class VisualInertialFusionTest : public ::testing::Test {
protected:
    void SetUp() override {
        // Create mock objects
        imu_interface = std::make_shared<MockBNO085Interface>();
        tracking = std::make_shared<MockMultiCameraTracking>();
        motion_model = std::make_shared<VRMotionModel>();
        
        // Create configuration
        VisualInertialFusion::Config config;
        config.use_imu = true;
        config.use_multi_camera = true;
        config.imu_frequency = 200.0f;
        config.visual_frequency = 90.0f;
        config.prediction_horizon_ms = 16.0f;
        config.enable_jerk_modeling = true;
        
        // Create fusion object
        fusion = std::make_unique<VisualInertialFusion>(config, imu_interface, tracking, motion_model);
    }
    
    void TearDown() override {
        fusion.reset();
        motion_model.reset();
        tracking.reset();
        imu_interface.reset();
    }
    
    std::shared_ptr<MockBNO085Interface> imu_interface;
    std::shared_ptr<MockMultiCameraTracking> tracking;
    std::shared_ptr<VRMotionModel> motion_model;
    std::unique_ptr<VisualInertialFusion> fusion;
};

TEST_F(VisualInertialFusionTest, Initialization) {
    EXPECT_TRUE(fusion->Initialize());
    EXPECT_EQ(fusion->GetState(), VisualInertialFusion::State::UNINITIALIZED);
    EXPECT_FALSE(fusion->IsInitialized());
}

TEST_F(VisualInertialFusionTest, IMUProcessing) {
    // Create synthetic IMU data
    std::vector<IMU::Point> imu_data;
    double current_time = 0.0;
    
    for (int i = 0; i < 100; i++) {
        float ax = 0.1f * sin(current_time * 2.0);
        float ay = 0.1f * cos(current_time * 2.0);
        float az = 9.81f; // Gravity
        
        float gx = 0.2f * sin(current_time * 3.0);
        float gy = 0.2f * cos(current_time * 3.0);
        float gz = 0.1f * sin(current_time * 1.5);
        
        imu_data.emplace_back(ax, ay, az, gx, gy, gz, current_time);
        current_time += 0.005; // 200Hz
    }
    
    // Process IMU data
    EXPECT_TRUE(fusion->ProcessIMUMeasurements(imu_data));
}

TEST_F(VisualInertialFusionTest, PoseRetrieval) {
    // Get current pose (should be identity before initialization)
    Sophus::SE3<float> pose = fusion->GetCurrentPose();
    EXPECT_TRUE(pose.matrix().isApprox(Eigen::Matrix4f::Identity()));
    
    // Get predicted pose
    Sophus::SE3<float> predicted_pose = fusion->GetPredictedPose(16.0);
    // The prediction should still be close to identity without motion
    EXPECT_TRUE(predicted_pose.matrix().isApprox(Eigen::Matrix4f::Identity(), 1e-3));
}

TEST_F(VisualInertialFusionTest, StateManagement) {
    // Test state transitions
    EXPECT_TRUE(fusion->Initialize());
    EXPECT_EQ(fusion->GetState(), VisualInertialFusion::State::UNINITIALIZED);
    
    // Start fusion
    EXPECT_TRUE(fusion->Start());
    
    // Stop fusion
    fusion->Stop();
    
    // Reset fusion
    EXPECT_TRUE(fusion->Reset());
}

TEST_F(VisualInertialFusionTest, PerformanceMetrics) {
    // Get initial performance metrics
    auto metrics = fusion->GetPerformanceMetrics();
    EXPECT_NEAR(metrics.average_fusion_time_ms, 0.0, 1e-6);
    EXPECT_NEAR(metrics.tracking_percentage, 100.0, 1e-6);
    EXPECT_EQ(metrics.relocalization_count, 0);
}

} // namespace ORB_SLAM3

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}

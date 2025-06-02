#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <memory>
#include <vector>
#include <Eigen/Core>
#include <Eigen/Geometry>
#include <sophus/se3.hpp>

#include "../include/vr_motion_model.hpp"

using namespace ORB_SLAM3;
using namespace testing;

class VRMotionModelTest : public ::testing::Test {
protected:
    void SetUp() override {
        // Create default configuration
        VRMotionModel::PredictionConfig config;
        config.prediction_horizon_ms = 16.0;
        config.max_prediction_ms = 50.0;
        config.use_imu_for_prediction = true;
        config.adaptive_prediction = true;
        config.stationary_threshold = 0.05;
        config.fast_movement_threshold = 0.5;
        config.rotation_only_threshold = 0.1;
        
        // Create motion model with configuration
        motion_model_ = std::make_unique<VRMotionModel>(config);
    }
    
    void TearDown() override {
        motion_model_.reset();
    }
    
    // Helper method to create a pose at a specific position
    Sophus::SE3f createPose(float x, float y, float z, float qw = 1.0f, float qx = 0.0f, float qy = 0.0f, float qz = 0.0f) {
        Eigen::Vector3f translation(x, y, z);
        Eigen::Quaternionf rotation(qw, qx, qy, qz);
        rotation.normalize();
        return Sophus::SE3f(rotation, translation);
    }
    
    // Helper method to create a sequence of poses for linear motion
    void createLinearMotionSequence(double start_time, double time_step, int count) {
        for (int i = 0; i < count; ++i) {
            double t = start_time + i * time_step;
            float x = 0.1f * i;  // Move 10cm per step along X axis
            Sophus::SE3f pose = createPose(x, 0.0f, 0.0f);
            motion_model_->AddPose(pose, t);
        }
    }
    
    // Helper method to create a sequence of poses for rotation
    void createRotationSequence(double start_time, double time_step, int count) {
        for (int i = 0; i < count; ++i) {
            double t = start_time + i * time_step;
            float angle = 0.1f * i;  // Rotate 0.1 radians per step around Y axis
            Eigen::Quaternionf q(Eigen::AngleAxisf(angle, Eigen::Vector3f::UnitY()));
            Sophus::SE3f pose = createPose(0.0f, 0.0f, 0.0f, q.w(), q.x(), q.y(), q.z());
            motion_model_->AddPose(pose, t);
        }
    }
    
    // Helper method to create a sequence of poses for complex motion
    void createComplexMotionSequence(double start_time, double time_step, int count) {
        for (int i = 0; i < count; ++i) {
            double t = start_time + i * time_step;
            float x = 0.05f * i;  // Move 5cm per step along X axis
            float y = 0.02f * std::sin(i * 0.5f);  // Sinusoidal motion along Y
            float z = 0.01f * i;  // Slow movement along Z
            
            // Rotation around Y axis
            float angle = 0.05f * i;
            Eigen::Quaternionf q(Eigen::AngleAxisf(angle, Eigen::Vector3f::UnitY()));
            
            Sophus::SE3f pose = createPose(x, y, z, q.w(), q.x(), q.y(), q.z());
            motion_model_->AddPose(pose, t);
        }
    }
    
    // Helper method to add IMU measurements
    void addIMUMeasurements(double start_time, double time_step, int count, 
                           const Eigen::Vector3f& gyro, const Eigen::Vector3f& accel) {
        for (int i = 0; i < count; ++i) {
            double t = start_time + i * time_step;
            motion_model_->AddIMU(gyro, accel, t);
        }
    }
    
    std::unique_ptr<VRMotionModel> motion_model_;
};

// Test initialization
TEST_F(VRMotionModelTest, Initialization) {
    // Verify default state
    EXPECT_EQ(motion_model_->EstimateHeadsetState(), VRMotionModel::HeadsetState::STATIONARY);
    EXPECT_EQ(motion_model_->EstimateLinearVelocity(), Eigen::Vector3f::Zero());
    EXPECT_EQ(motion_model_->EstimateAngularVelocity(), Eigen::Vector3f::Zero());
    
    // Verify configuration
    VRMotionModel::PredictionConfig config = motion_model_->GetConfig();
    EXPECT_FLOAT_EQ(config.prediction_horizon_ms, 16.0);
    EXPECT_FLOAT_EQ(config.max_prediction_ms, 50.0);
    EXPECT_TRUE(config.use_imu_for_prediction);
    EXPECT_TRUE(config.adaptive_prediction);
}

// Test pose addition and velocity estimation
TEST_F(VRMotionModelTest, PoseAdditionAndVelocityEstimation) {
    // Add two poses with known time difference and displacement
    Sophus::SE3f pose1 = createPose(0.0f, 0.0f, 0.0f);
    Sophus::SE3f pose2 = createPose(0.1f, 0.0f, 0.0f);  // 10cm displacement in X
    
    motion_model_->AddPose(pose1, 0.0);
    motion_model_->AddPose(pose2, 0.1);  // 100ms later
    
    // Expected velocity: 0.1m / 0.1s = 1.0 m/s in X direction
    Eigen::Vector3f expected_velocity(1.0f, 0.0f, 0.0f);
    Eigen::Vector3f actual_velocity = motion_model_->EstimateLinearVelocity();
    
    EXPECT_NEAR(actual_velocity.x(), expected_velocity.x(), 0.01f);
    EXPECT_NEAR(actual_velocity.y(), expected_velocity.y(), 0.01f);
    EXPECT_NEAR(actual_velocity.z(), expected_velocity.z(), 0.01f);
}

// Test constant velocity prediction
TEST_F(VRMotionModelTest, ConstantVelocityPrediction) {
    // Create a sequence of poses with constant velocity
    createLinearMotionSequence(0.0, 0.1, 3);  // 3 poses, 100ms apart, 10cm per step
    
    // Predict 100ms into the future
    Sophus::SE3f predicted_pose = motion_model_->PredictPose(100.0);
    
    // Expected position: last position (0.2) + velocity (1.0 m/s) * time (0.1s) = 0.3
    Eigen::Vector3f expected_position(0.3f, 0.0f, 0.0f);
    Eigen::Vector3f actual_position = predicted_pose.translation();
    
    EXPECT_NEAR(actual_position.x(), expected_position.x(), 0.01f);
    EXPECT_NEAR(actual_position.y(), expected_position.y(), 0.01f);
    EXPECT_NEAR(actual_position.z(), expected_position.z(), 0.01f);
}

// Test constant acceleration prediction
TEST_F(VRMotionModelTest, ConstantAccelerationPrediction) {
    // Create a sequence of poses with increasing velocity
    double time_step = 0.1;
    double start_time = 0.0;
    
    // Position increases quadratically: x = 0.5 * a * t^2
    // With a = 2.0, positions at t=0,0.1,0.2,0.3 are 0, 0.01, 0.04, 0.09
    motion_model_->AddPose(createPose(0.00f, 0.0f, 0.0f), start_time);
    motion_model_->AddPose(createPose(0.01f, 0.0f, 0.0f), start_time + time_step);
    motion_model_->AddPose(createPose(0.04f, 0.0f, 0.0f), start_time + 2 * time_step);
    motion_model_->AddPose(createPose(0.09f, 0.0f, 0.0f), start_time + 3 * time_step);
    
    // Predict 100ms into the future
    Sophus::SE3f predicted_pose = motion_model_->PredictPose(100.0);
    
    // Expected acceleration: 2.0 m/s^2
    // Expected position at t=0.4: 0.5 * 2.0 * 0.4^2 = 0.16
    Eigen::Vector3f expected_position(0.16f, 0.0f, 0.0f);
    Eigen::Vector3f actual_position = predicted_pose.translation();
    
    EXPECT_NEAR(actual_position.x(), expected_position.x(), 0.02f);
    EXPECT_NEAR(actual_position.y(), expected_position.y(), 0.01f);
    EXPECT_NEAR(actual_position.z(), expected_position.z(), 0.01f);
}

// Test jerk-aware prediction
TEST_F(VRMotionModelTest, JerkAwarePrediction) {
    // Create a sequence of poses with changing acceleration
    double time_step = 0.1;
    double start_time = 0.0;
    
    // Position follows cubic function: x = (1/6) * j * t^3
    // With j = 6.0, positions at t=0,0.1,0.2,0.3,0.4 are 0, 0.001, 0.008, 0.027, 0.064
    motion_model_->AddPose(createPose(0.000f, 0.0f, 0.0f), start_time);
    motion_model_->AddPose(createPose(0.001f, 0.0f, 0.0f), start_time + time_step);
    motion_model_->AddPose(createPose(0.008f, 0.0f, 0.0f), start_time + 2 * time_step);
    motion_model_->AddPose(createPose(0.027f, 0.0f, 0.0f), start_time + 3 * time_step);
    motion_model_->AddPose(createPose(0.064f, 0.0f, 0.0f), start_time + 4 * time_step);
    
    // Predict 100ms into the future
    Sophus::SE3f predicted_pose = motion_model_->PredictPose(100.0);
    
    // Expected jerk: 6.0 m/s^3
    // Expected position at t=0.5: (1/6) * 6.0 * 0.5^3 = 0.125
    Eigen::Vector3f expected_position(0.125f, 0.0f, 0.0f);
    Eigen::Vector3f actual_position = predicted_pose.translation();
    
    EXPECT_NEAR(actual_position.x(), expected_position.x(), 0.03f);
    EXPECT_NEAR(actual_position.y(), expected_position.y(), 0.01f);
    EXPECT_NEAR(actual_position.z(), expected_position.z(), 0.01f);
}

// Test rotation prediction
TEST_F(VRMotionModelTest, RotationPrediction) {
    // Create a sequence of poses with rotation around Y axis
    createRotationSequence(0.0, 0.1, 3);  // 3 poses, 100ms apart, 0.1 rad per step
    
    // Predict 100ms into the future
    Sophus::SE3f predicted_pose = motion_model_->PredictPose(100.0);
    
    // Expected rotation: last rotation (0.2 rad) + angular velocity (1.0 rad/s) * time (0.1s) = 0.3 rad
    float expected_angle = 0.3f;
    
    // Extract rotation from predicted pose
    Eigen::AngleAxisf angle_axis(predicted_pose.unit_quaternion());
    float actual_angle = angle_axis.angle();
    Eigen::Vector3f actual_axis = angle_axis.axis();
    
    EXPECT_NEAR(actual_angle, expected_angle, 0.05f);
    EXPECT_NEAR(actual_axis.y(), 1.0f, 0.05f);  // Should rotate around Y axis
}

// Test IMU integration
TEST_F(VRMotionModelTest, IMUIntegration) {
    // Add a pose
    Sophus::SE3f pose = createPose(0.0f, 0.0f, 0.0f);
    motion_model_->AddPose(pose, 0.0);
    
    // Add IMU measurements with constant angular velocity
    Eigen::Vector3f gyro(0.0f, 1.0f, 0.0f);  // 1 rad/s around Y axis
    Eigen::Vector3f accel(0.0f, 0.0f, 0.0f);  // No acceleration
    
    addIMUMeasurements(0.0, 0.01, 10, gyro, accel);  // 10 measurements, 10ms apart
    
    // Predict 100ms into the future
    Sophus::SE3f predicted_pose = motion_model_->PredictPose(100.0);
    
    // Expected rotation: angular velocity (1.0 rad/s) * time (0.1s) = 0.1 rad
    float expected_angle = 0.1f;
    
    // Extract rotation from predicted pose
    Eigen::AngleAxisf angle_axis(predicted_pose.unit_quaternion());
    float actual_angle = angle_axis.angle();
    Eigen::Vector3f actual_axis = angle_axis.axis();
    
    EXPECT_NEAR(actual_angle, expected_angle, 0.05f);
    EXPECT_NEAR(actual_axis.y(), 1.0f, 0.05f);  // Should rotate around Y axis
}

// Test headset state estimation
TEST_F(VRMotionModelTest, HeadsetStateEstimation) {
    // Test stationary state
    {
        motion_model_->Reset();
        motion_model_->AddPose(createPose(0.0f, 0.0f, 0.0f), 0.0);
        motion_model_->AddPose(createPose(0.001f, 0.0f, 0.0f), 0.1);  // Very small movement
        
        EXPECT_EQ(motion_model_->EstimateHeadsetState(), VRMotionModel::HeadsetState::STATIONARY);
    }
    
    // Test slow movement state
    {
        motion_model_->Reset();
        motion_model_->AddPose(createPose(0.0f, 0.0f, 0.0f), 0.0);
        motion_model_->AddPose(createPose(0.02f, 0.0f, 0.0f), 0.1);  // 20cm/s, should be slow movement
        
        EXPECT_EQ(motion_model_->EstimateHeadsetState(), VRMotionModel::HeadsetState::SLOW_MOVEMENT);
    }
    
    // Test fast movement state
    {
        motion_model_->Reset();
        motion_model_->AddPose(createPose(0.0f, 0.0f, 0.0f), 0.0);
        motion_model_->AddPose(createPose(0.1f, 0.0f, 0.0f), 0.1);  // 1m/s, should be fast movement
        
        EXPECT_EQ(motion_model_->EstimateHeadsetState(), VRMotionModel::HeadsetState::FAST_MOVEMENT);
    }
    
    // Test rotation-only state
    {
        motion_model_->Reset();
        Eigen::Quaternionf q(Eigen::AngleAxisf(0.2f, Eigen::Vector3f::UnitY()));
        motion_model_->AddPose(createPose(0.0f, 0.0f, 0.0f), 0.0);
        motion_model_->AddPose(createPose(0.0f, 0.0f, 0.0f, q.w(), q.x(), q.y(), q.z()), 0.1);
        
        EXPECT_EQ(motion_model_->EstimateHeadsetState(), VRMotionModel::HeadsetState::ROTATION_ONLY);
    }
}

// Test interaction mode
TEST_F(VRMotionModelTest, InteractionMode) {
    // Test default mode
    EXPECT_EQ(motion_model_->GetInteractionMode(), VRMotionModel::InteractionMode::STANDING);
    
    // Test setting seated mode
    motion_model_->SetInteractionMode(VRMotionModel::InteractionMode::SEATED);
    EXPECT_EQ(motion_model_->GetInteractionMode(), VRMotionModel::InteractionMode::SEATED);
    
    // Test setting room-scale mode
    motion_model_->SetInteractionMode(VRMotionModel::InteractionMode::ROOM_SCALE);
    EXPECT_EQ(motion_model_->GetInteractionMode(), VRMotionModel::InteractionMode::ROOM_SCALE);
}

// Test Kalman filter prediction
TEST_F(VRMotionModelTest, KalmanFilterPrediction) {
    // Create a sequence of poses with constant velocity
    createLinearMotionSequence(0.0, 0.1, 5);  // 5 poses, 100ms apart, 10cm per step
    
    // Predict 100ms into the future using Kalman filter
    Sophus::SE3f predicted_pose = motion_model_->PredictPoseKalman(100.0);
    
    // Expected position: last position (0.4) + velocity (1.0 m/s) * time (0.1s) = 0.5
    Eigen::Vector3f expected_position(0.5f, 0.0f, 0.0f);
    Eigen::Vector3f actual_position = predicted_pose.translation();
    
    EXPECT_NEAR(actual_position.x(), expected_position.x(), 0.05f);
    EXPECT_NEAR(actual_position.y(), expected_position.y(), 0.01f);
    EXPECT_NEAR(actual_position.z(), expected_position.z(), 0.01f);
}

// Test jerk estimation
TEST_F(VRMotionModelTest, JerkEstimation) {
    // Create a sequence of poses with changing acceleration
    double time_step = 0.1;
    double start_time = 0.0;
    
    // Position follows cubic function: x = (1/6) * j * t^3
    // With j = 6.0, positions at t=0,0.1,0.2,0.3,0.4 are 0, 0.001, 0.008, 0.027, 0.064
    motion_model_->AddPose(createPose(0.000f, 0.0f, 0.0f), start_time);
    motion_model_->AddPose(createPose(0.001f, 0.0f, 0.0f), start_time + time_step);
    motion_model_->AddPose(createPose(0.008f, 0.0f, 0.0f), start_time + 2 * time_step);
    motion_model_->AddPose(createPose(0.027f, 0.0f, 0.0f), start_time + 3 * time_step);
    motion_model_->AddPose(createPose(0.064f, 0.0f, 0.0f), start_time + 4 * time_step);
    
    // Expected jerk: 6.0 m/s^3 in X direction
    Eigen::Vector3f expected_jerk(6.0f, 0.0f, 0.0f);
    Eigen::Vector3f actual_jerk = motion_model_->EstimateLinearJerk();
    
    EXPECT_NEAR(actual_jerk.x(), expected_jerk.x(), 1.0f);  // Allow some error due to numerical differentiation
    EXPECT_NEAR(actual_jerk.y(), expected_jerk.y(), 0.1f);
    EXPECT_NEAR(actual_jerk.z(), expected_jerk.z(), 0.1f);
}

// Test user behavior model
TEST_F(VRMotionModelTest, UserBehaviorModel) {
    // Create a complex motion sequence
    createComplexMotionSequence(0.0, 0.1, 20);  // 20 poses with complex motion
    
    // Get user behavior model
    VRMotionModel::UserBehaviorModel behavior = motion_model_->GetUserBehaviorModel();
    
    // Verify that behavior model has been updated
    EXPECT_GT(behavior.avg_linear_speed, 0.0f);
    EXPECT_GT(behavior.avg_angular_speed, 0.0f);
    
    // Verify that ratios sum to approximately 1.0
    float sum = behavior.stationary_ratio + behavior.rotation_only_ratio + 
                behavior.slow_movement_ratio + behavior.fast_movement_ratio;
    EXPECT_NEAR(sum, 1.0f, 0.01f);
}

// Test latency compensation
TEST_F(VRMotionModelTest, LatencyCompensation) {
    // Set latency compensation
    double latency_ms = 20.0;
    motion_model_->SetLatencyCompensation(latency_ms);
    
    // Verify that latency compensation is set correctly
    EXPECT_FLOAT_EQ(motion_model_->GetLatencyCompensation(), latency_ms);
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}

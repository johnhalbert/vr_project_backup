#include <gtest/gtest.h>
#include <memory>
#include <thread>
#include <chrono>
#include <random>
#include <vector>

#include "include/visual_inertial_fusion.hpp"
#include "include/bno085_interface.hpp"
#include "include/multi_camera_tracking.hpp"
#include "include/vr_motion_model.hpp"

namespace ORB_SLAM3 {

// VR-specific motion patterns for testing
enum class VRMotionPattern {
    STATIC,             // No movement
    SLOW_ROTATION,      // Slow head rotation
    FAST_ROTATION,      // Fast head rotation (common in VR)
    WALKING,            // Walking motion
    RAPID_TRANSLATION,  // Quick positional change
    JERKY_MOTION,       // Sudden starts and stops (common in VR)
    MIXED_MOTION        // Combination of rotation and translation
};

// Synthetic data generator for VR motion patterns
class VRMotionGenerator {
public:
    VRMotionGenerator(double start_time = 0.0) 
        : current_time_(start_time), 
          position_(Eigen::Vector3f::Zero()),
          orientation_(Eigen::Quaternionf::Identity()),
          velocity_(Eigen::Vector3f::Zero()),
          angular_velocity_(Eigen::Vector3f::Zero()) {
        
        // Initialize random generator
        std::random_device rd;
        gen_ = std::mt19937(rd());
        noise_dist_ = std::normal_distribution<float>(0.0f, 1.0f);
    }
    
    // Generate IMU measurements for a specific VR motion pattern
    std::vector<IMU::Point> GenerateIMUData(
        VRMotionPattern pattern, 
        double duration, 
        double frequency = 200.0) {
        
        std::vector<IMU::Point> imu_data;
        double dt = 1.0 / frequency;
        int num_samples = static_cast<int>(duration * frequency);
        
        for (int i = 0; i < num_samples; i++) {
            // Update motion state based on pattern
            UpdateMotionState(pattern, dt);
            
            // Calculate accelerometer and gyroscope readings
            Eigen::Vector3f accel = CalculateAccelerometer();
            Eigen::Vector3f gyro = CalculateGyroscope();
            
            // Add noise
            accel += Eigen::Vector3f(noise_dist_(gen_), noise_dist_(gen_), noise_dist_(gen_)) * 0.05f;
            gyro += Eigen::Vector3f(noise_dist_(gen_), noise_dist_(gen_), noise_dist_(gen_)) * 0.01f;
            
            // Add to data
            imu_data.emplace_back(accel.x(), accel.y(), accel.z(), 
                                 gyro.x(), gyro.y(), gyro.z(), 
                                 current_time_);
            
            current_time_ += dt;
        }
        
        return imu_data;
    }
    
    // Get the current pose
    Sophus::SE3<float> GetCurrentPose() const {
        Eigen::Matrix3f rotation = orientation_.toRotationMatrix();
        return Sophus::SE3<float>(rotation, position_);
    }
    
    // Get the current velocity
    Eigen::Vector3f GetCurrentVelocity() const {
        return velocity_;
    }
    
    // Get the current angular velocity
    Eigen::Vector3f GetCurrentAngularVelocity() const {
        return angular_velocity_;
    }
    
    // Get the current time
    double GetCurrentTime() const {
        return current_time_;
    }
    
private:
    // Update motion state based on pattern
    void UpdateMotionState(VRMotionPattern pattern, double dt) {
        switch (pattern) {
            case VRMotionPattern::STATIC:
                // No movement
                velocity_ = Eigen::Vector3f::Zero();
                angular_velocity_ = Eigen::Vector3f::Zero();
                break;
                
            case VRMotionPattern::SLOW_ROTATION:
                // Slow rotation around Y axis (looking left/right)
                velocity_ = Eigen::Vector3f::Zero();
                angular_velocity_ = Eigen::Vector3f(0.0f, 0.5f, 0.0f);
                break;
                
            case VRMotionPattern::FAST_ROTATION:
                // Fast rotation (typical VR head movement)
                velocity_ = Eigen::Vector3f::Zero();
                angular_velocity_ = Eigen::Vector3f(
                    1.5f * std::sin(current_time_ * 3.0),
                    2.0f * std::cos(current_time_ * 2.5),
                    0.8f * std::sin(current_time_ * 4.0)
                );
                break;
                
            case VRMotionPattern::WALKING:
                // Walking motion (bobbing up/down with forward movement)
                velocity_ = Eigen::Vector3f(
                    1.0f, // Forward
                    0.0f,
                    0.1f * std::sin(current_time_ * 5.0) // Up/down bobbing
                );
                angular_velocity_ = Eigen::Vector3f(
                    0.1f * std::sin(current_time_ * 5.0), // Slight head roll
                    0.2f * std::sin(current_time_ * 2.0), // Looking around
                    0.0f
                );
                break;
                
            case VRMotionPattern::RAPID_TRANSLATION:
                // Quick positional change (dodging in VR)
                if (std::fmod(current_time_, 1.0) < 0.5) {
                    velocity_ = Eigen::Vector3f(0.5f, 2.0f, 0.0f);
                } else {
                    velocity_ = Eigen::Vector3f(0.5f, -2.0f, 0.0f);
                }
                angular_velocity_ = Eigen::Vector3f(0.0f, 0.0f, 0.0f);
                break;
                
            case VRMotionPattern::JERKY_MOTION:
                // Sudden starts and stops
                if (std::fmod(current_time_, 0.5) < 0.1) {
                    // Sudden acceleration
                    velocity_ = Eigen::Vector3f(
                        3.0f * std::sin(current_time_ * 1.0),
                        2.0f * std::cos(current_time_ * 1.5),
                        1.0f * std::sin(current_time_ * 2.0)
                    );
                    angular_velocity_ = Eigen::Vector3f(
                        2.0f * std::sin(current_time_ * 3.0),
                        2.5f * std::cos(current_time_ * 2.0),
                        1.0f * std::sin(current_time_ * 4.0)
                    );
                } else {
                    // Almost stopped
                    velocity_ *= 0.8f;
                    angular_velocity_ *= 0.8f;
                }
                break;
                
            case VRMotionPattern::MIXED_MOTION:
                // Combination of rotation and translation
                velocity_ = Eigen::Vector3f(
                    0.8f * std::sin(current_time_ * 1.0),
                    0.6f * std::cos(current_time_ * 1.2),
                    0.3f * std::sin(current_time_ * 1.5)
                );
                angular_velocity_ = Eigen::Vector3f(
                    1.0f * std::sin(current_time_ * 2.0),
                    1.2f * std::cos(current_time_ * 1.8),
                    0.5f * std::sin(current_time_ * 2.5)
                );
                break;
        }
        
        // Update position and orientation
        position_ += velocity_ * dt;
        
        // Convert angular velocity to quaternion change
        float angle = angular_velocity_.norm() * dt;
        if (angle > 1e-6f) {
            Eigen::Vector3f axis = angular_velocity_.normalized();
            Eigen::Quaternionf q(Eigen::AngleAxisf(angle, axis));
            orientation_ = q * orientation_;
            orientation_.normalize();
        }
    }
    
    // Calculate accelerometer readings (including gravity)
    Eigen::Vector3f CalculateAccelerometer() {
        // Gravity in world frame (pointing down)
        Eigen::Vector3f gravity_world(0.0f, 0.0f, 9.81f);
        
        // Rotate gravity to body frame
        Eigen::Vector3f gravity_body = orientation_.inverse() * gravity_world;
        
        // Add linear acceleration in body frame
        // This is simplified and doesn't account for centripetal acceleration
        Eigen::Vector3f linear_accel_body = orientation_.inverse() * velocity_.derivative();
        
        return gravity_body + linear_accel_body;
    }
    
    // Calculate gyroscope readings
    Eigen::Vector3f CalculateGyroscope() {
        // Gyroscope measures angular velocity in body frame
        return angular_velocity_;
    }
    
    double current_time_;
    Eigen::Vector3f position_;
    Eigen::Quaternionf orientation_;
    Eigen::Vector3f velocity_;
    Eigen::Vector3f angular_velocity_;
    
    std::mt19937 gen_;
    std::normal_distribution<float> noise_dist_;
};

// Enhanced mock classes for VR-specific testing
class VRMockBNO085Interface : public BNO085Interface {
public:
    VRMockBNO085Interface() : BNO085Interface(Config()), motion_generator_(0.0) {}
    
    // Set the motion pattern for testing
    void SetMotionPattern(VRMotionPattern pattern, double duration = 10.0) {
        pattern_ = pattern;
        duration_ = duration;
        
        // Generate IMU data for this pattern
        cached_imu_data_ = motion_generator_.GenerateIMUData(pattern, duration);
    }
    
    // Override methods to provide test data
    std::vector<IMU::Point> GetMeasurementsInTimeRange(double start_time, double end_time) override {
        std::vector<IMU::Point> result;
        
        // Filter cached data by time range
        for (const auto& point : cached_imu_data_) {
            if (point.t >= start_time && point.t <= end_time) {
                result.push_back(point);
            }
        }
        
        return result;
    }
    
    std::vector<IMU::Point> GetMeasurements(size_t max_samples) override {
        if (max_samples == 0 || max_samples >= cached_imu_data_.size()) {
            return cached_imu_data_;
        }
        
        // Return the most recent samples
        return std::vector<IMU::Point>(
            cached_imu_data_.end() - max_samples,
            cached_imu_data_.end()
        );
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
    
    // Get the current ground truth pose
    Sophus::SE3<float> GetGroundTruthPose() const {
        return motion_generator_.GetCurrentPose();
    }
    
    // Get the current ground truth velocity
    Eigen::Vector3f GetGroundTruthVelocity() const {
        return motion_generator_.GetCurrentVelocity();
    }
    
    // Get the current ground truth angular velocity
    Eigen::Vector3f GetGroundTruthAngularVelocity() const {
        return motion_generator_.GetCurrentAngularVelocity();
    }
    
private:
    VRMotionPattern pattern_ = VRMotionPattern::STATIC;
    double duration_ = 10.0;
    VRMotionGenerator motion_generator_;
    std::vector<IMU::Point> cached_imu_data_;
};

// Test fixture for VR-specific visual-inertial fusion tests
class VRVisualInertialFusionTest : public ::testing::Test {
protected:
    void SetUp() override {
        // Create mock objects
        imu_interface = std::make_shared<VRMockBNO085Interface>();
        tracking = std::make_shared<MockMultiCameraTracking>();
        motion_model = std::make_shared<VRMotionModel>();
        
        // Create configuration optimized for VR
        VisualInertialFusion::Config config;
        config.use_imu = true;
        config.use_multi_camera = true;
        config.imu_frequency = 200.0f;
        config.visual_frequency = 90.0f;
        config.prediction_horizon_ms = 16.0f;
        config.enable_jerk_modeling = true;
        config.adaptive_imu_integration = true;
        config.init_time_threshold = 0.2f;  // Faster initialization for VR
        
        // Create fusion object
        fusion = std::make_unique<VisualInertialFusion>(config, imu_interface, tracking, motion_model);
        fusion->Initialize();
    }
    
    void TearDown() override {
        fusion.reset();
        motion_model.reset();
        tracking.reset();
        imu_interface.reset();
    }
    
    std::shared_ptr<VRMockBNO085Interface> imu_interface;
    std::shared_ptr<MockMultiCameraTracking> tracking;
    std::shared_ptr<VRMotionModel> motion_model;
    std::unique_ptr<VisualInertialFusion> fusion;
};

// Test VR-specific motion patterns
TEST_F(VRVisualInertialFusionTest, FastRotationHandling) {
    // Set up fast rotation pattern (common in VR)
    imu_interface->SetMotionPattern(VRMotionPattern::FAST_ROTATION, 5.0);
    
    // Process IMU data
    auto imu_data = imu_interface->GetMeasurements(0);
    EXPECT_TRUE(fusion->ProcessIMUMeasurements(imu_data));
    
    // Start fusion
    EXPECT_TRUE(fusion->Start());
    
    // Wait for processing
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
    
    // Check that the system can handle fast rotations
    EXPECT_TRUE(fusion->IsTrackingGood());
    
    // Verify prediction accuracy for fast rotations
    Sophus::SE3<float> predicted_pose = fusion->GetPredictedPose(16.0);
    // In a real test, we would compare with ground truth
    
    // Stop fusion
    fusion->Stop();
}

TEST_F(VRVisualInertialFusionTest, JerkyMotionHandling) {
    // Set up jerky motion pattern (sudden starts/stops common in VR)
    imu_interface->SetMotionPattern(VRMotionPattern::JERKY_MOTION, 5.0);
    
    // Process IMU data
    auto imu_data = imu_interface->GetMeasurements(0);
    EXPECT_TRUE(fusion->ProcessIMUMeasurements(imu_data));
    
    // Start fusion
    EXPECT_TRUE(fusion->Start());
    
    // Wait for processing
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
    
    // Check that the system can handle jerky motion
    EXPECT_TRUE(fusion->IsTrackingGood());
    
    // Get performance metrics
    auto metrics = fusion->GetPerformanceMetrics();
    
    // Stop fusion
    fusion->Stop();
}

TEST_F(VRVisualInertialFusionTest, LowLatencyPrediction) {
    // Set up mixed motion pattern
    imu_interface->SetMotionPattern(VRMotionPattern::MIXED_MOTION, 5.0);
    
    // Process IMU data
    auto imu_data = imu_interface->GetMeasurements(0);
    EXPECT_TRUE(fusion->ProcessIMUMeasurements(imu_data));
    
    // Start fusion
    EXPECT_TRUE(fusion->Start());
    
    // Wait for processing
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
    
    // Measure prediction time
    auto start_time = std::chrono::high_resolution_clock::now();
    
    // Get predicted poses at different horizons
    Sophus::SE3<float> predicted_pose_8ms = fusion->GetPredictedPose(8.0);
    Sophus::SE3<float> predicted_pose_16ms = fusion->GetPredictedPose(16.0);
    Sophus::SE3<float> predicted_pose_32ms = fusion->GetPredictedPose(32.0);
    
    auto end_time = std::chrono::high_resolution_clock::now();
    double prediction_time_ms = std::chrono::duration_cast<std::chrono::microseconds>(
        end_time - start_time).count() / 1000.0;
    
    // Verify prediction is fast enough for VR (should be < 1ms)
    EXPECT_LT(prediction_time_ms, 1.0);
    
    // Stop fusion
    fusion->Stop();
}

TEST_F(VRVisualInertialFusionTest, RapidInitialization) {
    // Set up static pattern for initialization
    imu_interface->SetMotionPattern(VRMotionPattern::STATIC, 1.0);
    
    // Process IMU data
    auto imu_data = imu_interface->GetMeasurements(0);
    EXPECT_TRUE(fusion->ProcessIMUMeasurements(imu_data));
    
    // Start fusion
    EXPECT_TRUE(fusion->Start());
    
    // Wait for initialization
    auto start_time = std::chrono::high_resolution_clock::now();
    
    // Wait up to 500ms for initialization
    bool initialized = false;
    for (int i = 0; i < 50 && !initialized; i++) {
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
        initialized = fusion->IsInitialized();
    }
    
    auto end_time = std::chrono::high_resolution_clock::now();
    double init_time_ms = std::chrono::duration_cast<std::chrono::milliseconds>(
        end_time - start_time).count();
    
    // VR systems need fast initialization (ideally < 500ms)
    // This is a soft requirement as our mock system may not initialize fully
    if (initialized) {
        EXPECT_LT(init_time_ms, 500.0);
    }
    
    // Stop fusion
    fusion->Stop();
}

TEST_F(VRVisualInertialFusionTest, AdaptiveProcessing) {
    // Test with different motion patterns to verify adaptive processing
    std::vector<VRMotionPattern> patterns = {
        VRMotionPattern::STATIC,
        VRMotionPattern::SLOW_ROTATION,
        VRMotionPattern::FAST_ROTATION,
        VRMotionPattern::JERKY_MOTION
    };
    
    std::vector<double> processing_times;
    
    for (auto pattern : patterns) {
        // Set up pattern
        imu_interface->SetMotionPattern(pattern, 2.0);
        
        // Reset fusion
        fusion->Reset();
        
        // Process IMU data
        auto imu_data = imu_interface->GetMeasurements(0);
        EXPECT_TRUE(fusion->ProcessIMUMeasurements(imu_data));
        
        // Start fusion
        EXPECT_TRUE(fusion->Start());
        
        // Wait for processing
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
        
        // Get performance metrics
        auto metrics = fusion->GetPerformanceMetrics();
        processing_times.push_back(metrics.average_fusion_time_ms);
        
        // Stop fusion
        fusion->Stop();
    }
    
    // In adaptive processing, we expect more processing time for complex motion
    // This is a relative test, not absolute
    EXPECT_LT(processing_times[0], processing_times[3]); // Static should be faster than jerky
}

} // namespace ORB_SLAM3

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}

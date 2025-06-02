#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <memory>
#include <vector>
#include <random>
#include <opencv2/core/core.hpp>
#include <opencv2/imgproc/imgproc.hpp>

#include "../include/vr_motion_model.hpp"
#include "../include/multi_camera_rig.hpp"
#include "../include/multi_camera_tracking.hpp"

using namespace ORB_SLAM3;
using namespace testing;

class SyntheticDataGenerator {
public:
    // Generate synthetic camera trajectory
    static std::vector<Sophus::SE3f> GenerateCameraTrajectory(
        int num_frames, 
        const Sophus::SE3f& initial_pose = Sophus::SE3f(),
        TrajectoryType type = TrajectoryType::CIRCLE) {
        
        std::vector<Sophus::SE3f> trajectory;
        trajectory.reserve(num_frames);
        
        Sophus::SE3f current_pose = initial_pose;
        trajectory.push_back(current_pose);
        
        switch (type) {
            case TrajectoryType::CIRCLE:
                generateCircleTrajectory(trajectory, num_frames, initial_pose);
                break;
                
            case TrajectoryType::STRAIGHT_LINE:
                generateStraightLineTrajectory(trajectory, num_frames, initial_pose);
                break;
                
            case TrajectoryType::RANDOM_WALK:
                generateRandomWalkTrajectory(trajectory, num_frames, initial_pose);
                break;
                
            case TrajectoryType::VR_HEAD_MOVEMENT:
                generateVRHeadMovementTrajectory(trajectory, num_frames, initial_pose);
                break;
        }
        
        return trajectory;
    }
    
    // Generate synthetic IMU measurements for a trajectory
    static std::vector<IMUMeasurement> GenerateIMUMeasurements(
        const std::vector<Sophus::SE3f>& trajectory, 
        double frame_time_step,
        int imu_rate = 100) {
        
        std::vector<IMUMeasurement> measurements;
        
        // IMU measurements are typically at a higher rate than camera frames
        double imu_time_step = 1.0 / imu_rate;
        int imu_steps_per_frame = static_cast<int>(frame_time_step / imu_time_step);
        
        for (size_t i = 1; i < trajectory.size(); ++i) {
            const Sophus::SE3f& prev_pose = trajectory[i-1];
            const Sophus::SE3f& curr_pose = trajectory[i];
            
            // Calculate velocity and acceleration
            Eigen::Vector3f position_diff = curr_pose.translation() - prev_pose.translation();
            Eigen::Vector3f velocity = position_diff / frame_time_step;
            
            // Calculate rotation difference
            Eigen::Quaternionf q_prev = prev_pose.unit_quaternion();
            Eigen::Quaternionf q_curr = curr_pose.unit_quaternion();
            Eigen::Quaternionf q_diff = q_curr * q_prev.inverse();
            
            // Convert to axis-angle representation
            Eigen::AngleAxisf angle_axis(q_diff);
            Eigen::Vector3f angular_velocity = angle_axis.axis() * angle_axis.angle() / frame_time_step;
            
            // Generate IMU measurements between frames
            for (int j = 0; j < imu_steps_per_frame; ++j) {
                double t = j * imu_time_step / frame_time_step;  // Interpolation factor
                
                // Interpolate position and orientation
                Eigen::Vector3f interp_position = prev_pose.translation() + position_diff * t;
                Eigen::Quaternionf interp_orientation = q_prev.slerp(t, q_curr);
                
                // Create gravity vector in world frame (assuming Z is up)
                Eigen::Vector3f gravity_world(0, 0, 9.81);
                
                // Transform gravity to body frame
                Eigen::Vector3f gravity_body = interp_orientation.inverse() * gravity_world;
                
                // Add acceleration due to motion
                Eigen::Vector3f accel = -gravity_body;  // Remove gravity
                
                // Add noise to measurements
                std::normal_distribution<float> accel_noise(0.0f, 0.05f);
                std::normal_distribution<float> gyro_noise(0.0f, 0.01f);
                
                for (int k = 0; k < 3; ++k) {
                    accel[k] += accel_noise(random_generator_);
                    angular_velocity[k] += gyro_noise(random_generator_);
                }
                
                // Create IMU measurement
                IMUMeasurement measurement;
                measurement.timestamp = (i-1) * frame_time_step + j * imu_time_step;
                measurement.accelerometer = accel;
                measurement.gyroscope = angular_velocity;
                
                measurements.push_back(measurement);
            }
        }
        
        return measurements;
    }
    
    // Generate synthetic images for a multi-camera rig along a trajectory
    static std::vector<std::vector<cv::Mat>> GenerateSyntheticImages(
        const MultiCameraRig& rig,
        const std::vector<Sophus::SE3f>& trajectory,
        int num_features = 1000,
        int patch_size = 11) {
        
        std::vector<std::vector<cv::Mat>> all_camera_images;
        
        // Generate random 3D points in the world
        std::vector<cv::Point3f> world_points = generateRandomWorldPoints(num_features, 10.0f);
        
        // For each pose in the trajectory
        for (const auto& pose : trajectory) {
            std::vector<cv::Mat> camera_images;
            
            // For each camera in the rig
            for (const auto& camera : rig.GetAllCameras()) {
                // Create empty image
                cv::Mat image = cv::Mat::zeros(camera.height, camera.width, CV_8UC1);
                
                // Calculate camera pose in world coordinates
                Sophus::SE3f camera_pose = pose * rig.GetCameraPose(camera.id);
                
                // Project world points to camera
                for (const auto& world_point : world_points) {
                    // Transform point to camera coordinates
                    Eigen::Vector3f point_world(world_point.x, world_point.y, world_point.z);
                    Eigen::Vector3f point_camera = camera_pose.inverse() * point_world;
                    
                    // Skip points behind the camera
                    if (point_camera.z() <= 0) continue;
                    
                    // Project to image plane
                    float u = camera.K.at<float>(0,0) * point_camera.x() / point_camera.z() + camera.K.at<float>(0,2);
                    float v = camera.K.at<float>(1,1) * point_camera.y() / point_camera.z() + camera.K.at<float>(1,2);
                    
                    // Check if point is within image bounds
                    if (u >= 0 && u < camera.width && v >= 0 && v < camera.height) {
                        // Draw feature patch
                        cv::Point center(static_cast<int>(u), static_cast<int>(v));
                        cv::circle(image, center, patch_size/2, cv::Scalar(255), -1);
                    }
                }
                
                camera_images.push_back(image);
            }
            
            all_camera_images.push_back(camera_images);
        }
        
        return all_camera_images;
    }
    
    enum class TrajectoryType {
        CIRCLE,
        STRAIGHT_LINE,
        RANDOM_WALK,
        VR_HEAD_MOVEMENT
    };
    
    struct IMUMeasurement {
        double timestamp;
        Eigen::Vector3f accelerometer;
        Eigen::Vector3f gyroscope;
    };
    
private:
    static std::default_random_engine random_generator_;
    
    static void generateCircleTrajectory(
        std::vector<Sophus::SE3f>& trajectory, 
        int num_frames, 
        const Sophus::SE3f& initial_pose) {
        
        float radius = 2.0f;
        float angular_step = 2.0f * M_PI / num_frames;
        
        for (int i = 1; i < num_frames; ++i) {
            float angle = i * angular_step;
            
            // Position on circle
            Eigen::Vector3f position(
                radius * std::cos(angle),
                radius * std::sin(angle),
                0.0f
            );
            
            // Orientation (looking at center)
            Eigen::Vector3f look_dir = -position.normalized();
            Eigen::Vector3f up(0.0f, 0.0f, 1.0f);
            Eigen::Vector3f right = up.cross(look_dir).normalized();
            up = look_dir.cross(right).normalized();
            
            Eigen::Matrix3f rotation;
            rotation.col(0) = right;
            rotation.col(1) = up;
            rotation.col(2) = look_dir;
            
            Eigen::Quaternionf q(rotation);
            
            // Create pose
            Sophus::SE3f pose(q, position);
            trajectory.push_back(pose);
        }
    }
    
    static void generateStraightLineTrajectory(
        std::vector<Sophus::SE3f>& trajectory, 
        int num_frames, 
        const Sophus::SE3f& initial_pose) {
        
        Eigen::Vector3f direction(1.0f, 0.0f, 0.0f);
        float step_size = 0.1f;
        
        Eigen::Vector3f initial_position = initial_pose.translation();
        Eigen::Quaternionf initial_orientation = initial_pose.unit_quaternion();
        
        for (int i = 1; i < num_frames; ++i) {
            Eigen::Vector3f position = initial_position + direction * step_size * i;
            Sophus::SE3f pose(initial_orientation, position);
            trajectory.push_back(pose);
        }
    }
    
    static void generateRandomWalkTrajectory(
        std::vector<Sophus::SE3f>& trajectory, 
        int num_frames, 
        const Sophus::SE3f& initial_pose) {
        
        std::normal_distribution<float> position_noise(0.0f, 0.05f);
        std::normal_distribution<float> orientation_noise(0.0f, 0.01f);
        
        Sophus::SE3f current_pose = initial_pose;
        
        for (int i = 1; i < num_frames; ++i) {
            // Add random displacement
            Eigen::Vector3f position = current_pose.translation();
            position.x() += position_noise(random_generator_);
            position.y() += position_noise(random_generator_);
            position.z() += position_noise(random_generator_);
            
            // Add random rotation
            Eigen::Quaternionf orientation = current_pose.unit_quaternion();
            Eigen::Vector3f rotation_vector(
                orientation_noise(random_generator_),
                orientation_noise(random_generator_),
                orientation_noise(random_generator_)
            );
            
            float angle = rotation_vector.norm();
            if (angle > 1e-6) {
                Eigen::AngleAxisf aa(angle, rotation_vector.normalized());
                Eigen::Quaternionf delta_q(aa);
                orientation = delta_q * orientation;
                orientation.normalize();
            }
            
            // Create new pose
            current_pose = Sophus::SE3f(orientation, position);
            trajectory.push_back(current_pose);
        }
    }
    
    static void generateVRHeadMovementTrajectory(
        std::vector<Sophus::SE3f>& trajectory, 
        int num_frames, 
        const Sophus::SE3f& initial_pose) {
        
        // Parameters for VR head movement simulation
        float translation_amplitude = 0.2f;  // 20cm movement range
        float rotation_amplitude = 0.5f;     // ~30 degrees rotation range
        
        // Frequencies for different movement components
        float fast_freq = 0.1f;  // Fast head rotation
        float slow_freq = 0.02f; // Slow body movement
        
        Eigen::Vector3f initial_position = initial_pose.translation();
        Eigen::Quaternionf initial_orientation = initial_pose.unit_quaternion();
        
        for (int i = 1; i < num_frames; ++i) {
            float t = static_cast<float>(i) / num_frames;
            
            // Simulate typical VR head movement patterns
            
            // Slow translation (body movement)
            Eigen::Vector3f position = initial_position;
            position.x() += translation_amplitude * std::sin(2.0f * M_PI * slow_freq * i);
            position.y() += translation_amplitude * 0.5f * std::sin(2.0f * M_PI * slow_freq * 0.7f * i + 0.5f);
            
            // Fast rotation (head looking around)
            Eigen::Vector3f rotation_vector(
                rotation_amplitude * 0.5f * std::sin(2.0f * M_PI * fast_freq * 1.1f * i + 0.2f),
                rotation_amplitude * std::sin(2.0f * M_PI * fast_freq * i),
                rotation_amplitude * 0.3f * std::sin(2.0f * M_PI * fast_freq * 0.9f * i + 0.7f)
            );
            
            Eigen::AngleAxisf aa(rotation_vector.norm(), 
                                rotation_vector.norm() > 1e-6 ? rotation_vector.normalized() : Eigen::Vector3f::UnitX());
            Eigen::Quaternionf orientation = aa * initial_orientation;
            orientation.normalize();
            
            // Create new pose
            Sophus::SE3f pose(orientation, position);
            trajectory.push_back(pose);
        }
    }
    
    static std::vector<cv::Point3f> generateRandomWorldPoints(int num_points, float max_distance) {
        std::vector<cv::Point3f> points;
        points.reserve(num_points);
        
        std::uniform_real_distribution<float> dist(-max_distance, max_distance);
        
        for (int i = 0; i < num_points; ++i) {
            cv::Point3f point(
                dist(random_generator_),
                dist(random_generator_),
                dist(random_generator_)
            );
            
            // Ensure points are not too close to origin
            if (std::sqrt(point.x*point.x + point.y*point.y + point.z*point.z) < 1.0f) {
                point.x += (point.x > 0) ? 1.0f : -1.0f;
                point.y += (point.y > 0) ? 1.0f : -1.0f;
                point.z += (point.z > 0) ? 1.0f : -1.0f;
            }
            
            points.push_back(point);
        }
        
        return points;
    }
};

std::default_random_engine SyntheticDataGenerator::random_generator_(42);  // Fixed seed for reproducibility

class VRMotionModelSimulationTest : public ::testing::Test {
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
    
    // Helper method to run a simulation with a synthetic trajectory
    void runSimulation(const std::vector<Sophus::SE3f>& trajectory, double frame_time_step) {
        // Generate IMU measurements
        auto imu_measurements = SyntheticDataGenerator::GenerateIMUMeasurements(
            trajectory, frame_time_step);
        
        // Reset motion model
        motion_model_->Reset();
        
        // Process trajectory and IMU data
        for (size_t i = 0; i < trajectory.size(); ++i) {
            double timestamp = i * frame_time_step;
            motion_model_->AddPose(trajectory[i], timestamp);
            
            // Add IMU measurements up to this timestamp
            while (!imu_measurements.empty() && imu_measurements.front().timestamp <= timestamp) {
                auto& imu = imu_measurements.front();
                motion_model_->AddIMU(imu.gyroscope, imu.accelerometer, imu.timestamp);
                imu_measurements.erase(imu_measurements.begin());
            }
        }
    }
    
    // Helper method to evaluate prediction accuracy
    double evaluatePredictionAccuracy(
        const std::vector<Sophus::SE3f>& trajectory, 
        double frame_time_step,
        double prediction_time_ms) {
        
        double total_error = 0.0;
        int prediction_count = 0;
        
        // Reset motion model
        motion_model_->Reset();
        
        // Process trajectory
        for (size_t i = 0; i < trajectory.size() - 1; ++i) {
            double timestamp = i * frame_time_step;
            motion_model_->AddPose(trajectory[i], timestamp);
            
            // Skip first few frames to allow motion model to initialize
            if (i < 5) continue;
            
            // Predict future pose
            Sophus::SE3f predicted_pose = motion_model_->PredictPose(prediction_time_ms);
            
            // Calculate actual future pose index
            double future_time = timestamp + prediction_time_ms / 1000.0;
            size_t future_frame = static_cast<size_t>(future_time / frame_time_step);
            
            // Skip if future frame is beyond trajectory
            if (future_frame >= trajectory.size()) continue;
            
            // Calculate position error
            Eigen::Vector3f predicted_position = predicted_pose.translation();
            Eigen::Vector3f actual_position = trajectory[future_frame].translation();
            float position_error = (predicted_position - actual_position).norm();
            
            // Calculate orientation error
            Eigen::Quaternionf predicted_orientation = predicted_pose.unit_quaternion();
            Eigen::Quaternionf actual_orientation = trajectory[future_frame].unit_quaternion();
            Eigen::Quaternionf orientation_diff = predicted_orientation.inverse() * actual_orientation;
            Eigen::AngleAxisf angle_axis(orientation_diff);
            float orientation_error = angle_axis.angle();
            
            // Combine errors (weighted sum)
            float combined_error = position_error + 0.1f * orientation_error;
            total_error += combined_error;
            prediction_count++;
        }
        
        return prediction_count > 0 ? total_error / prediction_count : 0.0;
    }
    
    std::unique_ptr<VRMotionModel> motion_model_;
};

// Test VR head movement prediction
TEST_F(VRMotionModelSimulationTest, VRHeadMovementPrediction) {
    // Generate VR head movement trajectory
    int num_frames = 100;
    double frame_time_step = 1.0 / 30.0;  // 30 fps
    
    auto trajectory = SyntheticDataGenerator::GenerateCameraTrajectory(
        num_frames, Sophus::SE3f(), SyntheticDataGenerator::TrajectoryType::VR_HEAD_MOVEMENT);
    
    // Run simulation
    runSimulation(trajectory, frame_time_step);
    
    // Evaluate prediction accuracy for different prediction horizons
    double error_10ms = evaluatePredictionAccuracy(trajectory, frame_time_step, 10.0);
    double error_20ms = evaluatePredictionAccuracy(trajectory, frame_time_step, 20.0);
    double error_30ms = evaluatePredictionAccuracy(trajectory, frame_time_step, 30.0);
    
    // Verify that prediction error increases with prediction horizon
    EXPECT_LT(error_10ms, error_20ms);
    EXPECT_LT(error_20ms, error_30ms);
    
    // Verify that prediction error is within reasonable bounds
    EXPECT_LT(error_10ms, 0.05);  // Less than 5cm error for 10ms prediction
}

// Test different prediction methods
TEST_F(VRMotionModelSimulationTest, PredictionMethodComparison) {
    // Generate trajectory
    int num_frames = 100;
    double frame_time_step = 1.0 / 30.0;  // 30 fps
    
    auto trajectory = SyntheticDataGenerator::GenerateCameraTrajectory(
        num_frames, Sophus::SE3f(), SyntheticDataGenerator::TrajectoryType::CIRCLE);
    
    // Generate IMU measurements
    auto imu_measurements = SyntheticDataGenerator::GenerateIMUMeasurements(
        trajectory, frame_time_step);
    
    // Reset motion model
    motion_model_->Reset();
    
    // Process first 50 frames
    for (int i = 0; i < 50; ++i) {
        double timestamp = i * frame_time_step;
        motion_model_->AddPose(trajectory[i], timestamp);
        
        // Add IMU measurements up to this timestamp
        while (!imu_measurements.empty() && imu_measurements.front().timestamp <= timestamp) {
            auto& imu = imu_measurements.front();
            motion_model_->AddIMU(imu.gyroscope, imu.accelerometer, imu.timestamp);
            imu_measurements.erase(imu_measurements.begin());
        }
    }
    
    // Get predictions from different methods
    double prediction_time_ms = 20.0;
    Sophus::SE3f kalman_prediction = motion_model_->PredictPoseKalman(prediction_time_ms);
    
    // Temporarily disable IMU for prediction
    VRMotionModel::PredictionConfig config = motion_model_->GetConfig();
    config.use_imu_for_prediction = false;
    motion_model_->SetConfig(config);
    
    Sophus::SE3f standard_prediction = motion_model_->PredictPose(prediction_time_ms);
    
    // Calculate actual future pose
    double future_time = 50 * frame_time_step + prediction_time_ms / 1000.0;
    size_t future_frame = static_cast<size_t>(future_time / frame_time_step);
    Sophus::SE3f actual_pose = trajectory[future_frame];
    
    // Calculate errors
    float kalman_error = (kalman_prediction.translation() - actual_pose.translation()).norm();
    float standard_error = (standard_prediction.translation() - actual_pose.translation()).norm();
    
    // Verify that Kalman filter prediction is more accurate
    EXPECT_LT(kalman_error, standard_error);
}

// Test interaction mode adaptation
TEST_F(VRMotionModelSimulationTest, InteractionModeAdaptation) {
    // Generate trajectories for different interaction modes
    int num_frames = 100;
    double frame_time_step = 1.0 / 30.0;  // 30 fps
    
    // Seated mode: small movements
    auto seated_trajectory = SyntheticDataGenerator::GenerateCameraTrajectory(
        num_frames, Sophus::SE3f(), SyntheticDataGenerator::TrajectoryType::RANDOM_WALK);
    
    // Room-scale mode: larger movements
    auto room_scale_trajectory = SyntheticDataGenerator::GenerateCameraTrajectory(
        num_frames, Sophus::SE3f(), SyntheticDataGenerator::TrajectoryType::CIRCLE);
    
    // Test seated mode
    motion_model_->Reset();
    motion_model_->SetInteractionMode(VRMotionModel::InteractionMode::SEATED);
    runSimulation(seated_trajectory, frame_time_step);
    
    // Verify that seated mode has appropriate thresholds
    VRMotionModel::PredictionConfig seated_config = motion_model_->GetConfig();
    EXPECT_LT(seated_config.stationary_threshold, 0.05);
    
    // Test room-scale mode
    motion_model_->Reset();
    motion_model_->SetInteractionMode(VRMotionModel::InteractionMode::ROOM_SCALE);
    runSimulation(room_scale_trajectory, frame_time_step);
    
    // Verify that room-scale mode has appropriate thresholds
    VRMotionModel::PredictionConfig room_config = motion_model_->GetConfig();
    EXPECT_GT(room_config.stationary_threshold, seated_config.stationary_threshold);
    EXPECT_GT(room_config.fast_movement_threshold, seated_config.fast_movement_threshold);
}

class MultiCameraTrackingSimulationTest : public ::testing::Test {
protected:
    void SetUp() override {
        // Create a multi-camera rig
        setupMultiCameraRig();
    }
    
    void TearDown() override {
    }
    
    void setupMultiCameraRig() {
        // Create a 4-camera rig for VR headset
        rig_ = std::make_unique<MultiCameraRig>(0);  // Reference camera ID = 0
        
        // Front camera (reference)
        MultiCameraRig::CameraInfo frontCamera;
        frontCamera.id = 0;
        frontCamera.K = (cv::Mat_<float>(3, 3) << 
            500.0f, 0.0f, 320.0f,
            0.0f, 500.0f, 240.0f,
            0.0f, 0.0f, 1.0f);
        frontCamera.distCoef = cv::Mat::zeros(1, 5, CV_32F);
        frontCamera.T_ref_cam = cv::Mat::eye(4, 4, CV_32F);
        frontCamera.fps = 30.0f;
        frontCamera.width = 640;
        frontCamera.height = 480;
        frontCamera.model = "pinhole";
        frontCamera.fov_horizontal = 90.0f;
        frontCamera.fov_vertical = 70.0f;
        rig_->AddCamera(frontCamera);
        
        // Right camera
        MultiCameraRig::CameraInfo rightCamera;
        rightCamera.id = 1;
        rightCamera.K = frontCamera.K.clone();
        rightCamera.distCoef = frontCamera.distCoef.clone();
        rightCamera.T_ref_cam = (cv::Mat_<float>(4, 4) << 
            0.0f, 0.0f, 1.0f, 0.1f,
            0.0f, 1.0f, 0.0f, 0.0f,
            -1.0f, 0.0f, 0.0f, 0.0f,
            0.0f, 0.0f, 0.0f, 1.0f);
        rightCamera.fps = 30.0f;
        rightCamera.width = 640;
        rightCamera.height = 480;
        rightCamera.model = "pinhole";
        rightCamera.fov_horizontal = 90.0f;
        rightCamera.fov_vertical = 70.0f;
        rig_->AddCamera(rightCamera);
        
        // Back camera
        MultiCameraRig::CameraInfo backCamera;
        backCamera.id = 2;
        backCamera.K = frontCamera.K.clone();
        backCamera.distCoef = frontCamera.distCoef.clone();
        backCamera.T_ref_cam = (cv::Mat_<float>(4, 4) << 
            -1.0f, 0.0f, 0.0f, 0.0f,
            0.0f, 1.0f, 0.0f, 0.0f,
            0.0f, 0.0f, -1.0f, -0.1f,
            0.0f, 0.0f, 0.0f, 1.0f);
        backCamera.fps = 30.0f;
        backCamera.width = 640;
        backCamera.height = 480;
        backCamera.model = "pinhole";
        backCamera.fov_horizontal = 90.0f;
        backCamera.fov_vertical = 70.0f;
        rig_->AddCamera(backCamera);
        
        // Left camera
        MultiCameraRig::CameraInfo leftCamera;
        leftCamera.id = 3;
        leftCamera.K = frontCamera.K.clone();
        leftCamera.distCoef = frontCamera.distCoef.clone();
        leftCamera.T_ref_cam = (cv::Mat_<float>(4, 4) << 
            0.0f, 0.0f, -1.0f, -0.1f,
            0.0f, 1.0f, 0.0f, 0.0f,
            1.0f, 0.0f, 0.0f, 0.0f,
            0.0f, 0.0f, 0.0f, 1.0f);
        leftCamera.fps = 30.0f;
        leftCamera.width = 640;
        leftCamera.height = 480;
        leftCamera.model = "pinhole";
        leftCamera.fov_horizontal = 90.0f;
        leftCamera.fov_vertical = 70.0f;
        rig_->AddCamera(leftCamera);
    }
    
    // Helper method to test feature visibility across cameras
    void testFeatureVisibilityAcrossCameras() {
        // Create 3D points around the rig
        std::vector<cv::Point3f> test_points = {
            cv::Point3f(0.0f, 0.0f, 1.0f),    // In front
            cv::Point3f(1.0f, 0.0f, 0.0f),    // To the right
            cv::Point3f(0.0f, 0.0f, -1.0f),   // Behind
            cv::Point3f(-1.0f, 0.0f, 0.0f),   // To the left
            cv::Point3f(0.0f, 1.0f, 0.0f),    // Above
            cv::Point3f(0.0f, -1.0f, 0.0f)    // Below
        };
        
        // For each point, check which cameras can see it
        for (const auto& point : test_points) {
            std::vector<int> visible_cameras;
            
            for (const auto& camera : rig_->GetAllCameras()) {
                // Transform point to camera coordinates
                cv::Mat point_mat = (cv::Mat_<float>(4, 1) << point.x, point.y, point.z, 1.0f);
                cv::Mat camera_transform = rig_->GetTransform(0, camera.id);
                cv::Mat point_camera = camera_transform * point_mat;
                
                // Check if point is in front of camera
                if (point_camera.at<float>(2) > 0) {
                    // Project to image plane
                    float u = camera.K.at<float>(0,0) * point_camera.at<float>(0) / point_camera.at<float>(2) + camera.K.at<float>(0,2);
                    float v = camera.K.at<float>(1,1) * point_camera.at<float>(1) / point_camera.at<float>(2) + camera.K.at<float>(1,2);
                    
                    // Check if point is within image bounds
                    if (u >= 0 && u < camera.width && v >= 0 && v < camera.height) {
                        visible_cameras.push_back(camera.id);
                    }
                }
            }
            
            // Print results
            std::cout << "Point " << point << " is visible in cameras: ";
            for (int id : visible_cameras) {
                std::cout << id << " ";
            }
            std::cout << std::endl;
            
            // Verify that each point is visible in at least one camera
            EXPECT_FALSE(visible_cameras.empty());
        }
    }
    
    // Helper method to test camera handoff
    void testCameraHandoff() {
        // Generate a circular trajectory that goes around the rig
        int num_frames = 100;
        auto trajectory = SyntheticDataGenerator::GenerateCameraTrajectory(
            num_frames, Sophus::SE3f(), SyntheticDataGenerator::TrajectoryType::CIRCLE);
        
        // Create a fixed 3D point that will be visible throughout the trajectory
        cv::Point3f fixed_point(0.0f, 0.0f, 0.0f);  // At the center of the circle
        
        // For each pose in the trajectory
        for (const auto& pose : trajectory) {
            // Find which cameras can see the fixed point
            std::vector<int> visible_cameras;
            
            for (const auto& camera : rig_->GetAllCameras()) {
                // Calculate camera pose in world coordinates
                Sophus::SE3f camera_pose = pose * rig_->GetCameraPose(camera.id);
                
                // Transform point to camera coordinates
                Eigen::Vector3f point_world(fixed_point.x, fixed_point.y, fixed_point.z);
                Eigen::Vector3f point_camera = camera_pose.inverse() * point_world;
                
                // Check if point is in front of camera
                if (point_camera.z() > 0) {
                    // Project to image plane
                    float u = camera.K.at<float>(0,0) * point_camera.x() / point_camera.z() + camera.K.at<float>(0,2);
                    float v = camera.K.at<float>(1,1) * point_camera.y() / point_camera.z() + camera.K.at<float>(1,2);
                    
                    // Check if point is within image bounds
                    if (u >= 0 && u < camera.width && v >= 0 && v < camera.height) {
                        visible_cameras.push_back(camera.id);
                    }
                }
            }
            
            // Verify that the point is visible in at least one camera at all times
            EXPECT_FALSE(visible_cameras.empty());
        }
    }
    
    std::unique_ptr<MultiCameraRig> rig_;
};

// Test feature visibility across cameras
TEST_F(MultiCameraTrackingSimulationTest, FeatureVisibilityAcrossCameras) {
    testFeatureVisibilityAcrossCameras();
}

// Test camera handoff
TEST_F(MultiCameraTrackingSimulationTest, CameraHandoff) {
    testCameraHandoff();
}

// Test synthetic image generation
TEST_F(MultiCameraTrackingSimulationTest, SyntheticImageGeneration) {
    // Generate a simple trajectory
    int num_frames = 10;
    auto trajectory = SyntheticDataGenerator::GenerateCameraTrajectory(
        num_frames, Sophus::SE3f(), SyntheticDataGenerator::TrajectoryType::STRAIGHT_LINE);
    
    // Generate synthetic images
    auto images = SyntheticDataGenerator::GenerateSyntheticImages(*rig_, trajectory);
    
    // Verify image dimensions and count
    EXPECT_EQ(images.size(), num_frames);
    EXPECT_EQ(images[0].size(), 4);  // 4 cameras
    EXPECT_EQ(images[0][0].rows, 480);
    EXPECT_EQ(images[0][0].cols, 640);
    
    // Verify that images contain features (non-zero pixels)
    for (const auto& frame_images : images) {
        for (const auto& image : frame_images) {
            cv::Scalar sum = cv::sum(image);
            EXPECT_GT(sum[0], 0);
        }
    }
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}

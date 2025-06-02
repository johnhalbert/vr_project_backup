#include <vector>
#include <random>
#include <chrono>
#include <opencv2/core.hpp>
#include <opencv2/imgproc.hpp>
#include <opencv2/highgui.hpp>
#include <Eigen/Core>
#include <Eigen/Geometry>

#include "../../ORB_SLAM3/include/ImuTypes.h"

namespace ORB_SLAM3 {
namespace Testing {

/**
 * @brief Class for generating synthetic data for SLAM testing
 * 
 * This class provides methods for generating synthetic images, IMU data,
 * and camera trajectories for testing SLAM components without physical hardware.
 */
class SyntheticDataGenerator {
public:
    /**
     * @brief Constructor
     * 
     * @param seed Random seed for reproducibility
     */
    SyntheticDataGenerator(unsigned int seed = std::chrono::system_clock::now().time_since_epoch().count())
        : rng_(seed)
    {
    }
    
    /**
     * @brief Generate a synthetic image with known features
     * 
     * @param width Image width
     * @param height Image height
     * @param num_features Number of features to generate
     * @param feature_size Size of features
     * @param noise_level Noise level (0.0-1.0)
     * @return Generated image
     */
    cv::Mat GenerateSyntheticImage(
        int width = 640,
        int height = 480,
        int num_features = 100,
        int feature_size = 10,
        float noise_level = 0.1)
    {
        // Create empty image
        cv::Mat image = cv::Mat::zeros(height, width, CV_8UC1);
        
        // Generate random features
        std::uniform_int_distribution<int> dist_x(feature_size, width - feature_size);
        std::uniform_int_distribution<int> dist_y(feature_size, height - feature_size);
        std::uniform_int_distribution<int> dist_intensity(100, 255);
        
        for (int i = 0; i < num_features; i++) {
            int x = dist_x(rng_);
            int y = dist_y(rng_);
            int intensity = dist_intensity(rng_);
            
            // Draw feature (simple circle)
            cv::circle(image, cv::Point(x, y), feature_size, cv::Scalar(intensity), -1);
        }
        
        // Add noise
        if (noise_level > 0) {
            cv::Mat noise(height, width, CV_8UC1);
            std::normal_distribution<float> dist_noise(0, noise_level * 255);
            
            for (int y = 0; y < height; y++) {
                for (int x = 0; x < width; x++) {
                    noise.at<uchar>(y, x) = cv::saturate_cast<uchar>(dist_noise(rng_));
                }
            }
            
            cv::addWeighted(image, 1.0, noise, 1.0, 0.0, image);
        }
        
        return image;
    }
    
    /**
     * @brief Generate synthetic IMU measurements
     * 
     * @param duration_sec Duration in seconds
     * @param sample_rate_hz Sample rate in Hz
     * @param accel_noise Accelerometer noise level
     * @param gyro_noise Gyroscope noise level
     * @param motion_pattern Motion pattern to simulate
     * @return Vector of IMU measurements
     */
    std::vector<IMU::Point> GenerateSyntheticIMUData(
        double duration_sec = 10.0,
        double sample_rate_hz = 100.0,
        float accel_noise = 0.01,
        float gyro_noise = 0.001,
        const std::string& motion_pattern = "random")
    {
        std::vector<IMU::Point> measurements;
        
        // Calculate number of samples
        int num_samples = static_cast<int>(duration_sec * sample_rate_hz);
        double dt = 1.0 / sample_rate_hz;
        
        // Initialize state
        Eigen::Vector3f position = Eigen::Vector3f::Zero();
        Eigen::Vector3f velocity = Eigen::Vector3f::Zero();
        Eigen::Quaternionf orientation = Eigen::Quaternionf::Identity();
        Eigen::Vector3f angular_velocity = Eigen::Vector3f::Zero();
        
        // Noise distributions
        std::normal_distribution<float> accel_noise_dist(0, accel_noise);
        std::normal_distribution<float> gyro_noise_dist(0, gyro_noise);
        
        // Motion pattern parameters
        float motion_amplitude = 1.0f;
        float motion_frequency = 0.5f;
        
        // Generate measurements
        for (int i = 0; i < num_samples; i++) {
            double timestamp = i * dt;
            
            // Update state based on motion pattern
            if (motion_pattern == "random") {
                // Random motion
                std::normal_distribution<float> accel_dist(0, 0.1);
                std::normal_distribution<float> gyro_dist(0, 0.01);
                
                Eigen::Vector3f acceleration(accel_dist(rng_), accel_dist(rng_), 9.81f + accel_dist(rng_));
                angular_velocity = Eigen::Vector3f(gyro_dist(rng_), gyro_dist(rng_), gyro_dist(rng_));
                
                // Update position and velocity
                position += velocity * dt + 0.5f * acceleration * dt * dt;
                velocity += acceleration * dt;
                
                // Update orientation
                float angle = angular_velocity.norm() * dt;
                if (angle > 0) {
                    Eigen::Quaternionf dq(Eigen::AngleAxisf(angle, angular_velocity.normalized()));
                    orientation = orientation * dq;
                    orientation.normalize();
                }
            }
            else if (motion_pattern == "circle") {
                // Circular motion
                float t = timestamp;
                float radius = motion_amplitude;
                float omega = motion_frequency * 2 * M_PI;
                
                position.x() = radius * std::cos(omega * t);
                position.y() = radius * std::sin(omega * t);
                position.z() = 0;
                
                velocity.x() = -radius * omega * std::sin(omega * t);
                velocity.y() = radius * omega * std::cos(omega * t);
                velocity.z() = 0;
                
                Eigen::Vector3f acceleration;
                acceleration.x() = -radius * omega * omega * std::cos(omega * t);
                acceleration.y() = -radius * omega * omega * std::sin(omega * t);
                acceleration.z() = 0;
                
                angular_velocity.x() = 0;
                angular_velocity.y() = 0;
                angular_velocity.z() = omega;
                
                // Update orientation
                float angle = angular_velocity.norm() * dt;
                if (angle > 0) {
                    Eigen::Quaternionf dq(Eigen::AngleAxisf(angle, angular_velocity.normalized()));
                    orientation = orientation * dq;
                    orientation.normalize();
                }
            }
            else if (motion_pattern == "walking") {
                // Walking motion (simplified)
                float t = timestamp;
                float step_frequency = 2.0f; // Hz
                float step_amplitude = 0.05f; // m
                
                // Forward motion with slight up/down oscillation
                position.x() += 1.0f * dt; // 1 m/s forward
                position.z() = step_amplitude * std::sin(2 * M_PI * step_frequency * t);
                
                velocity.x() = 1.0f;
                velocity.z() = step_amplitude * 2 * M_PI * step_frequency * std::cos(2 * M_PI * step_frequency * t);
                
                Eigen::Vector3f acceleration;
                acceleration.x() = 0;
                acceleration.z() = -step_amplitude * 4 * M_PI * M_PI * step_frequency * step_frequency * std::sin(2 * M_PI * step_frequency * t);
                acceleration.y() = 0;
                
                // Add gravity
                acceleration.z() += 9.81f;
                
                // Slight rotation around y-axis (heading changes)
                angular_velocity.y() = 0.1f * std::sin(0.5f * t);
                
                // Update orientation
                float angle = angular_velocity.norm() * dt;
                if (angle > 0) {
                    Eigen::Quaternionf dq(Eigen::AngleAxisf(angle, angular_velocity.normalized()));
                    orientation = orientation * dq;
                    orientation.normalize();
                }
            }
            
            // Transform acceleration to body frame
            Eigen::Vector3f gravity(0, 0, 9.81f);
            Eigen::Vector3f acceleration_body = orientation.inverse() * (velocity.cross(angular_velocity) + gravity);
            
            // Add noise
            Eigen::Vector3f accel_with_noise = acceleration_body + Eigen::Vector3f(
                accel_noise_dist(rng_),
                accel_noise_dist(rng_),
                accel_noise_dist(rng_)
            );
            
            Eigen::Vector3f gyro_with_noise = angular_velocity + Eigen::Vector3f(
                gyro_noise_dist(rng_),
                gyro_noise_dist(rng_),
                gyro_noise_dist(rng_)
            );
            
            // Create IMU measurement
            IMU::Point measurement(
                accel_with_noise.x(), accel_with_noise.y(), accel_with_noise.z(),
                gyro_with_noise.x(), gyro_with_noise.y(), gyro_with_noise.z(),
                timestamp
            );
            
            measurements.push_back(measurement);
        }
        
        return measurements;
    }
    
    /**
     * @brief Generate a synthetic camera trajectory
     * 
     * @param duration_sec Duration in seconds
     * @param sample_rate_hz Sample rate in Hz
     * @param motion_pattern Motion pattern to simulate
     * @return Vector of camera poses (position, orientation)
     */
    std::vector<std::pair<Eigen::Vector3f, Eigen::Quaternionf>> GenerateSyntheticCameraTrajectory(
        double duration_sec = 10.0,
        double sample_rate_hz = 30.0,
        const std::string& motion_pattern = "random")
    {
        std::vector<std::pair<Eigen::Vector3f, Eigen::Quaternionf>> trajectory;
        
        // Calculate number of samples
        int num_samples = static_cast<int>(duration_sec * sample_rate_hz);
        double dt = 1.0 / sample_rate_hz;
        
        // Initialize state
        Eigen::Vector3f position = Eigen::Vector3f::Zero();
        Eigen::Vector3f velocity = Eigen::Vector3f::Zero();
        Eigen::Quaternionf orientation = Eigen::Quaternionf::Identity();
        Eigen::Vector3f angular_velocity = Eigen::Vector3f::Zero();
        
        // Motion pattern parameters
        float motion_amplitude = 1.0f;
        float motion_frequency = 0.5f;
        
        // Generate trajectory
        for (int i = 0; i < num_samples; i++) {
            double timestamp = i * dt;
            
            // Update state based on motion pattern
            if (motion_pattern == "random") {
                // Random motion
                std::normal_distribution<float> vel_dist(0, 0.1);
                std::normal_distribution<float> ang_vel_dist(0, 0.01);
                
                velocity = Eigen::Vector3f(vel_dist(rng_), vel_dist(rng_), vel_dist(rng_));
                angular_velocity = Eigen::Vector3f(ang_vel_dist(rng_), ang_vel_dist(rng_), ang_vel_dist(rng_));
                
                // Update position
                position += velocity * dt;
                
                // Update orientation
                float angle = angular_velocity.norm() * dt;
                if (angle > 0) {
                    Eigen::Quaternionf dq(Eigen::AngleAxisf(angle, angular_velocity.normalized()));
                    orientation = orientation * dq;
                    orientation.normalize();
                }
            }
            else if (motion_pattern == "circle") {
                // Circular motion
                float t = timestamp;
                float radius = motion_amplitude;
                float omega = motion_frequency * 2 * M_PI;
                
                position.x() = radius * std::cos(omega * t);
                position.y() = radius * std::sin(omega * t);
                position.z() = 0;
                
                // Always look at the center
                Eigen::Vector3f look_at = Eigen::Vector3f::Zero();
                Eigen::Vector3f up = Eigen::Vector3f::UnitZ();
                
                Eigen::Vector3f z = (look_at - position).normalized();
                Eigen::Vector3f x = up.cross(z).normalized();
                Eigen::Vector3f y = z.cross(x);
                
                Eigen::Matrix3f rotation;
                rotation.col(0) = x;
                rotation.col(1) = y;
                rotation.col(2) = z;
                
                orientation = Eigen::Quaternionf(rotation);
            }
            else if (motion_pattern == "forward") {
                // Forward motion
                position.z() += 1.0f * dt; // 1 m/s forward
                
                // Slight random rotation
                std::normal_distribution<float> ang_vel_dist(0, 0.01);
                angular_velocity = Eigen::Vector3f(ang_vel_dist(rng_), ang_vel_dist(rng_), ang_vel_dist(rng_));
                
                // Update orientation
                float angle = angular_velocity.norm() * dt;
                if (angle > 0) {
                    Eigen::Quaternionf dq(Eigen::AngleAxisf(angle, angular_velocity.normalized()));
                    orientation = orientation * dq;
                    orientation.normalize();
                }
            }
            
            // Add to trajectory
            trajectory.push_back(std::make_pair(position, orientation));
        }
        
        return trajectory;
    }
    
    /**
     * @brief Generate a synthetic multi-camera trajectory
     * 
     * @param num_cameras Number of cameras
     * @param duration_sec Duration in seconds
     * @param sample_rate_hz Sample rate in Hz
     * @param motion_pattern Motion pattern to simulate
     * @return Vector of camera poses for each camera
     */
    std::vector<std::vector<std::pair<Eigen::Vector3f, Eigen::Quaternionf>>> GenerateSyntheticMultiCameraTrajectory(
        int num_cameras = 4,
        double duration_sec = 10.0,
        double sample_rate_hz = 30.0,
        const std::string& motion_pattern = "random")
    {
        // Generate reference camera trajectory
        auto reference_trajectory = GenerateSyntheticCameraTrajectory(
            duration_sec, sample_rate_hz, motion_pattern);
        
        // Create multi-camera trajectory
        std::vector<std::vector<std::pair<Eigen::Vector3f, Eigen::Quaternionf>>> multi_camera_trajectory(num_cameras);
        
        // Define relative poses for each camera
        std::vector<std::pair<Eigen::Vector3f, Eigen::Quaternionf>> relative_poses(num_cameras);
        
        // Set up a typical VR headset multi-camera configuration
        if (num_cameras == 4) {
            // Front camera (reference)
            relative_poses[0] = std::make_pair(
                Eigen::Vector3f(0, 0, 0),
                Eigen::Quaternionf::Identity()
            );
            
            // Left camera
            relative_poses[1] = std::make_pair(
                Eigen::Vector3f(-0.05f, 0, 0),
                Eigen::Quaternionf(Eigen::AngleAxisf(-M_PI/4, Eigen::Vector3f::UnitY()))
            );
            
            // Right camera
            relative_poses[2] = std::make_pair(
                Eigen::Vector3f(0.05f, 0, 0),
                Eigen::Quaternionf(Eigen::AngleAxisf(M_PI/4, Eigen::Vector3f::UnitY()))
            );
            
            // Back camera
            relative_poses[3] = std::make_pair(
                Eigen::Vector3f(0, 0, -0.05f),
                Eigen::Quaternionf(Eigen::AngleAxisf(M_PI, Eigen::Vector3f::UnitY()))
            );
        }
        else {
            // Generic configuration
            for (int i = 0; i < num_cameras; i++) {
                float angle = 2 * M_PI * i / num_cameras;
                relative_poses[i] = std::make_pair(
                    Eigen::Vector3f(0.05f * std::cos(angle), 0, 0.05f * std::sin(angle)),
                    Eigen::Quaternionf(Eigen::AngleAxisf(angle, Eigen::Vector3f::UnitY()))
                );
            }
        }
        
        // Transform reference trajectory to each camera
        for (int i = 0; i < num_cameras; i++) {
            const auto& [rel_pos, rel_ori] = relative_poses[i];
            
            for (const auto& [ref_pos, ref_ori] : reference_trajectory) {
                // Transform position and orientation
                Eigen::Vector3f pos = ref_pos + ref_ori * rel_pos;
                Eigen::Quaternionf ori = ref_ori * rel_ori;
                
                multi_camera_trajectory[i].push_back(std::make_pair(pos, ori));
            }
        }
        
        return multi_camera_trajectory;
    }
    
    /**
     * @brief Generate ground truth data for evaluation
     * 
     * @param duration_sec Duration in seconds
     * @param sample_rate_hz Sample rate in Hz
     * @return Ground truth data (timestamp, position, orientation)
     */
    std::vector<std::tuple<double, Eigen::Vector3f, Eigen::Quaternionf>> GenerateGroundTruthData(
        double duration_sec = 10.0,
        double sample_rate_hz = 100.0)
    {
        std::vector<std::tuple<double, Eigen::Vector3f, Eigen::Quaternionf>> ground_truth;
        
        // Calculate number of samples
        int num_samples = static_cast<int>(duration_sec * sample_rate_hz);
        double dt = 1.0 / sample_rate_hz;
        
        // Generate ground truth data
        for (int i = 0; i < num_samples; i++) {
            double timestamp = i * dt;
            
            // Generate position and orientation
            float t = timestamp;
            Eigen::Vector3f position(
                std::sin(t),
                std::cos(t),
                0.1f * std::sin(2 * t)
            );
            
            Eigen::Quaternionf orientation(
                Eigen::AngleAxisf(0.1f * std::sin(t), Eigen::Vector3f::UnitX()) *
                Eigen::AngleAxisf(0.2f * std::cos(t), Eigen::Vector3f::UnitY()) *
                Eigen::AngleAxisf(0.3f * std::sin(2 * t), Eigen::Vector3f::UnitZ())
            );
            
            ground_truth.push_back(std::make_tuple(timestamp, position, orientation));
        }
        
        return ground_truth;
    }
    
private:
    std::mt19937 rng_;
};

} // namespace Testing
} // namespace ORB_SLAM3

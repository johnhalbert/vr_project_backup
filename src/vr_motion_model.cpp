#include "include/vr_motion_model.hpp"
#include <algorithm>
#include <cmath>
#include <iostream>
#include <Eigen/Dense>

namespace ORB_SLAM3
{

VRMotionModel::VRMotionModel() : current_state_(HeadsetState::STATIONARY), latency_compensation_ms_(0.0)
{
    // Default configuration
    config_.prediction_horizon_ms = 16.0;  // ~60Hz refresh rate
    config_.max_prediction_ms = 50.0;      // Maximum reasonable prediction
    config_.use_imu_for_prediction = true;
    config_.adaptive_prediction = true;
    config_.stationary_threshold = 0.05;   // 5cm/s
    config_.fast_movement_threshold = 0.5; // 50cm/s
    config_.rotation_only_threshold = 0.1; // Ratio of translation to rotation
    
    // Initialize state variables
    linear_velocity_ = Eigen::Vector3f::Zero();
    angular_velocity_ = Eigen::Vector3f::Zero();
    linear_acceleration_ = Eigen::Vector3f::Zero();
    angular_acceleration_ = Eigen::Vector3f::Zero();
    linear_jerk_ = Eigen::Vector3f::Zero();
    angular_jerk_ = Eigen::Vector3f::Zero();
    
    // Initialize Kalman filter state
    initializeKalmanFilter();
}

VRMotionModel::VRMotionModel(const PredictionConfig& config) 
    : config_(config), current_state_(HeadsetState::STATIONARY), latency_compensation_ms_(0.0)
{
    // Initialize state variables
    linear_velocity_ = Eigen::Vector3f::Zero();
    angular_velocity_ = Eigen::Vector3f::Zero();
    linear_acceleration_ = Eigen::Vector3f::Zero();
    angular_acceleration_ = Eigen::Vector3f::Zero();
    linear_jerk_ = Eigen::Vector3f::Zero();
    angular_jerk_ = Eigen::Vector3f::Zero();
    
    // Initialize Kalman filter state
    initializeKalmanFilter();
}

VRMotionModel::~VRMotionModel()
{
}

void VRMotionModel::SetConfig(const PredictionConfig& config)
{
    config_ = config;
}

VRMotionModel::PredictionConfig VRMotionModel::GetConfig() const
{
    return config_;
}

void VRMotionModel::AddPose(const Sophus::SE3f& pose, double timestamp)
{
    // Add new pose to history
    PoseRecord record;
    record.pose = pose;
    record.timestamp = timestamp;
    pose_history_.push_front(record);
    
    // Prune history to maintain maximum size
    PruneHistory();
    
    // Update state estimates
    UpdateState();
    
    // Update Kalman filter with new measurement
    if (pose_history_.size() >= 2) {
        updateKalmanFilter(pose, timestamp);
    }
}

void VRMotionModel::AddIMU(const Eigen::Vector3f& gyro, const Eigen::Vector3f& accel, double timestamp)
{
    // Add new IMU measurement to history
    IMURecord record;
    record.gyro = gyro;
    record.accel = accel;
    record.timestamp = timestamp;
    imu_history_.push_front(record);
    
    // Prune history to maintain maximum size
    while (imu_history_.size() > MAX_HISTORY_SIZE) {
        imu_history_.pop_back();
    }
    
    // If we have IMU data but no pose data yet, use IMU for initial velocity estimates
    if (!pose_history_.empty() && imu_history_.size() >= 2) {
        // Use gyro directly for angular velocity if no poses available
        if (pose_history_.size() < 2) {
            angular_velocity_ = gyro;
        }
        
        // Update Kalman filter with IMU measurement
        updateKalmanFilterWithIMU(gyro, accel, timestamp);
    }
}

Sophus::SE3f VRMotionModel::PredictPose(double prediction_time_ms)
{
    // Ensure we have at least one pose
    if (pose_history_.empty()) {
        std::cerr << "Cannot predict pose: No pose history available" << std::endl;
        return Sophus::SE3f();
    }
    
    // Clamp prediction time to maximum
    double clamped_prediction_ms = std::min(prediction_time_ms, config_.max_prediction_ms);
    
    // Adjust prediction time based on headset state if adaptive prediction is enabled
    if (config_.adaptive_prediction) {
        switch (current_state_) {
            case HeadsetState::STATIONARY:
                // Minimal prediction for stationary headset
                clamped_prediction_ms = std::min(clamped_prediction_ms, 5.0);
                break;
            case HeadsetState::SLOW_MOVEMENT:
                // Standard prediction for slow movement
                break;
            case HeadsetState::FAST_MOVEMENT:
                // Reduced prediction for fast movement to avoid overshooting
                clamped_prediction_ms = std::min(clamped_prediction_ms, config_.prediction_horizon_ms * 0.8);
                break;
            case HeadsetState::ROTATION_ONLY:
                // Focused on rotational prediction for rotation-only movement
                break;
        }
    }
    
    // Choose prediction method based on available data and configuration
    if (config_.use_imu_for_prediction && !imu_history_.empty()) {
        return PredictWithIMU(clamped_prediction_ms);
    } else if (pose_history_.size() >= 4) {
        // Use jerk-aware prediction if we have enough history
        return PredictWithJerk(clamped_prediction_ms);
    } else if (pose_history_.size() >= 3) {
        return PredictWithConstantAcceleration(clamped_prediction_ms);
    } else if (pose_history_.size() >= 2) {
        return PredictWithConstantVelocity(clamped_prediction_ms);
    } else {
        // If only one pose is available, return it (no prediction)
        return pose_history_.front().pose;
    }
}

Sophus::SE3f VRMotionModel::PredictPoseKalman(double prediction_time_ms)
{
    // Ensure we have at least one pose
    if (pose_history_.empty()) {
        std::cerr << "Cannot predict pose: No pose history available" << std::endl;
        return Sophus::SE3f();
    }
    
    // Get most recent pose
    const Sophus::SE3f& current_pose = pose_history_.front().pose;
    
    // Convert prediction time to seconds
    double prediction_time_s = prediction_time_ms / 1000.0;
    
    // Predict using Kalman filter
    Eigen::VectorXf state_prediction = predictKalmanState(prediction_time_s);
    
    // Extract position and orientation from state vector
    Eigen::Vector3f predicted_translation(
        state_prediction(0),
        state_prediction(1),
        state_prediction(2)
    );
    
    // For rotation, we use the angular velocity from the state vector
    Eigen::Vector3f angular_velocity(
        state_prediction(9),
        state_prediction(10),
        state_prediction(11)
    );
    
    // Predict rotation using angular velocity
    Eigen::Vector3f rotation_vector = angular_velocity * prediction_time_s;
    Eigen::AngleAxisf rotation(rotation_vector.norm(), 
                              rotation_vector.norm() > 1e-6 ? rotation_vector.normalized() : Eigen::Vector3f::UnitX());
    Eigen::Quaternionf predicted_rotation = rotation * current_pose.unit_quaternion();
    
    // Combine into predicted pose
    return Sophus::SE3f(predicted_rotation, predicted_translation);
}

VRMotionModel::HeadsetState VRMotionModel::EstimateHeadsetState()
{
    return current_state_;
}

Eigen::Vector3f VRMotionModel::EstimateLinearVelocity()
{
    return linear_velocity_;
}

Eigen::Vector3f VRMotionModel::EstimateAngularVelocity()
{
    return angular_velocity_;
}

Eigen::Vector3f VRMotionModel::EstimateLinearAcceleration()
{
    return linear_acceleration_;
}

Eigen::Vector3f VRMotionModel::EstimateAngularAcceleration()
{
    return angular_acceleration_;
}

Eigen::Vector3f VRMotionModel::EstimateLinearJerk()
{
    return linear_jerk_;
}

Eigen::Vector3f VRMotionModel::EstimateAngularJerk()
{
    return angular_jerk_;
}

void VRMotionModel::Reset()
{
    // Clear history
    pose_history_.clear();
    imu_history_.clear();
    
    // Reset state
    current_state_ = HeadsetState::STATIONARY;
    linear_velocity_ = Eigen::Vector3f::Zero();
    angular_velocity_ = Eigen::Vector3f::Zero();
    linear_acceleration_ = Eigen::Vector3f::Zero();
    angular_acceleration_ = Eigen::Vector3f::Zero();
    linear_jerk_ = Eigen::Vector3f::Zero();
    angular_jerk_ = Eigen::Vector3f::Zero();
    
    // Reset Kalman filter
    initializeKalmanFilter();
}

void VRMotionModel::SetLatencyCompensation(double latency_ms)
{
    latency_compensation_ms_ = latency_ms;
}

double VRMotionModel::GetLatencyCompensation() const
{
    return latency_compensation_ms_;
}

void VRMotionModel::SetInteractionMode(InteractionMode mode)
{
    interaction_mode_ = mode;
    
    // Adjust prediction parameters based on interaction mode
    switch (mode) {
        case InteractionMode::SEATED:
            // Seated mode: less movement, more precise rotation
            config_.stationary_threshold = 0.03;   // 3cm/s
            config_.fast_movement_threshold = 0.4; // 40cm/s
            config_.rotation_only_threshold = 0.05; // Lower threshold for rotation-only detection
            break;
            
        case InteractionMode::STANDING:
            // Standing mode: moderate movement
            config_.stationary_threshold = 0.05;   // 5cm/s
            config_.fast_movement_threshold = 0.5; // 50cm/s
            config_.rotation_only_threshold = 0.1; // Default threshold
            break;
            
        case InteractionMode::ROOM_SCALE:
            // Room-scale mode: more movement, faster transitions
            config_.stationary_threshold = 0.08;   // 8cm/s
            config_.fast_movement_threshold = 0.7; // 70cm/s
            config_.rotation_only_threshold = 0.15; // Higher threshold for rotation-only detection
            break;
    }
}

VRMotionModel::InteractionMode VRMotionModel::GetInteractionMode() const
{
    return interaction_mode_;
}

void VRMotionModel::UpdateState()
{
    // Need at least two poses to estimate velocity
    if (pose_history_.size() < 2) {
        current_state_ = HeadsetState::STATIONARY;
        return;
    }
    
    // Get the two most recent poses
    const PoseRecord& current = pose_history_[0];
    const PoseRecord& previous = pose_history_[1];
    
    // Calculate time difference
    double dt = current.timestamp - previous.timestamp;
    if (dt <= 0.0) {
        std::cerr << "Invalid timestamps in pose history" << std::endl;
        return;
    }
    
    // Calculate position and orientation differences
    Eigen::Vector3f position_diff = current.pose.translation() - previous.pose.translation();
    Eigen::Quaternionf q_current = current.pose.unit_quaternion();
    Eigen::Quaternionf q_previous = previous.pose.unit_quaternion();
    Eigen::Quaternionf q_diff = q_current * q_previous.inverse();
    
    // Convert quaternion difference to angular velocity
    Eigen::AngleAxisf angle_axis(q_diff);
    Eigen::Vector3f rotation_diff = angle_axis.axis() * angle_axis.angle();
    
    // Calculate linear and angular velocities
    Eigen::Vector3f new_linear_velocity = position_diff / dt;
    Eigen::Vector3f new_angular_velocity = rotation_diff / dt;
    
    // Calculate linear and angular accelerations if we have enough history
    if (pose_history_.size() >= 3) {
        const PoseRecord& prev_prev = pose_history_[2];
        double prev_dt = previous.timestamp - prev_prev.timestamp;
        
        if (prev_dt > 0.0) {
            Eigen::Vector3f prev_position_diff = previous.pose.translation() - prev_prev.pose.translation();
            Eigen::Quaternionf q_prev_prev = prev_prev.pose.unit_quaternion();
            Eigen::Quaternionf q_prev_diff = q_previous * q_prev_prev.inverse();
            Eigen::AngleAxisf prev_angle_axis(q_prev_diff);
            Eigen::Vector3f prev_rotation_diff = prev_angle_axis.axis() * prev_angle_axis.angle();
            
            Eigen::Vector3f prev_linear_velocity = prev_position_diff / prev_dt;
            Eigen::Vector3f prev_angular_velocity = prev_rotation_diff / prev_dt;
            
            Eigen::Vector3f new_linear_acceleration = (new_linear_velocity - prev_linear_velocity) / dt;
            Eigen::Vector3f new_angular_acceleration = (new_angular_velocity - prev_angular_velocity) / dt;
            
            // Apply smoothing to acceleration
            const float accel_alpha = 0.6f; // Smoothing factor for acceleration
            linear_acceleration_ = accel_alpha * new_linear_acceleration + (1.0f - accel_alpha) * linear_acceleration_;
            angular_acceleration_ = accel_alpha * new_angular_acceleration + (1.0f - accel_alpha) * angular_acceleration_;
            
            // Calculate jerk if we have enough history
            if (pose_history_.size() >= 4) {
                const PoseRecord& prev_prev_prev = pose_history_[3];
                double prev_prev_dt = prev_prev.timestamp - prev_prev_prev.timestamp;
                
                if (prev_prev_dt > 0.0) {
                    Eigen::Vector3f prev_prev_position_diff = prev_prev.pose.translation() - prev_prev_prev.pose.translation();
                    Eigen::Quaternionf q_prev_prev_prev = prev_prev_prev.pose.unit_quaternion();
                    Eigen::Quaternionf q_prev_prev_diff = q_prev_prev * q_prev_prev_prev.inverse();
                    Eigen::AngleAxisf prev_prev_angle_axis(q_prev_prev_diff);
                    Eigen::Vector3f prev_prev_rotation_diff = prev_prev_angle_axis.axis() * prev_prev_angle_axis.angle();
                    
                    Eigen::Vector3f prev_prev_linear_velocity = prev_prev_position_diff / prev_prev_dt;
                    Eigen::Vector3f prev_prev_angular_velocity = prev_prev_rotation_diff / prev_prev_dt;
                    
                    Eigen::Vector3f prev_linear_acceleration = (prev_linear_velocity - prev_prev_linear_velocity) / prev_dt;
                    Eigen::Vector3f prev_angular_acceleration = (prev_angular_velocity - prev_prev_angular_velocity) / prev_dt;
                    
                    Eigen::Vector3f new_linear_jerk = (new_linear_acceleration - prev_linear_acceleration) / dt;
                    Eigen::Vector3f new_angular_jerk = (new_angular_acceleration - prev_angular_acceleration) / dt;
                    
                    // Apply smoothing to jerk
                    const float jerk_alpha = 0.5f; // Smoothing factor for jerk
                    linear_jerk_ = jerk_alpha * new_linear_jerk + (1.0f - jerk_alpha) * linear_jerk_;
                    angular_jerk_ = jerk_alpha * new_angular_jerk + (1.0f - jerk_alpha) * angular_jerk_;
                }
            }
        }
    }
    
    // Update velocities with some smoothing
    const float alpha = 0.7f; // Smoothing factor for velocity
    linear_velocity_ = alpha * new_linear_velocity + (1.0f - alpha) * linear_velocity_;
    angular_velocity_ = alpha * new_angular_velocity + (1.0f - alpha) * angular_velocity_;
    
    // Determine headset state
    float linear_speed = linear_velocity_.norm();
    float angular_speed = angular_velocity_.norm();
    
    // Adjust thresholds based on interaction mode
    float stationary_threshold = config_.stationary_threshold;
    float fast_movement_threshold = config_.fast_movement_threshold;
    float rotation_only_threshold = config_.rotation_only_threshold;
    
    // Determine state based on thresholds
    if (linear_speed < stationary_threshold && angular_speed < 0.1f) {
        current_state_ = HeadsetState::STATIONARY;
    } else if (linear_speed > fast_movement_threshold) {
        current_state_ = HeadsetState::FAST_MOVEMENT;
    } else if (linear_speed < rotation_only_threshold && angular_speed > 0.2f) {
        current_state_ = HeadsetState::ROTATION_ONLY;
    } else {
        current_state_ = HeadsetState::SLOW_MOVEMENT;
    }
    
    // Update user behavior model
    updateUserBehaviorModel();
}

void VRMotionModel::PruneHistory()
{
    // Maintain maximum history size
    while (pose_history_.size() > MAX_HISTORY_SIZE) {
        pose_history_.pop_back();
    }
    
    // Also remove poses that are too old (more than 1 second)
    if (!pose_history_.empty()) {
        double newest_timestamp = pose_history_.front().timestamp;
        while (!pose_history_.empty() && 
               (newest_timestamp - pose_history_.back().timestamp) > 1.0) {
            pose_history_.pop_back();
        }
    }
}

Sophus::SE3f VRMotionModel::PredictWithConstantVelocity(double prediction_time_ms)
{
    // Need at least two poses for velocity-based prediction
    if (pose_history_.size() < 2) {
        return pose_history_.front().pose;
    }
    
    // Get most recent pose
    const Sophus::SE3f& current_pose = pose_history_.front().pose;
    
    // Convert prediction time to seconds
    double prediction_time_s = prediction_time_ms / 1000.0;
    
    // Predict translation using linear velocity
    Eigen::Vector3f predicted_translation = current_pose.translation() + 
                                           linear_velocity_ * prediction_time_s;
    
    // Predict rotation using angular velocity
    Eigen::Vector3f rotation_vector = angular_velocity_ * prediction_time_s;
    Eigen::AngleAxisf rotation(rotation_vector.norm(), 
                              rotation_vector.norm() > 1e-6 ? rotation_vector.normalized() : Eigen::Vector3f::UnitX());
    Eigen::Quaternionf predicted_rotation = rotation * current_pose.unit_quaternion();
    
    // Combine into predicted pose
    return Sophus::SE3f(predicted_rotation, predicted_translation);
}

Sophus::SE3f VRMotionModel::PredictWithConstantAcceleration(double prediction_time_ms)
{
    // Need at least three poses for acceleration-based prediction
    if (pose_history_.size() < 3) {
        return PredictWithConstantVelocity(prediction_time_ms);
    }
    
    // Get most recent pose
    const Sophus::SE3f& current_pose = pose_history_.front().pose;
    
    // Convert prediction time to seconds
    double prediction_time_s = prediction_time_ms / 1000.0;
    double t_squared = prediction_time_s * prediction_time_s;
    
    // Predict translation using linear velocity and acceleration
    Eigen::Vector3f predicted_translation = current_pose.translation() + 
                                           linear_velocity_ * prediction_time_s +
                                           0.5f * linear_acceleration_ * t_squared;
    
    // Predict rotation using angular velocity and acceleration
    Eigen::Vector3f predicted_angular_velocity = angular_velocity_ + 
                                               angular_acceleration_ * prediction_time_s;
    Eigen::Vector3f avg_angular_velocity = (angular_velocity_ + predicted_angular_velocity) * 0.5f;
    Eigen::Vector3f rotation_vector = avg_angular_velocity * prediction_time_s;
    
    Eigen::AngleAxisf rotation(rotation_vector.norm(), 
                              rotation_vector.norm() > 1e-6 ? rotation_vector.normalized() : Eigen::Vector3f::UnitX());
    Eigen::Quaternionf predicted_rotation = rotation * current_pose.unit_quaternion();
    
    // Combine into predicted pose
    return Sophus::SE3f(predicted_rotation, predicted_translation);
}

Sophus::SE3f VRMotionModel::PredictWithJerk(double prediction_time_ms)
{
    // Need at least four poses for jerk-based prediction
    if (pose_history_.size() < 4) {
        return PredictWithConstantAcceleration(prediction_time_ms);
    }
    
    // Get most recent pose
    const Sophus::SE3f& current_pose = pose_history_.front().pose;
    
    // Convert prediction time to seconds
    double prediction_time_s = prediction_time_ms / 1000.0;
    double t_squared = prediction_time_s * prediction_time_s;
    double t_cubed = t_squared * prediction_time_s;
    
    // Predict translation using linear velocity, acceleration, and jerk
    Eigen::Vector3f predicted_translation = current_pose.translation() + 
                                           linear_velocity_ * prediction_time_s +
                                           0.5f * linear_acceleration_ * t_squared +
                                           (1.0f/6.0f) * linear_jerk_ * t_cubed;
    
    // Predict angular velocity using angular acceleration and jerk
    Eigen::Vector3f predicted_angular_velocity = angular_velocity_ + 
                                               angular_acceleration_ * prediction_time_s +
                                               0.5f * angular_jerk_ * t_squared;
    
    // Use average angular velocity for rotation prediction
    Eigen::Vector3f avg_angular_velocity = (angular_velocity_ + predicted_angular_velocity) * 0.5f;
    Eigen::Vector3f rotation_vector = avg_angular_velocity * prediction_time_s;
    
    Eigen::AngleAxisf rotation(rotation_vector.norm(), 
                              rotation_vector.norm() > 1e-6 ? rotation_vector.normalized() : Eigen::Vector3f::UnitX());
    Eigen::Quaternionf predicted_rotation = rotation * current_pose.unit_quaternion();
    
    // Combine into predicted pose
    return Sophus::SE3f(predicted_rotation, predicted_translation);
}

Sophus::SE3f VRMotionModel::PredictWithIMU(double prediction_time_ms)
{
    // Need at least one pose and one IMU measurement
    if (pose_history_.empty() || imu_history_.empty()) {
        return pose_history_.empty() ? Sophus::SE3f() : pose_history_.front().pose;
    }
    
    // Get most recent pose and IMU measurement
    const Sophus::SE3f& current_pose = pose_history_.front().pose;
    const IMURecord& current_imu = imu_history_.front();
    
    // Convert prediction time to seconds
    double prediction_time_s = prediction_time_ms / 1000.0;
    
    // For rotation prediction, use gyroscope directly
    Eigen::Vector3f rotation_vector = current_imu.gyro * prediction_time_s;
    Eigen::AngleAxisf rotation(rotation_vector.norm(), 
                              rotation_vector.norm() > 1e-6 ? rotation_vector.normalized() : Eigen::Vector3f::UnitX());
    Eigen::Quaternionf predicted_rotation = rotation * current_pose.unit_quaternion();
    
    // For translation prediction, we need to:
    // 1. Remove gravity from accelerometer reading
    // 2. Transform acceleration from IMU frame to world frame
    // 3. Integrate twice to get position
    
    // Gravity vector in world frame (assuming Z is up)
    Eigen::Vector3f gravity_world(0, 0, 9.81);
    
    // Transform gravity to IMU frame
    Eigen::Vector3f gravity_imu = current_pose.unit_quaternion().inverse() * gravity_world;
    
    // Remove gravity from accelerometer reading
    Eigen::Vector3f accel_without_gravity = current_imu.accel - gravity_imu;
    
    // Transform acceleration to world frame
    Eigen::Vector3f accel_world = current_pose.unit_quaternion() * accel_without_gravity;
    
    // Integrate acceleration to get velocity (using current velocity estimate)
    Eigen::Vector3f predicted_velocity = linear_velocity_ + accel_world * prediction_time_s;
    
    // Integrate velocity to get position
    Eigen::Vector3f predicted_translation = current_pose.translation() + 
                                           0.5f * (linear_velocity_ + predicted_velocity) * prediction_time_s;
    
    // Combine into predicted pose
    return Sophus::SE3f(predicted_rotation, predicted_translation);
}

Sophus::SE3f VRMotionModel::InterpolatePoses(const Sophus::SE3f& pose1, const Sophus::SE3f& pose2, float t)
{
    // Interpolate translation linearly
    Eigen::Vector3f translation = pose1.translation() * (1 - t) + pose2.translation() * t;
    
    // Interpolate rotation using SLERP
    Eigen::Quaternionf q1 = pose1.unit_quaternion();
    Eigen::Quaternionf q2 = pose2.unit_quaternion();
    Eigen::Quaternionf q = q1.slerp(t, q2);
    
    return Sophus::SE3f(q, translation);
}

void VRMotionModel::initializeKalmanFilter()
{
    // State vector: [x, y, z, qw, qx, qy, qz, vx, vy, vz, wx, wy, wz, ax, ay, az]
    // where (x,y,z) is position, (qw,qx,qy,qz) is orientation quaternion,
    // (vx,vy,vz) is linear velocity, (wx,wy,wz) is angular velocity,
    // and (ax,ay,az) is linear acceleration
    
    // Initialize state vector
    kalman_state_ = Eigen::VectorXf::Zero(16);
    
    // Initialize state covariance matrix
    kalman_covariance_ = Eigen::MatrixXf::Identity(16, 16);
    
    // Set initial uncertainties
    for (int i = 0; i < 3; ++i) {
        kalman_covariance_(i, i) = 0.01f;  // Position uncertainty
    }
    for (int i = 3; i < 7; ++i) {
        kalman_covariance_(i, i) = 0.01f;  // Orientation uncertainty
    }
    for (int i = 7; i < 10; ++i) {
        kalman_covariance_(i, i) = 0.1f;   // Linear velocity uncertainty
    }
    for (int i = 10; i < 13; ++i) {
        kalman_covariance_(i, i) = 0.1f;   // Angular velocity uncertainty
    }
    for (int i = 13; i < 16; ++i) {
        kalman_covariance_(i, i) = 1.0f;   // Linear acceleration uncertainty
    }
    
    // Initialize process noise covariance
    kalman_process_noise_ = Eigen::MatrixXf::Identity(16, 16) * 0.01f;
    
    // Initialize measurement noise covariance
    kalman_measurement_noise_ = Eigen::MatrixXf::Identity(7, 7) * 0.01f;
    
    // Set last update time
    kalman_last_update_time_ = 0.0;
}

void VRMotionModel::updateKalmanFilter(const Sophus::SE3f& pose, double timestamp)
{
    // Skip if this is the first update
    if (kalman_last_update_time_ == 0.0) {
        // Initialize state with first pose
        kalman_state_.segment<3>(0) = pose.translation();
        Eigen::Quaternionf q = pose.unit_quaternion();
        kalman_state_(3) = q.w();
        kalman_state_(4) = q.x();
        kalman_state_(5) = q.y();
        kalman_state_(6) = q.z();
        
        kalman_last_update_time_ = timestamp;
        return;
    }
    
    // Calculate time delta
    double dt = timestamp - kalman_last_update_time_;
    if (dt <= 0.0) {
        return;
    }
    
    // Predict step
    predictKalmanState(dt);
    
    // Update step
    
    // Measurement vector: [x, y, z, qw, qx, qy, qz]
    Eigen::VectorXf measurement(7);
    measurement.segment<3>(0) = pose.translation();
    Eigen::Quaternionf q = pose.unit_quaternion();
    measurement(3) = q.w();
    measurement(4) = q.x();
    measurement(5) = q.y();
    measurement(6) = q.z();
    
    // Measurement matrix (maps state to measurement)
    Eigen::MatrixXf H = Eigen::MatrixXf::Zero(7, 16);
    H.block<3, 3>(0, 0) = Eigen::Matrix3f::Identity(); // Position
    H.block<4, 4>(3, 3) = Eigen::Matrix4f::Identity(); // Orientation
    
    // Kalman gain
    Eigen::MatrixXf K = kalman_covariance_ * H.transpose() * 
                       (H * kalman_covariance_ * H.transpose() + kalman_measurement_noise_).inverse();
    
    // Update state
    Eigen::VectorXf innovation = measurement - H * kalman_state_;
    
    // Special handling for quaternion innovation (shortest path)
    Eigen::Quaternionf q_state(kalman_state_(3), kalman_state_(4), kalman_state_(5), kalman_state_(6));
    Eigen::Quaternionf q_meas(measurement(3), measurement(4), measurement(5), measurement(6));
    
    // Ensure quaternions are normalized
    q_state.normalize();
    q_meas.normalize();
    
    // Calculate quaternion difference
    Eigen::Quaternionf q_diff = q_meas * q_state.inverse();
    
    // Convert to axis-angle representation
    Eigen::AngleAxisf aa(q_diff);
    Eigen::Vector3f axis_angle = aa.axis() * aa.angle();
    
    // Replace quaternion innovation with axis-angle
    innovation.segment<4>(3) << 0, axis_angle.x(), axis_angle.y(), axis_angle.z();
    
    // Update state
    kalman_state_ = kalman_state_ + K * innovation;
    
    // Normalize quaternion part of state
    Eigen::Quaternionf q_updated(kalman_state_(3), kalman_state_(4), kalman_state_(5), kalman_state_(6));
    q_updated.normalize();
    kalman_state_(3) = q_updated.w();
    kalman_state_(4) = q_updated.x();
    kalman_state_(5) = q_updated.y();
    kalman_state_(6) = q_updated.z();
    
    // Update covariance
    kalman_covariance_ = (Eigen::MatrixXf::Identity(16, 16) - K * H) * kalman_covariance_;
    
    // Update last update time
    kalman_last_update_time_ = timestamp;
}

void VRMotionModel::updateKalmanFilterWithIMU(const Eigen::Vector3f& gyro, const Eigen::Vector3f& accel, double timestamp)
{
    // Skip if this is the first update
    if (kalman_last_update_time_ == 0.0) {
        kalman_last_update_time_ = timestamp;
        return;
    }
    
    // Calculate time delta
    double dt = timestamp - kalman_last_update_time_;
    if (dt <= 0.0) {
        return;
    }
    
    // Predict step
    predictKalmanState(dt);
    
    // Update step for IMU measurements
    
    // Measurement vector: [wx, wy, wz, ax, ay, az]
    Eigen::VectorXf measurement(6);
    measurement.segment<3>(0) = gyro;
    measurement.segment<3>(3) = accel;
    
    // Measurement matrix (maps state to measurement)
    Eigen::MatrixXf H = Eigen::MatrixXf::Zero(6, 16);
    H.block<3, 3>(0, 10) = Eigen::Matrix3f::Identity(); // Angular velocity
    H.block<3, 3>(3, 13) = Eigen::Matrix3f::Identity(); // Linear acceleration
    
    // Measurement noise for IMU
    Eigen::MatrixXf R = Eigen::MatrixXf::Identity(6, 6) * 0.1f;
    
    // Kalman gain
    Eigen::MatrixXf K = kalman_covariance_ * H.transpose() * 
                       (H * kalman_covariance_ * H.transpose() + R).inverse();
    
    // Update state
    kalman_state_ = kalman_state_ + K * (measurement - H * kalman_state_);
    
    // Update covariance
    kalman_covariance_ = (Eigen::MatrixXf::Identity(16, 16) - K * H) * kalman_covariance_;
    
    // Update last update time
    kalman_last_update_time_ = timestamp;
}

Eigen::VectorXf VRMotionModel::predictKalmanState(double dt)
{
    // State transition matrix
    Eigen::MatrixXf F = Eigen::MatrixXf::Identity(16, 16);
    
    // Position update from velocity
    F.block<3, 3>(0, 7) = Eigen::Matrix3f::Identity() * dt;
    
    // Velocity update from acceleration
    F.block<3, 3>(7, 13) = Eigen::Matrix3f::Identity() * dt;
    
    // Predict state
    Eigen::VectorXf predicted_state = F * kalman_state_;
    
    // Special handling for quaternion update using angular velocity
    Eigen::Quaternionf q(kalman_state_(3), kalman_state_(4), kalman_state_(5), kalman_state_(6));
    Eigen::Vector3f omega(kalman_state_(10), kalman_state_(11), kalman_state_(12));
    
    // Create quaternion from angular velocity
    float angle = omega.norm() * dt;
    Eigen::Vector3f axis = omega.normalized();
    Eigen::Quaternionf dq;
    if (angle < 1e-6) {
        dq = Eigen::Quaternionf::Identity();
    } else {
        dq = Eigen::Quaternionf(Eigen::AngleAxisf(angle, axis));
    }
    
    // Update quaternion
    Eigen::Quaternionf q_updated = q * dq;
    q_updated.normalize();
    
    // Update quaternion part of state
    predicted_state(3) = q_updated.w();
    predicted_state(4) = q_updated.x();
    predicted_state(5) = q_updated.y();
    predicted_state(6) = q_updated.z();
    
    // Update state covariance
    kalman_covariance_ = F * kalman_covariance_ * F.transpose() + kalman_process_noise_;
    
    // Update state
    kalman_state_ = predicted_state;
    
    return predicted_state;
}

void VRMotionModel::updateUserBehaviorModel()
{
    // Skip if we don't have enough history
    if (pose_history_.size() < 10) {
        return;
    }
    
    // Calculate average time between poses
    double avg_dt = 0.0;
    for (size_t i = 1; i < pose_history_.size(); ++i) {
        avg_dt += pose_history_[i-1].timestamp - pose_history_[i].timestamp;
    }
    avg_dt /= (pose_history_.size() - 1);
    
    // Calculate average linear and angular speeds
    float avg_linear_speed = 0.0f;
    float avg_angular_speed = 0.0f;
    int stationary_count = 0;
    int rotation_only_count = 0;
    int slow_movement_count = 0;
    int fast_movement_count = 0;
    
    for (size_t i = 1; i < pose_history_.size(); ++i) {
        double dt = pose_history_[i-1].timestamp - pose_history_[i].timestamp;
        if (dt <= 0.0) continue;
        
        Eigen::Vector3f position_diff = pose_history_[i-1].pose.translation() - pose_history_[i].pose.translation();
        float linear_speed = position_diff.norm() / dt;
        avg_linear_speed += linear_speed;
        
        Eigen::Quaternionf q1 = pose_history_[i-1].pose.unit_quaternion();
        Eigen::Quaternionf q2 = pose_history_[i].pose.unit_quaternion();
        Eigen::Quaternionf q_diff = q1 * q2.inverse();
        Eigen::AngleAxisf angle_axis(q_diff);
        float angular_speed = angle_axis.angle() / dt;
        avg_angular_speed += angular_speed;
        
        // Count state occurrences
        if (linear_speed < config_.stationary_threshold && angular_speed < 0.1f) {
            stationary_count++;
        } else if (linear_speed > config_.fast_movement_threshold) {
            fast_movement_count++;
        } else if (linear_speed < config_.rotation_only_threshold && angular_speed > 0.2f) {
            rotation_only_count++;
        } else {
            slow_movement_count++;
        }
    }
    
    avg_linear_speed /= (pose_history_.size() - 1);
    avg_angular_speed /= (pose_history_.size() - 1);
    
    // Update user behavior model
    user_behavior_.avg_linear_speed = avg_linear_speed;
    user_behavior_.avg_angular_speed = avg_angular_speed;
    user_behavior_.stationary_ratio = static_cast<float>(stationary_count) / (pose_history_.size() - 1);
    user_behavior_.rotation_only_ratio = static_cast<float>(rotation_only_count) / (pose_history_.size() - 1);
    user_behavior_.slow_movement_ratio = static_cast<float>(slow_movement_count) / (pose_history_.size() - 1);
    user_behavior_.fast_movement_ratio = static_cast<float>(fast_movement_count) / (pose_history_.size() - 1);
    
    // Adapt prediction parameters based on user behavior
    if (config_.adaptive_prediction) {
        // If user is mostly stationary, reduce prediction horizon
        if (user_behavior_.stationary_ratio > 0.7f) {
            config_.prediction_horizon_ms = std::min(config_.prediction_horizon_ms, 10.0);
        }
        // If user is mostly doing fast movements, increase prediction horizon
        else if (user_behavior_.fast_movement_ratio > 0.5f) {
            config_.prediction_horizon_ms = std::min(30.0, config_.prediction_horizon_ms * 1.2);
        }
        // If user is mostly doing rotation-only movements, focus on rotation prediction
        else if (user_behavior_.rotation_only_ratio > 0.5f) {
            config_.rotation_only_threshold *= 1.1f;
        }
    }
}

} // namespace ORB_SLAM3

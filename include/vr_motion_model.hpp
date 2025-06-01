#ifndef VR_MOTION_MODEL_HPP
#define VR_MOTION_MODEL_HPP

#include <vector>
#include <deque>
#include <Eigen/Core>
#include <Eigen/Geometry>
#include <sophus/se3.hpp>

namespace ORB_SLAM3
{

/**
 * @brief VR-specific motion model for headset tracking
 * 
 * This class implements motion prediction and state estimation
 * algorithms specifically designed for VR headset tracking.
 */
class VRMotionModel
{
public:
    /**
     * @brief Headset motion state enumeration
     */
    enum class HeadsetState {
        STATIONARY,     ///< Headset is not moving
        SLOW_MOVEMENT,  ///< Headset is moving slowly
        FAST_MOVEMENT,  ///< Headset is moving quickly
        ROTATION_ONLY   ///< Headset is rotating but not translating
    };
    
    /**
     * @brief VR interaction mode enumeration
     */
    enum class InteractionMode {
        SEATED,         ///< User is seated (limited movement range)
        STANDING,       ///< User is standing (moderate movement range)
        ROOM_SCALE      ///< User is moving around a room (large movement range)
    };
    
    /**
     * @brief Configuration for prediction parameters
     */
    struct PredictionConfig {
        double prediction_horizon_ms;   ///< How far ahead to predict
        double max_prediction_ms;       ///< Maximum prediction time
        bool use_imu_for_prediction;    ///< Whether to use IMU data for prediction
        bool adaptive_prediction;       ///< Whether to adapt prediction based on motion
        double stationary_threshold;    ///< Velocity threshold for stationary state (m/s)
        double fast_movement_threshold; ///< Velocity threshold for fast movement state (m/s)
        double rotation_only_threshold; ///< Translation to rotation ratio threshold
    };
    
    /**
     * @brief User behavior model
     */
    struct UserBehaviorModel {
        float avg_linear_speed;       ///< Average linear speed (m/s)
        float avg_angular_speed;      ///< Average angular speed (rad/s)
        float stationary_ratio;       ///< Ratio of time spent stationary
        float rotation_only_ratio;    ///< Ratio of time spent in rotation-only state
        float slow_movement_ratio;    ///< Ratio of time spent in slow movement
        float fast_movement_ratio;    ///< Ratio of time spent in fast movement
    };
    
    /**
     * @brief Default constructor
     */
    VRMotionModel();
    
    /**
     * @brief Constructor with configuration
     * 
     * @param config Prediction configuration
     */
    explicit VRMotionModel(const PredictionConfig& config);
    
    /**
     * @brief Destructor
     */
    ~VRMotionModel();
    
    /**
     * @brief Set prediction configuration
     * 
     * @param config Prediction configuration
     */
    void SetConfig(const PredictionConfig& config);
    
    /**
     * @brief Get current prediction configuration
     * 
     * @return Current prediction configuration
     */
    PredictionConfig GetConfig() const;
    
    /**
     * @brief Add a new pose to the history
     * 
     * @param pose New pose
     * @param timestamp Timestamp in seconds
     */
    void AddPose(const Sophus::SE3f& pose, double timestamp);
    
    /**
     * @brief Add IMU measurement
     * 
     * @param gyro Gyroscope measurement (rad/s)
     * @param accel Accelerometer measurement (m/s^2)
     * @param timestamp Timestamp in seconds
     */
    void AddIMU(const Eigen::Vector3f& gyro, const Eigen::Vector3f& accel, double timestamp);
    
    /**
     * @brief Predict pose at a future time
     * 
     * @param prediction_time_ms Time in the future to predict (milliseconds)
     * @return Predicted pose
     */
    Sophus::SE3f PredictPose(double prediction_time_ms);
    
    /**
     * @brief Predict pose using Kalman filter
     * 
     * @param prediction_time_ms Time in the future to predict (milliseconds)
     * @return Predicted pose
     */
    Sophus::SE3f PredictPoseKalman(double prediction_time_ms);
    
    /**
     * @brief Estimate current headset state
     * 
     * @return Current headset state
     */
    HeadsetState EstimateHeadsetState();
    
    /**
     * @brief Estimate linear velocity
     * 
     * @return Current linear velocity (m/s)
     */
    Eigen::Vector3f EstimateLinearVelocity();
    
    /**
     * @brief Estimate angular velocity
     * 
     * @return Current angular velocity (rad/s)
     */
    Eigen::Vector3f EstimateAngularVelocity();
    
    /**
     * @brief Estimate linear acceleration
     * 
     * @return Current linear acceleration (m/s^2)
     */
    Eigen::Vector3f EstimateLinearAcceleration();
    
    /**
     * @brief Estimate angular acceleration
     * 
     * @return Current angular acceleration (rad/s^2)
     */
    Eigen::Vector3f EstimateAngularAcceleration();
    
    /**
     * @brief Estimate linear jerk
     * 
     * @return Current linear jerk (m/s^3)
     */
    Eigen::Vector3f EstimateLinearJerk();
    
    /**
     * @brief Estimate angular jerk
     * 
     * @return Current angular jerk (rad/s^3)
     */
    Eigen::Vector3f EstimateAngularJerk();
    
    /**
     * @brief Reset the motion model
     */
    void Reset();
    
    /**
     * @brief Set latency compensation
     * 
     * @param latency_ms Latency to compensate for (milliseconds)
     */
    void SetLatencyCompensation(double latency_ms);
    
    /**
     * @brief Get current latency compensation
     * 
     * @return Current latency compensation (milliseconds)
     */
    double GetLatencyCompensation() const;
    
    /**
     * @brief Set VR interaction mode
     * 
     * @param mode Interaction mode
     */
    void SetInteractionMode(InteractionMode mode);
    
    /**
     * @brief Get current interaction mode
     * 
     * @return Current interaction mode
     */
    InteractionMode GetInteractionMode() const;
    
    /**
     * @brief Get user behavior model
     * 
     * @return Current user behavior model
     */
    UserBehaviorModel GetUserBehaviorModel() const {
        return user_behavior_;
    }
    
private:
    // Configuration
    PredictionConfig config_;
    
    // Interaction mode
    InteractionMode interaction_mode_ = InteractionMode::STANDING;
    
    // User behavior model
    UserBehaviorModel user_behavior_ = {0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f};
    
    // Pose history
    struct PoseRecord {
        Sophus::SE3f pose;
        double timestamp;
    };
    std::deque<PoseRecord> pose_history_;
    
    // IMU history
    struct IMURecord {
        Eigen::Vector3f gyro;
        Eigen::Vector3f accel;
        double timestamp;
    };
    std::deque<IMURecord> imu_history_;
    
    // Current state
    HeadsetState current_state_;
    Eigen::Vector3f linear_velocity_;
    Eigen::Vector3f angular_velocity_;
    Eigen::Vector3f linear_acceleration_;
    Eigen::Vector3f angular_acceleration_;
    Eigen::Vector3f linear_jerk_;
    Eigen::Vector3f angular_jerk_;
    double latency_compensation_ms_;
    
    // Kalman filter state
    Eigen::VectorXf kalman_state_;
    Eigen::MatrixXf kalman_covariance_;
    Eigen::MatrixXf kalman_process_noise_;
    Eigen::MatrixXf kalman_measurement_noise_;
    double kalman_last_update_time_;
    
    // Maximum history size
    static constexpr int MAX_HISTORY_SIZE = 100;
    
    // Helper methods
    void UpdateState();
    void PruneHistory();
    Sophus::SE3f PredictWithConstantVelocity(double prediction_time_ms);
    Sophus::SE3f PredictWithConstantAcceleration(double prediction_time_ms);
    Sophus::SE3f PredictWithJerk(double prediction_time_ms);
    Sophus::SE3f PredictWithIMU(double prediction_time_ms);
    Sophus::SE3f InterpolatePoses(const Sophus::SE3f& pose1, const Sophus::SE3f& pose2, float t);
    
    // Kalman filter methods
    void initializeKalmanFilter();
    void updateKalmanFilter(const Sophus::SE3f& pose, double timestamp);
    void updateKalmanFilterWithIMU(const Eigen::Vector3f& gyro, const Eigen::Vector3f& accel, double timestamp);
    Eigen::VectorXf predictKalmanState(double dt);
    
    // User behavior modeling
    void updateUserBehaviorModel();
};

} // namespace ORB_SLAM3

#endif // VR_MOTION_MODEL_HPP

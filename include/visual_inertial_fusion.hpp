#ifndef VISUAL_INERTIAL_FUSION_HPP
#define VISUAL_INERTIAL_FUSION_HPP

#include <vector>
#include <memory>
#include <mutex>
#include <thread>
#include <atomic>
#include <queue>
#include <condition_variable>
#include <Eigen/Core>
#include <Eigen/Geometry>
#include <opencv2/core/core.hpp>

#include "ImuTypes.h"
#include "bno085_interface.hpp"
#include "multi_camera_tracking.hpp"
#include "vr_motion_model.hpp"

namespace ORB_SLAM3
{

/**
 * @brief Visual-Inertial Fusion class for VR SLAM
 * 
 * This class implements visual-inertial fusion based on VINS-Fusion principles,
 * optimized for VR applications. It integrates visual tracking from multiple cameras
 * with inertial data from the BNO085 IMU to provide robust and low-latency tracking.
 * 
 * Key features:
 * - Tight coupling of visual and inertial data
 * - Fast initialization for VR applications
 * - Robust tracking during rapid head movements
 * - Efficient relocalization after tracking loss
 * - Predictive tracking for low-latency VR rendering
 */
class VisualInertialFusion
{
public:
    /**
     * @brief Configuration structure for Visual-Inertial Fusion
     */
    struct Config {
        // General settings
        bool use_imu = true;                      ///< Whether to use IMU data
        bool use_multi_camera = true;             ///< Whether to use multiple cameras
        bool enable_mapping = true;               ///< Whether to enable mapping
        bool enable_loop_closing = true;          ///< Whether to enable loop closing
        
        // VI-specific settings
        float imu_frequency = 200.0f;             ///< IMU data frequency in Hz
        float visual_frequency = 90.0f;           ///< Visual tracking frequency in Hz
        float gravity_magnitude = 9.81f;          ///< Gravity magnitude in m/s^2
        
        // Initialization settings
        float init_time_threshold = 0.5f;         ///< Minimum time for initialization in seconds
        int init_min_features = 50;               ///< Minimum features for initialization
        float init_max_condition_number = 5000.0f; ///< Maximum condition number for initialization
        
        // Optimization settings
        int local_window_size = 10;               ///< Local window size for optimization
        int fixed_lag_size = 5;                   ///< Fixed-lag smoother size
        float huber_threshold = 0.1f;             ///< Huber loss threshold
        int max_iterations = 10;                  ///< Maximum iterations for optimization
        
        // VR-specific settings
        float prediction_horizon_ms = 16.0f;      ///< Prediction horizon for VR in milliseconds
        bool enable_jerk_modeling = true;         ///< Whether to model jerk for prediction
        bool adaptive_imu_integration = true;     ///< Whether to adapt IMU integration based on motion
        
        // Failure recovery settings
        float relocalization_timeout = 1.0f;      ///< Timeout for relocalization in seconds
        bool use_imu_only_fallback = true;        ///< Whether to fall back to IMU-only tracking
        float max_tracking_loss_time = 3.0f;      ///< Maximum time to maintain map during tracking loss
    };
    
    /**
     * @brief Fusion state enumeration
     */
    enum class State {
        UNINITIALIZED,      ///< System not initialized
        INITIALIZING,       ///< System initializing
        TRACKING_NOMINAL,   ///< Normal tracking with visual and inertial data
        TRACKING_RAPID,     ///< Tracking during rapid motion (prioritizes IMU)
        TRACKING_VISUAL,    ///< Tracking with primarily visual data (static scenes)
        LOST,               ///< Tracking lost
        RELOCALIZATION      ///< Attempting relocalization
    };
    
    /**
     * @brief Performance metrics structure
     */
    struct PerformanceMetrics {
        double average_fusion_time_ms = 0.0;      ///< Average fusion processing time
        double average_init_time_s = 0.0;         ///< Average initialization time
        double tracking_percentage = 100.0;       ///< Percentage of time with successful tracking
        int relocalization_count = 0;             ///< Number of relocalization events
        double average_relocalization_time_ms = 0.0; ///< Average relocalization time
        double position_rmse_mm = 0.0;            ///< Position RMSE in mm (if ground truth available)
        double orientation_rmse_deg = 0.0;        ///< Orientation RMSE in degrees (if ground truth available)
        double prediction_error_mm = 0.0;         ///< Prediction error at horizon time
    };
    
    /**
     * @brief Constructor with configuration
     * @param config Configuration structure
     * @param imu_interface Pointer to BNO085Interface
     * @param tracking Pointer to MultiCameraTracking
     * @param motion_model Pointer to VRMotionModel
     */
    VisualInertialFusion(
        const Config& config,
        std::shared_ptr<BNO085Interface> imu_interface,
        std::shared_ptr<MultiCameraTracking> tracking,
        std::shared_ptr<VRMotionModel> motion_model);
    
    /**
     * @brief Destructor
     */
    ~VisualInertialFusion();
    
    /**
     * @brief Initialize the fusion system
     * @return True if initialization was successful, false otherwise
     */
    bool Initialize();
    
    /**
     * @brief Start the fusion process
     * @return True if started successfully, false otherwise
     */
    bool Start();
    
    /**
     * @brief Stop the fusion process
     */
    void Stop();
    
    /**
     * @brief Reset the fusion system
     * @return True if reset was successful, false otherwise
     */
    bool Reset();
    
    /**
     * @brief Get the current fusion state
     * @return Current state
     */
    State GetState() const;
    
    /**
     * @brief Get the current pose estimate
     * @return Current pose as Sophus::SE3f
     */
    Sophus::SE3<float> GetCurrentPose() const;
    
    /**
     * @brief Get a predicted pose at a specified time in the future
     * @param prediction_time_ms Time in the future in milliseconds
     * @return Predicted pose
     */
    Sophus::SE3<float> GetPredictedPose(double prediction_time_ms) const;
    
    /**
     * @brief Get the current velocity estimate
     * @return Current velocity as Eigen::Vector3f
     */
    Eigen::Vector3f GetCurrentVelocity() const;
    
    /**
     * @brief Get the current acceleration estimate
     * @return Current acceleration as Eigen::Vector3f
     */
    Eigen::Vector3f GetCurrentAcceleration() const;
    
    /**
     * @brief Get the current angular velocity estimate
     * @return Current angular velocity as Eigen::Vector3f
     */
    Eigen::Vector3f GetCurrentAngularVelocity() const;
    
    /**
     * @brief Get the current bias estimates
     * @return Current IMU bias
     */
    IMU::Bias GetCurrentBias() const;
    
    /**
     * @brief Get the gravity direction in world frame
     * @return Gravity direction as Eigen::Vector3f
     */
    Eigen::Vector3f GetGravityDirection() const;
    
    /**
     * @brief Get the performance metrics
     * @return Performance metrics structure
     */
    PerformanceMetrics GetPerformanceMetrics() const;
    
    /**
     * @brief Process a batch of IMU measurements
     * @param measurements Vector of IMU measurements
     * @return True if processing was successful, false otherwise
     */
    bool ProcessIMUMeasurements(const std::vector<IMU::Point>& measurements);
    
    /**
     * @brief Process visual tracking results
     * @param pose Tracked camera pose
     * @param timestamp Timestamp of the pose
     * @param keypoints Vector of keypoints from all cameras
     * @param map_points Vector of corresponding map points
     * @return True if processing was successful, false otherwise
     */
    bool ProcessVisualTracking(
        const Sophus::SE3<float>& pose,
        double timestamp,
        const std::vector<std::vector<cv::KeyPoint>>& keypoints,
        const std::vector<std::vector<MapPoint*>>& map_points);
    
    /**
     * @brief Set the prediction horizon for VR
     * @param prediction_horizon_ms Prediction horizon in milliseconds
     */
    void SetPredictionHorizon(double prediction_horizon_ms);
    
    /**
     * @brief Set the VR interaction mode
     * @param mode Interaction mode
     */
    void SetVRInteractionMode(VRMotionModel::InteractionMode mode);
    
    /**
     * @brief Check if the system is initialized
     * @return True if initialized, false otherwise
     */
    bool IsInitialized() const;
    
    /**
     * @brief Check if tracking is currently good
     * @return True if tracking is good, false otherwise
     */
    bool IsTrackingGood() const;
    
    /**
     * @brief Get the initialization progress
     * @return Progress as a percentage (0-100)
     */
    float GetInitializationProgress() const;
    
    /**
     * @brief Get the tracking quality
     * @return Quality metric (0-1, where 1 is best)
     */
    float GetTrackingQuality() const;
    
    /**
     * @brief Save the current state to a file
     * @param filename Filename to save to
     * @return True if successful, false otherwise
     */
    bool SaveState(const std::string& filename) const;
    
    /**
     * @brief Load state from a file
     * @param filename Filename to load from
     * @return True if successful, false otherwise
     */
    bool LoadState(const std::string& filename);

private:
    // Configuration
    Config mConfig;
    
    // Component interfaces
    std::shared_ptr<BNO085Interface> mIMUInterface;
    std::shared_ptr<MultiCameraTracking> mTracking;
    std::shared_ptr<VRMotionModel> mMotionModel;
    
    // State variables
    std::atomic<State> mState;
    std::mutex mStateMutex;
    
    // Pose and motion state
    std::mutex mPoseMutex;
    Sophus::SE3<float> mCurrentPose;
    Eigen::Vector3f mCurrentVelocity;
    Eigen::Vector3f mCurrentAcceleration;
    Eigen::Vector3f mCurrentAngularVelocity;
    Eigen::Vector3f mGravityDirection;
    
    // IMU state
    std::mutex mIMUMutex;
    IMU::Bias mCurrentBias;
    IMU::Preintegrated* mpImuPreintegrated;
    std::queue<IMU::Point> mIMUQueue;
    double mLastIMUTimestamp;
    
    // Visual tracking state
    std::mutex mVisualMutex;
    double mLastVisualTimestamp;
    bool mVisualTrackingGood;
    int mTrackingLossCount;
    
    // Initialization state
    std::mutex mInitMutex;
    float mInitProgress;
    double mInitStartTime;
    bool mGravityInitialized;
    
    // Performance monitoring
    std::mutex mMetricsMutex;
    PerformanceMetrics mMetrics;
    
    // Processing thread
    std::thread mProcessingThread;
    std::atomic<bool> mRunning;
    std::condition_variable mProcessingCondition;
    std::mutex mProcessingMutex;
    
    // Private methods
    
    /**
     * @brief Main processing thread function
     */
    void ProcessingThreadFunction();
    
    /**
     * @brief Initialize the system with visual and inertial data
     * @return True if initialization was successful, false otherwise
     */
    bool InitializeSystem();
    
    /**
     * @brief Initialize gravity direction from IMU measurements
     * @return True if successful, false otherwise
     */
    bool InitializeGravity();
    
    /**
     * @brief Initialize scale and velocity from visual and inertial data
     * @return True if successful, false otherwise
     */
    bool InitializeScaleAndVelocity();
    
    /**
     * @brief Perform visual-inertial bundle adjustment
     * @param local_only Whether to perform only local bundle adjustment
     * @return True if successful, false otherwise
     */
    bool PerformVisualInertialBA(bool local_only = true);
    
    /**
     * @brief Update motion state from visual and inertial data
     * @return True if successful, false otherwise
     */
    bool UpdateMotionState();
    
    /**
     * @brief Attempt relocalization after tracking loss
     * @return True if relocalization was successful, false otherwise
     */
    bool AttemptRelocalization();
    
    /**
     * @brief Update the motion model with latest state
     * @return True if successful, false otherwise
     */
    bool UpdateMotionModel();
    
    /**
     * @brief Preintegrate IMU measurements
     * @param start_time Start time for preintegration
     * @param end_time End time for preintegration
     * @return True if successful, false otherwise
     */
    bool PreintegrateIMU(double start_time, double end_time);
    
    /**
     * @brief Detect and handle rapid motion
     * @return True if rapid motion was detected, false otherwise
     */
    bool DetectAndHandleRapidMotion();
    
    /**
     * @brief Update tracking state based on visual and inertial data
     */
    void UpdateTrackingState();
    
    /**
     * @brief Update performance metrics
     * @param fusion_time Fusion processing time in milliseconds
     */
    void UpdatePerformanceMetrics(double fusion_time);
};

} // namespace ORB_SLAM3

#endif // VISUAL_INERTIAL_FUSION_HPP

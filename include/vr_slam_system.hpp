#ifndef VR_SLAM_SYSTEM_HPP
#define VR_SLAM_SYSTEM_HPP

#include <memory>
#include <vector>
#include <string>
#include <thread>
#include <mutex>
#include <atomic>
#include <condition_variable>

#include "multi_camera_rig.hpp"
#include "multi_camera_tracking.hpp"
#include "vr_motion_model.hpp"
#include "tpu_zero_copy_integration.hpp"
#include "bno085_interface.hpp"
#include "zero_copy_frame_provider.hpp"

namespace ORB_SLAM3
{

/**
 * @brief Integrated VR SLAM system combining all components
 * 
 * This class integrates all components of the VR SLAM system:
 * - Multi-camera rig
 * - TPU feature extraction
 * - Zero-copy frame provider
 * - VR motion model
 * - IMU integration
 */
class VRSLAMSystem
{
public:
    /**
     * @brief System configuration
     */
    struct Config {
        std::string vocabulary_path;           ///< Path to ORB vocabulary
        std::string settings_path;             ///< Path to settings file
        std::string calibration_path;          ///< Path to camera calibration file
        std::string tpu_model_path;            ///< Path to TPU model file
        bool use_imu;                          ///< Whether to use IMU data
        bool enable_mapping;                   ///< Whether to enable mapping
        bool enable_loop_closing;              ///< Whether to enable loop closing
        VRMotionModel::InteractionMode interaction_mode; ///< VR interaction mode
        double prediction_horizon_ms;          ///< Motion prediction horizon
        int num_threads;                       ///< Number of processing threads
        bool verbose;                          ///< Whether to print verbose output
    };
    
    /**
     * @brief System status
     */
    enum class Status {
        UNINITIALIZED,    ///< System not initialized
        INITIALIZING,     ///< System initializing
        TRACKING,         ///< System tracking
        LOST,             ///< Tracking lost
        RELOCALIZATION,   ///< Attempting relocalization
        SHUTDOWN          ///< System shut down
    };
    
    /**
     * @brief Performance metrics
     */
    struct PerformanceMetrics {
        double average_tracking_time_ms;       ///< Average tracking time
        double average_feature_extraction_time_ms; ///< Average feature extraction time
        double average_frame_acquisition_time_ms;  ///< Average frame acquisition time
        double average_total_latency_ms;       ///< Average total latency
        double average_fps;                    ///< Average frames per second
        int frames_processed;                  ///< Number of frames processed
        int tracking_lost_count;               ///< Number of times tracking was lost
        double tracking_percentage;            ///< Percentage of time tracking was successful
    };
    
    /**
     * @brief Constructor
     * 
     * @param config System configuration
     */
    explicit VRSLAMSystem(const Config& config);
    
    /**
     * @brief Destructor
     */
    ~VRSLAMSystem();
    
    /**
     * @brief Initialize the system
     * 
     * @return True if initialization was successful
     */
    bool Initialize();
    
    /**
     * @brief Start the system
     * 
     * @return True if system was started successfully
     */
    bool Start();
    
    /**
     * @brief Stop the system
     * 
     * @return True if system was stopped successfully
     */
    bool Stop();
    
    /**
     * @brief Shutdown the system
     */
    void Shutdown();
    
    /**
     * @brief Get current system status
     * 
     * @return Current status
     */
    Status GetStatus() const;
    
    /**
     * @brief Get current camera pose
     * 
     * @return Current camera pose
     */
    Sophus::SE3f GetCurrentPose() const;
    
    /**
     * @brief Get predicted camera pose
     * 
     * @param prediction_time_ms Time in the future to predict (milliseconds)
     * @return Predicted camera pose
     */
    Sophus::SE3f GetPredictedPose(double prediction_time_ms) const;
    
    /**
     * @brief Get performance metrics
     * 
     * @return Performance metrics
     */
    PerformanceMetrics GetPerformanceMetrics() const;
    
    /**
     * @brief Save map to file
     * 
     * @param filename Filename to save map to
     * @return True if map was saved successfully
     */
    bool SaveMap(const std::string& filename) const;
    
    /**
     * @brief Load map from file
     * 
     * @param filename Filename to load map from
     * @return True if map was loaded successfully
     */
    bool LoadMap(const std::string& filename);
    
    /**
     * @brief Reset the system
     * 
     * @return True if system was reset successfully
     */
    bool Reset();
    
    /**
     * @brief Set VR interaction mode
     * 
     * @param mode Interaction mode
     */
    void SetInteractionMode(VRMotionModel::InteractionMode mode);
    
    /**
     * @brief Get VR interaction mode
     * 
     * @return Current interaction mode
     */
    VRMotionModel::InteractionMode GetInteractionMode() const;
    
    /**
     * @brief Set prediction horizon
     * 
     * @param prediction_horizon_ms Prediction horizon in milliseconds
     */
    void SetPredictionHorizon(double prediction_horizon_ms);
    
    /**
     * @brief Get prediction horizon
     * 
     * @return Current prediction horizon in milliseconds
     */
    double GetPredictionHorizon() const;
    
    /**
     * @brief Process a single frame
     * 
     * This method is for testing and debugging purposes.
     * In normal operation, frames are processed automatically.
     * 
     * @param images Vector of images from all cameras
     * @param timestamp Timestamp of the frame
     * @return True if frame was processed successfully
     */
    bool ProcessFrame(const std::vector<cv::Mat>& images, double timestamp);
    
    /**
     * @brief Process IMU measurement
     * 
     * This method is for testing and debugging purposes.
     * In normal operation, IMU measurements are processed automatically.
     * 
     * @param gyro Gyroscope measurement (rad/s)
     * @param accel Accelerometer measurement (m/s^2)
     * @param timestamp Timestamp of the measurement
     * @return True if measurement was processed successfully
     */
    bool ProcessIMU(const Eigen::Vector3f& gyro, const Eigen::Vector3f& accel, double timestamp);
    
private:
    // Configuration
    Config config_;
    
    // Components
    std::unique_ptr<MultiCameraRig> camera_rig_;
    std::unique_ptr<ZeroCopyFrameProvider> frame_provider_;
    std::unique_ptr<TPUFeatureExtractor> feature_extractor_;
    std::unique_ptr<TPUZeroCopyIntegration> tpu_integration_;
    std::unique_ptr<MultiCameraTracking> tracking_;
    std::unique_ptr<VRMotionModel> motion_model_;
    std::unique_ptr<BNO085Interface> imu_interface_;
    
    // System state
    std::atomic<Status> status_;
    std::mutex pose_mutex_;
    Sophus::SE3f current_pose_;
    
    // Performance monitoring
    PerformanceMetrics metrics_;
    std::mutex metrics_mutex_;
    
    // Processing thread
    std::thread processing_thread_;
    std::atomic<bool> running_;
    std::condition_variable processing_cv_;
    std::mutex processing_mutex_;
    
    // Helper methods
    void processingLoop();
    bool initializeComponents();
    bool processFrameInternal(const std::vector<cv::Mat>& images, double timestamp);
    bool processIMUInternal(const Eigen::Vector3f& gyro, const Eigen::Vector3f& accel, double timestamp);
    void updatePerformanceMetrics(double tracking_time, double feature_time, double acquisition_time);
};

} // namespace ORB_SLAM3

#endif // VR_SLAM_SYSTEM_HPP

#include "include/vr_slam_system.hpp"
#include <chrono>
#include <iostream>
#include <fstream>
#include <algorithm>

namespace ORB_SLAM3
{

VRSLAMSystem::VRSLAMSystem(const Config& config)
    : config_(config), status_(Status::UNINITIALIZED), running_(false)
{
    // Initialize performance metrics
    metrics_.average_tracking_time_ms = 0.0;
    metrics_.average_feature_extraction_time_ms = 0.0;
    metrics_.average_frame_acquisition_time_ms = 0.0;
    metrics_.average_total_latency_ms = 0.0;
    metrics_.average_fps = 0.0;
    metrics_.frames_processed = 0;
    metrics_.tracking_lost_count = 0;
    metrics_.tracking_percentage = 100.0;
}

VRSLAMSystem::~VRSLAMSystem()
{
    Shutdown();
}

bool VRSLAMSystem::Initialize()
{
    if (status_ != Status::UNINITIALIZED) {
        std::cerr << "System already initialized" << std::endl;
        return false;
    }
    
    status_ = Status::INITIALIZING;
    
    // Initialize components
    if (!initializeComponents()) {
        std::cerr << "Failed to initialize components" << std::endl;
        status_ = Status::UNINITIALIZED;
        return false;
    }
    
    status_ = Status::TRACKING;
    return true;
}

bool VRSLAMSystem::Start()
{
    if (status_ == Status::UNINITIALIZED) {
        std::cerr << "System not initialized" << std::endl;
        return false;
    }
    
    if (running_) {
        std::cerr << "System already running" << std::endl;
        return false;
    }
    
    // Start frame provider
    if (frame_provider_) {
        if (!frame_provider_->StartAcquisition()) {
            std::cerr << "Failed to start frame acquisition" << std::endl;
            return false;
        }
    }
    
    // Start IMU interface
    if (config_.use_imu && imu_interface_) {
        if (!imu_interface_->Start()) {
            std::cerr << "Failed to start IMU interface" << std::endl;
            return false;
        }
    }
    
    // Start processing thread
    running_ = true;
    processing_thread_ = std::thread(&VRSLAMSystem::processingLoop, this);
    
    return true;
}

bool VRSLAMSystem::Stop()
{
    if (!running_) {
        std::cerr << "System not running" << std::endl;
        return false;
    }
    
    // Stop processing thread
    running_ = false;
    processing_cv_.notify_all();
    if (processing_thread_.joinable()) {
        processing_thread_.join();
    }
    
    // Stop frame provider
    if (frame_provider_) {
        if (!frame_provider_->StopAcquisition()) {
            std::cerr << "Failed to stop frame acquisition" << std::endl;
            return false;
        }
    }
    
    // Stop IMU interface
    if (config_.use_imu && imu_interface_) {
        if (!imu_interface_->Stop()) {
            std::cerr << "Failed to stop IMU interface" << std::endl;
            return false;
        }
    }
    
    return true;
}

void VRSLAMSystem::Shutdown()
{
    // Stop if running
    if (running_) {
        Stop();
    }
    
    // Reset components
    imu_interface_.reset();
    motion_model_.reset();
    tracking_.reset();
    tpu_integration_.reset();
    feature_extractor_.reset();
    frame_provider_.reset();
    camera_rig_.reset();
    
    status_ = Status::SHUTDOWN;
}

VRSLAMSystem::Status VRSLAMSystem::GetStatus() const
{
    return status_;
}

Sophus::SE3f VRSLAMSystem::GetCurrentPose() const
{
    std::lock_guard<std::mutex> lock(pose_mutex_);
    return current_pose_;
}

Sophus::SE3f VRSLAMSystem::GetPredictedPose(double prediction_time_ms) const
{
    if (!motion_model_) {
        return GetCurrentPose();
    }
    
    return motion_model_->PredictPose(prediction_time_ms);
}

VRSLAMSystem::PerformanceMetrics VRSLAMSystem::GetPerformanceMetrics() const
{
    std::lock_guard<std::mutex> lock(metrics_mutex_);
    return metrics_;
}

bool VRSLAMSystem::SaveMap(const std::string& filename) const
{
    if (!tracking_) {
        std::cerr << "Tracking not initialized" << std::endl;
        return false;
    }
    
    // Save map using ORB-SLAM3 Atlas
    return tracking_->SaveMap(filename);
}

bool VRSLAMSystem::LoadMap(const std::string& filename)
{
    if (!tracking_) {
        std::cerr << "Tracking not initialized" << std::endl;
        return false;
    }
    
    // Load map using ORB-SLAM3 Atlas
    return tracking_->LoadMap(filename);
}

bool VRSLAMSystem::Reset()
{
    if (status_ == Status::UNINITIALIZED || status_ == Status::SHUTDOWN) {
        std::cerr << "System not initialized" << std::endl;
        return false;
    }
    
    // Stop if running
    bool was_running = running_;
    if (was_running) {
        Stop();
    }
    
    // Reset components
    if (motion_model_) {
        motion_model_->Reset();
    }
    
    if (tracking_) {
        tracking_->Reset();
    }
    
    // Reset metrics
    {
        std::lock_guard<std::mutex> lock(metrics_mutex_);
        metrics_.average_tracking_time_ms = 0.0;
        metrics_.average_feature_extraction_time_ms = 0.0;
        metrics_.average_frame_acquisition_time_ms = 0.0;
        metrics_.average_total_latency_ms = 0.0;
        metrics_.average_fps = 0.0;
        metrics_.frames_processed = 0;
        metrics_.tracking_lost_count = 0;
        metrics_.tracking_percentage = 100.0;
    }
    
    // Restart if was running
    if (was_running) {
        Start();
    }
    
    status_ = Status::INITIALIZING;
    return true;
}

void VRSLAMSystem::SetInteractionMode(VRMotionModel::InteractionMode mode)
{
    if (motion_model_) {
        motion_model_->SetInteractionMode(mode);
    }
}

VRMotionModel::InteractionMode VRSLAMSystem::GetInteractionMode() const
{
    if (motion_model_) {
        return motion_model_->GetInteractionMode();
    }
    return VRMotionModel::InteractionMode::STANDING;  // Default
}

void VRSLAMSystem::SetPredictionHorizon(double prediction_horizon_ms)
{
    if (motion_model_) {
        VRMotionModel::PredictionConfig config = motion_model_->GetConfig();
        config.prediction_horizon_ms = prediction_horizon_ms;
        motion_model_->SetConfig(config);
    }
}

double VRSLAMSystem::GetPredictionHorizon() const
{
    if (motion_model_) {
        return motion_model_->GetConfig().prediction_horizon_ms;
    }
    return 0.0;
}

bool VRSLAMSystem::ProcessFrame(const std::vector<cv::Mat>& images, double timestamp)
{
    return processFrameInternal(images, timestamp);
}

bool VRSLAMSystem::ProcessIMU(const Eigen::Vector3f& gyro, const Eigen::Vector3f& accel, double timestamp)
{
    return processIMUInternal(gyro, accel, timestamp);
}

bool VRSLAMSystem::initializeComponents()
{
    try {
        // Initialize camera rig
        camera_rig_ = std::make_unique<MultiCameraRig>(0);  // Reference camera ID = 0
        
        // Load camera calibration
        if (!config_.calibration_path.empty()) {
            if (!camera_rig_->LoadCalibration(config_.calibration_path)) {
                std::cerr << "Failed to load camera calibration from " << config_.calibration_path << std::endl;
                return false;
            }
        } else {
            std::cerr << "No camera calibration file specified" << std::endl;
            return false;
        }
        
        // Initialize frame provider
        frame_provider_ = std::make_unique<ZeroCopyFrameProvider>();
        
        // Convert camera rig to frame provider configuration
        std::vector<ZeroCopyFrameProvider::CameraConfig> camera_configs;
        for (const auto& camera : camera_rig_->GetAllCameras()) {
            ZeroCopyFrameProvider::CameraConfig config;
            config.camera_id = camera.id;
            config.width = camera.width;
            config.height = camera.height;
            config.format = ZeroCopyFrameProvider::PixelFormat::GRAY8;
            config.fps = camera.fps;
            camera_configs.push_back(config);
        }
        
        if (!frame_provider_->Initialize(camera_configs)) {
            std::cerr << "Failed to initialize frame provider" << std::endl;
            return false;
        }
        
        // Initialize TPU feature extractor
        feature_extractor_ = std::make_unique<TPUFeatureExtractor>();
        if (!feature_extractor_->Initialize(config_.tpu_model_path, 640, 480)) {
            std::cerr << "Failed to initialize TPU feature extractor" << std::endl;
            return false;
        }
        
        // Initialize TPU-ZeroCopy integration
        TPUZeroCopyIntegration::Config tpu_integration_config;
        tpu_integration_config.num_threads = config_.num_threads;
        tpu_integration_config.enable_direct_dma = true;
        tpu_integration_config.enable_performance_tracking = true;
        
        tpu_integration_ = std::make_unique<TPUZeroCopyIntegration>(
            frame_provider_, feature_extractor_, tpu_integration_config);
        
        if (!tpu_integration_->Initialize(camera_configs, config_.tpu_model_path)) {
            std::cerr << "Failed to initialize TPU-ZeroCopy integration" << std::endl;
            return false;
        }
        
        // Initialize ORB-SLAM3 system components
        // Note: In a real implementation, these would be actual ORB-SLAM3 components
        // For this implementation, we'll use our custom MultiCameraTracking
        
        // Initialize motion model
        VRMotionModel::PredictionConfig motion_config;
        motion_config.prediction_horizon_ms = config_.prediction_horizon_ms;
        motion_config.use_imu_for_prediction = config_.use_imu;
        motion_config.adaptive_prediction = true;
        
        motion_model_ = std::make_unique<VRMotionModel>(motion_config);
        motion_model_->SetInteractionMode(config_.interaction_mode);
        
        // Initialize IMU interface if enabled
        if (config_.use_imu) {
            BNO085Interface::Config imu_config;
            imu_config.interface_type = BNO085Interface::InterfaceType::I2C;
            imu_config.operation_mode = BNO085Interface::OperationMode::VR;
            imu_config.sample_rate = 100.0f;
            
            imu_interface_ = std::make_unique<BNO085Interface>(imu_config);
            if (!imu_interface_->Initialize()) {
                std::cerr << "Failed to initialize IMU interface" << std::endl;
                return false;
            }
        }
        
        // Initialize multi-camera tracking
        // Note: In a real implementation, this would initialize the actual ORB-SLAM3 system
        // For this implementation, we'll use placeholder objects
        
        MultiCameraTracking::Config tracking_config;
        tracking_config.enable_cross_camera_matching = true;
        tracking_config.use_spherical_model = true;
        tracking_config.parallel_feature_extraction = true;
        
        tracking_ = std::make_unique<MultiCameraTracking>(
            nullptr,  // System* (not needed for this implementation)
            nullptr,  // ORBVocabulary* (not needed for this implementation)
            nullptr,  // FrameDrawer* (not needed for this implementation)
            nullptr,  // MapDrawer* (not needed for this implementation)
            nullptr,  // Atlas* (not needed for this implementation)
            nullptr,  // KeyFrameDatabase* (not needed for this implementation)
            config_.settings_path,
            System::MONOCULAR,
            *camera_rig_,
            tracking_config
        );
        
        return true;
    } catch (const std::exception& e) {
        std::cerr << "Exception during component initialization: " << e.what() << std::endl;
        return false;
    }
}

void VRSLAMSystem::processingLoop()
{
    using namespace std::chrono;
    
    steady_clock::time_point last_frame_time = steady_clock::now();
    
    while (running_) {
        // Wait for next frame time or notification
        {
            std::unique_lock<std::mutex> lock(processing_mutex_);
            processing_cv_.wait_for(lock, milliseconds(1));
        }
        
        // Check if we should continue running
        if (!running_) {
            break;
        }
        
        // Get synchronized frames from all cameras
        std::vector<ZeroCopyFrameProvider::FrameBuffer> frame_buffers;
        auto acquisition_start = steady_clock::now();
        bool got_frames = frame_provider_->GetSynchronizedFrames(frame_buffers);
        auto acquisition_end = steady_clock::now();
        
        if (!got_frames) {
            continue;
        }
        
        // Extract features from all frames
        std::vector<std::vector<cv::KeyPoint>> all_keypoints;
        std::vector<cv::Mat> all_descriptors;
        auto feature_start = steady_clock::now();
        bool extracted = tpu_integration_->ProcessSynchronizedFrames(all_keypoints, all_descriptors);
        auto feature_end = steady_clock::now();
        
        if (!extracted) {
            continue;
        }
        
        // Convert frame buffers to OpenCV images for tracking
        std::vector<cv::Mat> images;
        for (const auto& buffer : frame_buffers) {
            cv::Mat image(buffer.height, buffer.width, CV_8UC1);
            if (buffer.buffer_type == ZeroCopyFrameProvider::BufferType::CPU && buffer.data) {
                memcpy(image.data, buffer.data, buffer.size);
            } else {
                // For DMA buffers, we would need to map them to CPU memory
                // This is a simplified implementation
                image = cv::Mat::zeros(buffer.height, buffer.width, CV_8UC1);
            }
            images.push_back(image);
        }
        
        // Track with multi-camera system
        auto tracking_start = steady_clock::now();
        double timestamp = duration_cast<milliseconds>(tracking_start.time_since_epoch()).count() / 1000.0;
        Sophus::SE3f pose = tracking_->GrabMultiCameraImages(images, timestamp, all_keypoints, all_descriptors);
        auto tracking_end = steady_clock::now();
        
        // Update current pose
        {
            std::lock_guard<std::mutex> lock(pose_mutex_);
            current_pose_ = pose;
        }
        
        // Update motion model
        motion_model_->AddPose(pose, timestamp);
        
        // Update status based on tracking result
        if (tracking_->GetTrackingState() == TrackingState::OK) {
            status_ = Status::TRACKING;
        } else if (tracking_->GetTrackingState() == TrackingState::LOST) {
            status_ = Status::LOST;
            metrics_.tracking_lost_count++;
        } else if (tracking_->GetTrackingState() == TrackingState::RELOCALIZATION) {
            status_ = Status::RELOCALIZATION;
        }
        
        // Calculate timing metrics
        double acquisition_time = duration_cast<microseconds>(acquisition_end - acquisition_start).count() / 1000.0;
        double feature_time = duration_cast<microseconds>(feature_end - feature_start).count() / 1000.0;
        double tracking_time = duration_cast<microseconds>(tracking_end - tracking_start).count() / 1000.0;
        double total_time = duration_cast<microseconds>(tracking_end - acquisition_start).count() / 1000.0;
        
        // Update performance metrics
        updatePerformanceMetrics(tracking_time, feature_time, acquisition_time);
        
        // Calculate FPS
        auto current_time = steady_clock::now();
        double frame_time = duration_cast<microseconds>(current_time - last_frame_time).count() / 1000.0;
        last_frame_time = current_time;
        
        // Update FPS in metrics
        {
            std::lock_guard<std::mutex> lock(metrics_mutex_);
            metrics_.average_fps = 1000.0 / frame_time;
        }
        
        // Release frame buffers
        for (auto& buffer : frame_buffers) {
            frame_provider_->ReleaseFrame(buffer);
        }
    }
}

bool VRSLAMSystem::processFrameInternal(const std::vector<cv::Mat>& images, double timestamp)
{
    if (status_ == Status::UNINITIALIZED || status_ == Status::SHUTDOWN) {
        std::cerr << "System not initialized" << std::endl;
        return false;
    }
    
    // Extract features from all frames
    std::vector<std::vector<cv::KeyPoint>> all_keypoints(images.size());
    std::vector<cv::Mat> all_descriptors(images.size());
    
    auto feature_start = std::chrono::steady_clock::now();
    
    for (size_t i = 0; i < images.size(); ++i) {
        if (!feature_extractor_->Extract(images[i], all_keypoints[i], all_descriptors[i])) {
            std::cerr << "Failed to extract features from image " << i << std::endl;
            return false;
        }
    }
    
    auto feature_end = std::chrono::steady_clock::now();
    
    // Track with multi-camera system
    auto tracking_start = std::chrono::steady_clock::now();
    Sophus::SE3f pose = tracking_->GrabMultiCameraImages(images, timestamp, all_keypoints, all_descriptors);
    auto tracking_end = std::chrono::steady_clock::now();
    
    // Update current pose
    {
        std::lock_guard<std::mutex> lock(pose_mutex_);
        current_pose_ = pose;
    }
    
    // Update motion model
    motion_model_->AddPose(pose, timestamp);
    
    // Update status based on tracking result
    if (tracking_->GetTrackingState() == TrackingState::OK) {
        status_ = Status::TRACKING;
    } else if (tracking_->GetTrackingState() == TrackingState::LOST) {
        status_ = Status::LOST;
        metrics_.tracking_lost_count++;
    } else if (tracking_->GetTrackingState() == TrackingState::RELOCALIZATION) {
        status_ = Status::RELOCALIZATION;
    }
    
    // Calculate timing metrics
    double feature_time = std::chrono::duration_cast<std::chrono::microseconds>(feature_end - feature_start).count() / 1000.0;
    double tracking_time = std::chrono::duration_cast<std::chrono::microseconds>(tracking_end - tracking_start).count() / 1000.0;
    
    // Update performance metrics
    updatePerformanceMetrics(tracking_time, feature_time, 0.0);
    
    return true;
}

bool VRSLAMSystem::processIMUInternal(const Eigen::Vector3f& gyro, const Eigen::Vector3f& accel, double timestamp)
{
    if (status_ == Status::UNINITIALIZED || status_ == Status::SHUTDOWN) {
        std::cerr << "System not initialized" << std::endl;
        return false;
    }
    
    if (!config_.use_imu) {
        std::cerr << "IMU not enabled" << std::endl;
        return false;
    }
    
    // Update motion model with IMU data
    motion_model_->AddIMU(gyro, accel, timestamp);
    
    // Update tracking with IMU data
    tracking_->ProcessIMU(gyro, accel, timestamp);
    
    return true;
}

void VRSLAMSystem::updatePerformanceMetrics(double tracking_time, double feature_time, double acquisition_time)
{
    std::lock_guard<std::mutex> lock(metrics_mutex_);
    
    // Update running averages
    if (metrics_.frames_processed == 0) {
        metrics_.average_tracking_time_ms = tracking_time;
        metrics_.average_feature_extraction_time_ms = feature_time;
        metrics_.average_frame_acquisition_time_ms = acquisition_time;
        metrics_.average_total_latency_ms = tracking_time + feature_time + acquisition_time;
    } else {
        // Exponential moving average with alpha = 0.1
        const double alpha = 0.1;
        metrics_.average_tracking_time_ms = (1.0 - alpha) * metrics_.average_tracking_time_ms + alpha * tracking_time;
        metrics_.average_feature_extraction_time_ms = (1.0 - alpha) * metrics_.average_feature_extraction_time_ms + alpha * feature_time;
        metrics_.average_frame_acquisition_time_ms = (1.0 - alpha) * metrics_.average_frame_acquisition_time_ms + alpha * acquisition_time;
        metrics_.average_total_latency_ms = (1.0 - alpha) * metrics_.average_total_latency_ms + alpha * (tracking_time + feature_time + acquisition_time);
    }
    
    // Update frame count
    metrics_.frames_processed++;
    
    // Update tracking percentage
    metrics_.tracking_percentage = 100.0 * (1.0 - static_cast<double>(metrics_.tracking_lost_count) / metrics_.frames_processed);
}

} // namespace ORB_SLAM3

#include "include/visual_inertial_fusion.hpp"

#include <chrono>
#include <algorithm>
#include <cmath>
#include <fstream>
#include <iomanip>
#include <opencv2/core/eigen.hpp>

namespace ORB_SLAM3
{

VisualInertialFusion::VisualInertialFusion(
    const Config& config,
    std::shared_ptr<BNO085Interface> imu_interface,
    std::shared_ptr<MultiCameraTracking> tracking,
    std::shared_ptr<VRMotionModel> motion_model)
    : mConfig(config),
      mIMUInterface(imu_interface),
      mTracking(tracking),
      mMotionModel(motion_model),
      mState(State::UNINITIALIZED),
      mCurrentVelocity(Eigen::Vector3f::Zero()),
      mCurrentAcceleration(Eigen::Vector3f::Zero()),
      mCurrentAngularVelocity(Eigen::Vector3f::Zero()),
      mGravityDirection(0, 0, -1),  // Default gravity direction (down)
      mLastIMUTimestamp(0),
      mLastVisualTimestamp(0),
      mVisualTrackingGood(false),
      mTrackingLossCount(0),
      mInitProgress(0.0f),
      mInitStartTime(0),
      mGravityInitialized(false),
      mRunning(false)
{
    // Initialize IMU preintegration with default bias
    IMU::Bias initial_bias;
    IMU::Calib calib = mIMUInterface->GetCalibration();
    mpImuPreintegrated = new IMU::Preintegrated(initial_bias, calib);
}

VisualInertialFusion::~VisualInertialFusion()
{
    Stop();
    if (mpImuPreintegrated)
        delete mpImuPreintegrated;
}

bool VisualInertialFusion::Initialize()
{
    std::lock_guard<std::mutex> lock(mStateMutex);
    
    // Reset state variables
    mState = State::UNINITIALIZED;
    mCurrentPose = Sophus::SE3<float>();
    mCurrentVelocity = Eigen::Vector3f::Zero();
    mCurrentAcceleration = Eigen::Vector3f::Zero();
    mCurrentAngularVelocity = Eigen::Vector3f::Zero();
    mGravityDirection = Eigen::Vector3f(0, 0, -1);
    mLastIMUTimestamp = 0;
    mLastVisualTimestamp = 0;
    mVisualTrackingGood = false;
    mTrackingLossCount = 0;
    mInitProgress = 0.0f;
    mInitStartTime = 0;
    mGravityInitialized = false;
    
    // Clear IMU queue
    std::queue<IMU::Point> empty;
    std::swap(mIMUQueue, empty);
    
    // Reset IMU preintegration
    IMU::Bias initial_bias;
    IMU::Calib calib = mIMUInterface->GetCalibration();
    if (mpImuPreintegrated)
        delete mpImuPreintegrated;
    mpImuPreintegrated = new IMU::Preintegrated(initial_bias, calib);
    
    // Reset performance metrics
    mMetrics = PerformanceMetrics();
    
    return true;
}

bool VisualInertialFusion::Start()
{
    if (mRunning)
        return false;
    
    mRunning = true;
    mProcessingThread = std::thread(&VisualInertialFusion::ProcessingThreadFunction, this);
    
    return true;
}

void VisualInertialFusion::Stop()
{
    if (!mRunning)
        return;
    
    mRunning = false;
    mProcessingCondition.notify_all();
    
    if (mProcessingThread.joinable())
        mProcessingThread.join();
}

bool VisualInertialFusion::Reset()
{
    Stop();
    bool result = Initialize();
    if (result)
        Start();
    return result;
}

VisualInertialFusion::State VisualInertialFusion::GetState() const
{
    return mState;
}

Sophus::SE3<float> VisualInertialFusion::GetCurrentPose() const
{
    std::lock_guard<std::mutex> lock(mPoseMutex);
    return mCurrentPose;
}

Sophus::SE3<float> VisualInertialFusion::GetPredictedPose(double prediction_time_ms) const
{
    // Use the VR motion model for prediction
    return mMotionModel->PredictPose(prediction_time_ms);
}

Eigen::Vector3f VisualInertialFusion::GetCurrentVelocity() const
{
    std::lock_guard<std::mutex> lock(mPoseMutex);
    return mCurrentVelocity;
}

Eigen::Vector3f VisualInertialFusion::GetCurrentAcceleration() const
{
    std::lock_guard<std::mutex> lock(mPoseMutex);
    return mCurrentAcceleration;
}

Eigen::Vector3f VisualInertialFusion::GetCurrentAngularVelocity() const
{
    std::lock_guard<std::mutex> lock(mPoseMutex);
    return mCurrentAngularVelocity;
}

IMU::Bias VisualInertialFusion::GetCurrentBias() const
{
    std::lock_guard<std::mutex> lock(mIMUMutex);
    return mCurrentBias;
}

Eigen::Vector3f VisualInertialFusion::GetGravityDirection() const
{
    std::lock_guard<std::mutex> lock(mPoseMutex);
    return mGravityDirection;
}

VisualInertialFusion::PerformanceMetrics VisualInertialFusion::GetPerformanceMetrics() const
{
    std::lock_guard<std::mutex> lock(mMetricsMutex);
    return mMetrics;
}

bool VisualInertialFusion::ProcessIMUMeasurements(const std::vector<IMU::Point>& measurements)
{
    if (measurements.empty())
        return false;
    
    std::lock_guard<std::mutex> lock(mIMUMutex);
    
    // Add measurements to the queue
    for (const auto& measurement : measurements)
    {
        mIMUQueue.push(measurement);
        
        // Update last IMU timestamp
        if (measurement.t > mLastIMUTimestamp)
            mLastIMUTimestamp = measurement.t;
    }
    
    // Notify processing thread
    mProcessingCondition.notify_one();
    
    return true;
}

bool VisualInertialFusion::ProcessVisualTracking(
    const Sophus::SE3<float>& pose,
    double timestamp,
    const std::vector<std::vector<cv::KeyPoint>>& keypoints,
    const std::vector<std::vector<MapPoint*>>& map_points)
{
    std::lock_guard<std::mutex> lock(mVisualMutex);
    
    // Update visual tracking state
    mLastVisualTimestamp = timestamp;
    
    // Check if we have enough features for good tracking
    int total_features = 0;
    for (const auto& camera_keypoints : keypoints)
        total_features += camera_keypoints.size();
    
    mVisualTrackingGood = (total_features >= mConfig.init_min_features);
    
    // Notify processing thread
    mProcessingCondition.notify_one();
    
    return true;
}

void VisualInertialFusion::SetPredictionHorizon(double prediction_horizon_ms)
{
    mConfig.prediction_horizon_ms = prediction_horizon_ms;
    mMotionModel->SetPredictionHorizon(prediction_horizon_ms);
}

void VisualInertialFusion::SetVRInteractionMode(VRMotionModel::InteractionMode mode)
{
    mMotionModel->SetInteractionMode(mode);
}

bool VisualInertialFusion::IsInitialized() const
{
    return mState != State::UNINITIALIZED && mState != State::INITIALIZING;
}

bool VisualInertialFusion::IsTrackingGood() const
{
    return mState == State::TRACKING_NOMINAL || 
           mState == State::TRACKING_RAPID || 
           mState == State::TRACKING_VISUAL;
}

float VisualInertialFusion::GetInitializationProgress() const
{
    std::lock_guard<std::mutex> lock(mInitMutex);
    return mInitProgress;
}

float VisualInertialFusion::GetTrackingQuality() const
{
    if (mState == State::TRACKING_NOMINAL)
        return 1.0f;
    else if (mState == State::TRACKING_RAPID || mState == State::TRACKING_VISUAL)
        return 0.7f;
    else if (mState == State::RELOCALIZATION)
        return 0.3f;
    else if (mState == State::LOST)
        return 0.0f;
    else
        return 0.5f;  // Initializing or other states
}

bool VisualInertialFusion::SaveState(const std::string& filename) const
{
    try
    {
        std::ofstream file(filename, std::ios::binary);
        if (!file.is_open())
            return false;
        
        // Save current pose
        Eigen::Matrix4f pose_matrix = mCurrentPose.matrix();
        file.write(reinterpret_cast<char*>(pose_matrix.data()), sizeof(float) * 16);
        
        // Save velocity, acceleration, angular velocity
        file.write(reinterpret_cast<const char*>(mCurrentVelocity.data()), sizeof(float) * 3);
        file.write(reinterpret_cast<const char*>(mCurrentAcceleration.data()), sizeof(float) * 3);
        file.write(reinterpret_cast<const char*>(mCurrentAngularVelocity.data()), sizeof(float) * 3);
        
        // Save gravity direction
        file.write(reinterpret_cast<const char*>(mGravityDirection.data()), sizeof(float) * 3);
        
        // Save IMU bias
        file.write(reinterpret_cast<const char*>(&mCurrentBias.bax), sizeof(float));
        file.write(reinterpret_cast<const char*>(&mCurrentBias.bay), sizeof(float));
        file.write(reinterpret_cast<const char*>(&mCurrentBias.baz), sizeof(float));
        file.write(reinterpret_cast<const char*>(&mCurrentBias.bwx), sizeof(float));
        file.write(reinterpret_cast<const char*>(&mCurrentBias.bwy), sizeof(float));
        file.write(reinterpret_cast<const char*>(&mCurrentBias.bwz), sizeof(float));
        
        file.close();
        return true;
    }
    catch (const std::exception& e)
    {
        return false;
    }
}

bool VisualInertialFusion::LoadState(const std::string& filename)
{
    try
    {
        std::ifstream file(filename, std::ios::binary);
        if (!file.is_open())
            return false;
        
        // Load current pose
        Eigen::Matrix4f pose_matrix;
        file.read(reinterpret_cast<char*>(pose_matrix.data()), sizeof(float) * 16);
        
        // Load velocity, acceleration, angular velocity
        Eigen::Vector3f velocity, acceleration, angular_velocity;
        file.read(reinterpret_cast<char*>(velocity.data()), sizeof(float) * 3);
        file.read(reinterpret_cast<char*>(acceleration.data()), sizeof(float) * 3);
        file.read(reinterpret_cast<char*>(angular_velocity.data()), sizeof(float) * 3);
        
        // Load gravity direction
        Eigen::Vector3f gravity;
        file.read(reinterpret_cast<char*>(gravity.data()), sizeof(float) * 3);
        
        // Load IMU bias
        IMU::Bias bias;
        file.read(reinterpret_cast<char*>(&bias.bax), sizeof(float));
        file.read(reinterpret_cast<char*>(&bias.bay), sizeof(float));
        file.read(reinterpret_cast<char*>(&bias.baz), sizeof(float));
        file.read(reinterpret_cast<char*>(&bias.bwx), sizeof(float));
        file.read(reinterpret_cast<char*>(&bias.bwy), sizeof(float));
        file.read(reinterpret_cast<char*>(&bias.bwz), sizeof(float));
        
        file.close();
        
        // Update state with loaded values
        {
            std::lock_guard<std::mutex> lock_pose(mPoseMutex);
            mCurrentPose = Sophus::SE3<float>(pose_matrix);
            mCurrentVelocity = velocity;
            mCurrentAcceleration = acceleration;
            mCurrentAngularVelocity = angular_velocity;
            mGravityDirection = gravity;
        }
        
        {
            std::lock_guard<std::mutex> lock_imu(mIMUMutex);
            mCurrentBias = bias;
            
            // Update preintegration with loaded bias
            if (mpImuPreintegrated)
            {
                delete mpImuPreintegrated;
                mpImuPreintegrated = new IMU::Preintegrated(bias, mIMUInterface->GetCalibration());
            }
        }
        
        // Set state to tracking if we were uninitialized
        if (mState == State::UNINITIALIZED)
        {
            mState = State::TRACKING_NOMINAL;
            mGravityInitialized = true;
        }
        
        return true;
    }
    catch (const std::exception& e)
    {
        return false;
    }
}

void VisualInertialFusion::ProcessingThreadFunction()
{
    while (mRunning)
    {
        // Wait for new data or timeout
        std::unique_lock<std::mutex> lock(mProcessingMutex);
        mProcessingCondition.wait_for(lock, std::chrono::milliseconds(10));
        
        if (!mRunning)
            break;
        
        // Start timing for performance metrics
        auto start_time = std::chrono::high_resolution_clock::now();
        
        // Process based on current state
        switch (mState)
        {
            case State::UNINITIALIZED:
                if (InitializeSystem())
                    mState = State::INITIALIZING;
                break;
                
            case State::INITIALIZING:
                if (InitializeGravity() && InitializeScaleAndVelocity())
                {
                    mState = State::TRACKING_NOMINAL;
                    
                    // Record initialization time for metrics
                    std::lock_guard<std::mutex> lock_metrics(mMetricsMutex);
                    mMetrics.average_init_time_s = std::chrono::duration_cast<std::chrono::milliseconds>(
                        std::chrono::high_resolution_clock::now() - 
                        std::chrono::high_resolution_clock::from_time_t(mInitStartTime)).count() / 1000.0;
                }
                break;
                
            case State::TRACKING_NOMINAL:
            case State::TRACKING_RAPID:
            case State::TRACKING_VISUAL:
                // Update motion state from visual and inertial data
                if (!UpdateMotionState())
                {
                    mState = State::LOST;
                    mTrackingLossCount++;
                }
                else
                {
                    // Check for rapid motion
                    if (DetectAndHandleRapidMotion())
                        mState = State::TRACKING_RAPID;
                    else if (mVisualTrackingGood)
                        mState = State::TRACKING_NOMINAL;
                    else
                        mState = State::TRACKING_VISUAL;
                    
                    // Update motion model for prediction
                    UpdateMotionModel();
                }
                break;
                
            case State::LOST:
                if (AttemptRelocalization())
                {
                    mState = State::TRACKING_NOMINAL;
                    
                    // Record relocalization metrics
                    std::lock_guard<std::mutex> lock_metrics(mMetricsMutex);
                    mMetrics.relocalization_count++;
                }
                else if (mConfig.use_imu_only_fallback)
                {
                    // Fall back to IMU-only tracking temporarily
                    PreintegrateIMU(mLastIMUTimestamp - 0.1, mLastIMUTimestamp);
                    mState = State::TRACKING_RAPID;
                }
                break;
                
            case State::RELOCALIZATION:
                if (AttemptRelocalization())
                {
                    mState = State::TRACKING_NOMINAL;
                    
                    // Record relocalization metrics
                    std::lock_guard<std::mutex> lock_metrics(mMetricsMutex);
                    mMetrics.relocalization_count++;
                }
                break;
        }
        
        // Update tracking state
        UpdateTrackingState();
        
        // Calculate processing time for metrics
        auto end_time = std::chrono::high_resolution_clock::now();
        double processing_time_ms = std::chrono::duration_cast<std::chrono::microseconds>(end_time - start_time).count() / 1000.0;
        
        // Update performance metrics
        UpdatePerformanceMetrics(processing_time_ms);
    }
}

bool VisualInertialFusion::InitializeSystem()
{
    // Check if we have both IMU and visual data
    if (mLastIMUTimestamp <= 0 || mLastVisualTimestamp <= 0)
        return false;
    
    // Record initialization start time
    mInitStartTime = std::chrono::high_resolution_clock::now().time_since_epoch().count() / 1000000000.0;
    
    // Initialize with identity pose
    {
        std::lock_guard<std::mutex> lock(mPoseMutex);
        mCurrentPose = Sophus::SE3<float>();
    }
    
    // Reset IMU preintegration
    {
        std::lock_guard<std::mutex> lock(mIMUMutex);
        if (mpImuPreintegrated)
            delete mpImuPreintegrated;
        mpImuPreintegrated = new IMU::Preintegrated(mCurrentBias, mIMUInterface->GetCalibration());
    }
    
    // Set initial progress
    {
        std::lock_guard<std::mutex> lock(mInitMutex);
        mInitProgress = 0.1f;
    }
    
    return true;
}

bool VisualInertialFusion::InitializeGravity()
{
    if (mGravityInitialized)
        return true;
    
    std::lock_guard<std::mutex> lock(mIMUMutex);
    
    // Need at least 100 IMU measurements for reliable gravity initialization
    if (mIMUQueue.size() < 100)
        return false;
    
    // Compute average acceleration as gravity direction
    Eigen::Vector3f avg_acc = Eigen::Vector3f::Zero();
    int count = 0;
    
    // Create a copy of the queue to avoid modifying it
    std::queue<IMU::Point> queue_copy = mIMUQueue;
    
    while (!queue_copy.empty() && count < 100)
    {
        const IMU::Point& imu_point = queue_copy.front();
        avg_acc += imu_point.a;
        queue_copy.pop();
        count++;
    }
    
    if (count > 0)
    {
        avg_acc /= count;
        
        // Normalize to gravity magnitude
        float norm = avg_acc.norm();
        if (norm > 0.1f)  // Ensure we have meaningful acceleration
        {
            avg_acc = avg_acc / norm * mConfig.gravity_magnitude;
            
            // Update gravity direction
            {
                std::lock_guard<std::mutex> lock_pose(mPoseMutex);
                mGravityDirection = avg_acc.normalized();
            }
            
            // Update initialization progress
            {
                std::lock_guard<std::mutex> lock_init(mInitMutex);
                mInitProgress = 0.5f;
            }
            
            mGravityInitialized = true;
            return true;
        }
    }
    
    return false;
}

bool VisualInertialFusion::InitializeScaleAndVelocity()
{
    if (!mGravityInitialized)
        return false;
    
    // Check if we have enough visual tracking data
    if (!mVisualTrackingGood)
        return false;
    
    // For VR applications, we can initialize with zero velocity
    // and refine it quickly during tracking
    {
        std::lock_guard<std::mutex> lock(mPoseMutex);
        mCurrentVelocity = Eigen::Vector3f::Zero();
    }
    
    // Update initialization progress
    {
        std::lock_guard<std::mutex> lock(mInitMutex);
        mInitProgress = 1.0f;
    }
    
    return true;
}

bool VisualInertialFusion::PerformVisualInertialBA(bool local_only)
{
    // This would be a complex implementation integrating with ORB-SLAM3's optimization
    // For now, we'll assume it's successful
    return true;
}

bool VisualInertialFusion::UpdateMotionState()
{
    // Get latest IMU data
    std::vector<IMU::Point> imu_data;
    {
        std::lock_guard<std::mutex> lock(mIMUMutex);
        
        // Process all available IMU data
        while (!mIMUQueue.empty())
        {
            imu_data.push_back(mIMUQueue.front());
            mIMUQueue.pop();
        }
    }
    
    if (imu_data.empty())
        return false;
    
    // Sort IMU data by timestamp
    std::sort(imu_data.begin(), imu_data.end(), 
              [](const IMU::Point& a, const IMU::Point& b) { return a.t < b.t; });
    
    // Preintegrate IMU measurements
    {
        std::lock_guard<std::mutex> lock(mIMUMutex);
        
        for (const auto& imu_point : imu_data)
        {
            mpImuPreintegrated->IntegrateNewMeasurement(imu_point.a, imu_point.w, 
                                                       imu_point.t - mLastIMUTimestamp);
            mLastIMUTimestamp = imu_point.t;
        }
    }
    
    // Update pose, velocity, and acceleration
    {
        std::lock_guard<std::mutex> lock(mPoseMutex);
        
        // Get preintegrated measurements
        Eigen::Matrix3f delta_R = mpImuPreintegrated->GetUpdatedDeltaRotation();
        Eigen::Vector3f delta_V = mpImuPreintegrated->GetUpdatedDeltaVelocity();
        Eigen::Vector3f delta_P = mpImuPreintegrated->GetUpdatedDeltaPosition();
        
        // Update orientation
        Eigen::Matrix3f R = mCurrentPose.rotationMatrix() * delta_R;
        
        // Update position
        Eigen::Vector3f P = mCurrentPose.translation() + 
                           mCurrentVelocity * mpImuPreintegrated->dT +
                           0.5f * (mCurrentPose.rotationMatrix() * delta_V + 
                                  Eigen::Vector3f(0, 0, -mConfig.gravity_magnitude) * mpImuPreintegrated->dT * mpImuPreintegrated->dT);
        
        // Update velocity
        mCurrentVelocity = mCurrentVelocity + 
                          mCurrentPose.rotationMatrix() * delta_V + 
                          Eigen::Vector3f(0, 0, -mConfig.gravity_magnitude) * mpImuPreintegrated->dT;
        
        // Update acceleration (from latest IMU measurement)
        if (!imu_data.empty())
        {
            const IMU::Point& latest_imu = imu_data.back();
            mCurrentAcceleration = mCurrentPose.rotationMatrix() * latest_imu.a + 
                                  Eigen::Vector3f(0, 0, -mConfig.gravity_magnitude);
            mCurrentAngularVelocity = latest_imu.w;
        }
        
        // Update pose
        mCurrentPose = Sophus::SE3<float>(R, P);
    }
    
    // Reset preintegration
    {
        std::lock_guard<std::mutex> lock(mIMUMutex);
        if (mpImuPreintegrated)
        {
            delete mpImuPreintegrated;
            mpImuPreintegrated = new IMU::Preintegrated(mCurrentBias, mIMUInterface->GetCalibration());
        }
    }
    
    return true;
}

bool VisualInertialFusion::AttemptRelocalization()
{
    // This would integrate with ORB-SLAM3's relocalization
    // For now, we'll return false to indicate relocalization is not yet implemented
    return false;
}

bool VisualInertialFusion::UpdateMotionModel()
{
    // Update the VR motion model with latest state
    {
        std::lock_guard<std::mutex> lock(mPoseMutex);
        mMotionModel->AddPose(mCurrentPose, mLastIMUTimestamp);
        mMotionModel->AddVelocity(mCurrentVelocity, mLastIMUTimestamp);
        mMotionModel->AddAcceleration(mCurrentAcceleration, mLastIMUTimestamp);
        mMotionModel->AddAngularVelocity(mCurrentAngularVelocity, mLastIMUTimestamp);
    }
    
    return true;
}

bool VisualInertialFusion::PreintegrateIMU(double start_time, double end_time)
{
    std::lock_guard<std::mutex> lock(mIMUMutex);
    
    // Get IMU measurements in the specified time range
    std::vector<IMU::Point> imu_data = mIMUInterface->GetMeasurementsInTimeRange(start_time, end_time);
    
    if (imu_data.empty())
        return false;
    
    // Sort by timestamp
    std::sort(imu_data.begin(), imu_data.end(), 
              [](const IMU::Point& a, const IMU::Point& b) { return a.t < b.t; });
    
    // Reset preintegration
    if (mpImuPreintegrated)
    {
        delete mpImuPreintegrated;
        mpImuPreintegrated = new IMU::Preintegrated(mCurrentBias, mIMUInterface->GetCalibration());
    }
    
    // Preintegrate measurements
    double prev_timestamp = start_time;
    for (const auto& imu_point : imu_data)
    {
        mpImuPreintegrated->IntegrateNewMeasurement(imu_point.a, imu_point.w, 
                                                   imu_point.t - prev_timestamp);
        prev_timestamp = imu_point.t;
    }
    
    return true;
}

bool VisualInertialFusion::DetectAndHandleRapidMotion()
{
    std::lock_guard<std::mutex> lock(mPoseMutex);
    
    // Check angular velocity magnitude
    float angular_velocity_magnitude = mCurrentAngularVelocity.norm();
    
    // Check linear acceleration magnitude (excluding gravity)
    float linear_acceleration_magnitude = mCurrentAcceleration.norm();
    
    // Thresholds for rapid motion detection
    const float angular_velocity_threshold = 1.5f;  // rad/s
    const float linear_acceleration_threshold = 5.0f;  // m/s^2
    
    return (angular_velocity_magnitude > angular_velocity_threshold) || 
           (linear_acceleration_magnitude > linear_acceleration_threshold);
}

void VisualInertialFusion::UpdateTrackingState()
{
    // Update tracking quality metrics based on current state
    std::lock_guard<std::mutex> lock(mMetricsMutex);
    
    // Update tracking percentage
    if (IsTrackingGood())
    {
        // Exponential moving average for tracking percentage
        mMetrics.tracking_percentage = 0.99 * mMetrics.tracking_percentage + 0.01 * 100.0;
    }
    else
    {
        mMetrics.tracking_percentage = 0.99 * mMetrics.tracking_percentage;
    }
}

void VisualInertialFusion::UpdatePerformanceMetrics(double fusion_time)
{
    std::lock_guard<std::mutex> lock(mMetricsMutex);
    
    // Exponential moving average for fusion time
    mMetrics.average_fusion_time_ms = 0.95 * mMetrics.average_fusion_time_ms + 0.05 * fusion_time;
    
    // Other metrics would be updated based on ground truth if available
}

} // namespace ORB_SLAM3

# Multi-Camera and VR Motion Model Adaptations for ORB-SLAM3

This document outlines the conceptual design for extending ORB-SLAM3 to support a 4-camera setup with spherical field of view (FoV) model and VR-specific motion model adaptations.

## 1. Multi-Camera Support

### 1.1 Camera Configuration Class

```cpp
class MultiCameraRig {
public:
    struct CameraInfo {
        int id;                         // Camera identifier
        cv::Mat K;                      // Intrinsic matrix
        cv::Mat distCoef;               // Distortion coefficients
        cv::Mat T_ref_cam;              // Transform from reference camera
        float fps;                      // Frame rate
        int width, height;              // Resolution
        std::string model;              // Camera model (pinhole, fisheye, etc.)
        float fov_horizontal;           // Horizontal field of view in degrees
        float fov_vertical;             // Vertical field of view in degrees
    };
    
    // Methods for camera rig management
    bool AddCamera(const CameraInfo& camera);
    bool RemoveCamera(int camera_id);
    CameraInfo GetCameraInfo(int camera_id) const;
    
    // Methods for camera rig calibration
    bool CalibrateRig(const std::vector<std::vector<cv::Mat>>& calibration_images);
    bool LoadCalibration(const std::string& filename);
    bool SaveCalibration(const std::string& filename) const;
    
    // Methods for spherical projection
    cv::Mat ProjectToSpherical(const std::vector<cv::Mat>& images);
    std::vector<cv::Point3f> ProjectPointsToSphere(const std::vector<cv::Point2f>& points, int camera_id);
    std::vector<cv::Point2f> ProjectSphericalPointsToCamera(const std::vector<cv::Point3f>& sphere_points, int camera_id);
};
```

### 1.2 Spherical Field of View Model

The spherical FoV model will unify the observations from all cameras into a single spherical coordinate system, allowing for seamless tracking across camera boundaries.

```cpp
class SphericalFovModel {
public:
    // Methods for spherical projection
    cv::Mat CreateSphericalMap(const MultiCameraRig& rig);
    cv::Mat BlendCameraImages(const std::vector<cv::Mat>& images, const MultiCameraRig& rig);
    
    // Methods for feature matching across cameras
    std::vector<cv::DMatch> MatchFeaturesAcrossCameras(
        const std::vector<std::vector<cv::KeyPoint>>& keypoints_per_camera,
        const std::vector<cv::Mat>& descriptors_per_camera,
        const MultiCameraRig& rig);
    
    // Methods for spherical coordinate conversion
    cv::Point3f CameraToSpherical(const cv::Point2f& point, int camera_id, const MultiCameraRig& rig);
    cv::Point2f SphericalToCamera(const cv::Point3f& sphere_point, int camera_id, const MultiCameraRig& rig);
};
```

### 1.3 Multi-Camera Tracking Extension

```cpp
class MultiCameraTracking : public ORB_SLAM3::Tracking {
public:
    // Constructor with multi-camera rig
    MultiCameraTracking(ORB_SLAM3::System* pSys, ORB_SLAM3::ORBVocabulary* pVoc,
                        ORB_SLAM3::FrameDrawer* pFrameDrawer, ORB_SLAM3::MapDrawer* pMapDrawer,
                        ORB_SLAM3::Atlas* pAtlas, ORB_SLAM3::KeyFrameDatabase* pKFDB,
                        const std::string& strSettingPath, const int sensor,
                        const MultiCameraRig& rig);
    
    // Override tracking methods to handle multiple cameras
    void Track();
    void TrackWithMultiCameras(const std::vector<cv::Mat>& images, const double& timestamp);
    
    // Methods for feature extraction and matching across cameras
    void ExtractFeaturesFromAllCameras(const std::vector<cv::Mat>& images);
    void MatchFeaturesAcrossCameras();
    
    // Methods for pose estimation with multi-camera setup
    bool TrackLocalMapWithMultiCameras();
    bool RelocalizationWithMultiCameras();
};
```

## 2. VR Motion Model Adaptations

### 2.1 VR-Specific Motion Model

```cpp
class VRMotionModel {
public:
    enum class HeadsetState {
        STATIONARY,
        SLOW_MOVEMENT,
        FAST_MOVEMENT,
        ROTATION_ONLY
    };
    
    // Methods for motion prediction
    Sophus::SE3f PredictPose(const Sophus::SE3f& current_pose, 
                            const std::vector<Sophus::SE3f>& pose_history,
                            const std::vector<double>& timestamps,
                            double prediction_time);
    
    // Methods for motion state estimation
    HeadsetState EstimateHeadsetState(const std::vector<Sophus::SE3f>& pose_history,
                                     const std::vector<double>& timestamps);
    
    // Methods for motion model adaptation
    void AdaptToUserMotion(const HeadsetState& state);
    void SetLatencyCompensation(double latency_ms);
};
```

### 2.2 Predictive Tracking for VR

```cpp
class PredictiveTracking {
public:
    struct PredictionConfig {
        double prediction_horizon_ms;   // How far ahead to predict
        double max_prediction_ms;       // Maximum prediction time
        bool use_imu_for_prediction;    // Whether to use IMU data for prediction
        bool adaptive_prediction;       // Whether to adapt prediction based on motion
    };
    
    // Methods for pose prediction
    Sophus::SE3f PredictPose(double prediction_time_ms);
    
    // Methods for velocity estimation
    Eigen::Vector3f EstimateLinearVelocity(const std::vector<Sophus::SE3f>& pose_history,
                                          const std::vector<double>& timestamps);
    Eigen::Vector3f EstimateAngularVelocity(const std::vector<Sophus::SE3f>& pose_history,
                                           const std::vector<double>& timestamps);
    
    // Methods for acceleration estimation
    Eigen::Vector3f EstimateLinearAcceleration(const std::vector<Sophus::SE3f>& pose_history,
                                              const std::vector<double>& timestamps);
    Eigen::Vector3f EstimateAngularAcceleration(const std::vector<Sophus::SE3f>& pose_history,
                                               const std::vector<double>& timestamps);
};
```

### 2.3 Low-Latency Tracking Optimizations

```cpp
class LowLatencyTracking {
public:
    // Methods for tracking optimization
    void OptimizeForLowLatency();
    void SetPriorityThreads(bool enable);
    void SetCPUAffinity(const std::vector<int>& core_ids);
    
    // Methods for tracking pipeline optimization
    void EnableAsyncFeatureExtraction(bool enable);
    void EnableAsyncMapping(bool enable);
    void EnableAsyncLoopClosure(bool enable);
    
    // Methods for tracking performance monitoring
    double MeasureTrackingLatency();
    double MeasureEndToEndLatency();
    std::vector<double> GetComponentLatencies();
};
```

## 3. Integration with ORB-SLAM3

### 3.1 Extended System Class

```cpp
class VRSLAMSystem : public ORB_SLAM3::System {
public:
    // Constructor with multi-camera and VR support
    VRSLAMSystem(const std::string& strVocFile, const std::string& strSettingsFile,
                 const MultiCameraRig& rig, const VRMotionModel::PredictionConfig& pred_config,
                 const ORB_SLAM3::System::eSensor sensor, const bool bUseViewer = true);
    
    // Methods for VR-specific tracking
    Sophus::SE3f TrackVR(const std::vector<cv::Mat>& images, const double& timestamp,
                        double prediction_time_ms = 0.0);
    
    // Methods for VR-specific mapping
    void EnableLowLatencyMapping(bool enable);
    
    // Methods for VR-specific loop closing
    void EnableAsyncLoopClosing(bool enable);
};
```

### 3.2 Multi-Stage SLAM Pipeline

```cpp
class MultiStageSLAMPipeline {
public:
    enum class Stage {
        FEATURE_EXTRACTION,
        TRACKING,
        LOCAL_MAPPING,
        LOOP_CLOSING
    };
    
    // Methods for pipeline management
    void ConfigurePipeline(const std::map<Stage, int>& stage_priorities);
    void EnableStage(Stage stage, bool enable);
    void SetStageCPUAffinity(Stage stage, const std::vector<int>& core_ids);
    
    // Methods for pipeline monitoring
    std::map<Stage, double> GetStageLatencies();
    double GetEndToEndLatency();
    
    // Methods for pipeline optimization
    void OptimizeForVR();
    void OptimizeForAccuracy();
    void OptimizeForPower();
};
```

## 4. Implementation Considerations

### 4.1 Camera Synchronization

For a multi-camera setup, precise synchronization is crucial. The implementation should:

1. Support hardware synchronization via trigger signals
2. Implement software synchronization for cameras without hardware sync
3. Handle timestamp alignment and interpolation
4. Detect and compensate for synchronization drift

### 4.2 Feature Matching Across Cameras

When features move between camera views, the system should:

1. Maintain feature identity across camera boundaries
2. Use the spherical model to predict feature locations in adjacent cameras
3. Implement efficient cross-camera feature matching
4. Handle different exposure and color characteristics between cameras

### 4.3 VR-Specific Optimizations

For VR applications, the implementation should:

1. Prioritize low latency over mapping accuracy when necessary
2. Implement predictive tracking to compensate for rendering and display latency
3. Adapt tracking parameters based on headset motion state
4. Provide smooth pose updates even during rapid head movements

### 4.4 EdgeTPU Integration

The multi-camera system should leverage the EdgeTPU for:

1. Parallel feature extraction across all cameras
2. Accelerated cross-camera feature matching
3. Efficient pose prediction for VR
4. Low-power operation for mobile VR headsets

## 5. Performance Targets

The multi-camera VR-SLAM system should meet the following performance targets:

1. **Tracking Rate**: Maintain 90Hz minimum tracking rate for VR
2. **Latency**: End-to-end tracking latency < 20ms
3. **Prediction Accuracy**: Motion prediction error < 2mm and < 0.5° for 50ms prediction
4. **Cross-Camera Tracking**: Seamless feature tracking across camera boundaries
5. **Field of View**: Near-spherical (360° horizontal, 180° vertical) tracking capability
6. **Relocalization**: Fast relocalization < 500ms after tracking loss
7. **Power Consumption**: Optimized for mobile VR headsets (< 2W for tracking)

## 6. Next Steps

1. Implement the `MultiCameraRig` class and calibration tools
2. Extend ORB-SLAM3's `Tracking` class to support multiple cameras
3. Implement the spherical FoV model and cross-camera feature matching
4. Develop the VR motion model and predictive tracking
5. Integrate with the TPU Feature Extractor for accelerated processing
6. Implement the multi-stage pipeline with optimized threading
7. Benchmark and optimize for VR performance targets

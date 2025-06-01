#ifndef MULTI_CAMERA_TRACKING_HPP
#define MULTI_CAMERA_TRACKING_HPP

#include <vector>
#include <map>
#include <mutex>
#include <thread>
#include <atomic>
#include <condition_variable>
#include <opencv2/core/mat.hpp>

#include "multi_camera_rig.hpp"
#include "../ORB_SLAM3/include/Tracking.h"
#include "../ORB_SLAM3/include/System.h"
#include "../ORB_SLAM3/include/Frame.h"
#include "../ORB_SLAM3/include/Atlas.h"
#include "../ORB_SLAM3/include/KeyFrameDatabase.h"
#include "../ORB_SLAM3/include/tpu_feature_extractor.hpp"

namespace ORB_SLAM3
{

/**
 * @brief Extension of the ORB-SLAM3 Tracking class to support multi-camera setups
 * 
 * This class extends the standard ORB-SLAM3 Tracking class to handle multiple
 * synchronized cameras, as would be found in a VR headset. It manages feature
 * extraction across cameras, cross-camera feature matching, and unified pose
 * estimation using a spherical field of view model.
 */
class MultiCameraTracking : public Tracking
{
public:
    /**
     * @brief Configuration structure for multi-camera tracking
     */
    struct Config {
        bool enable_cross_camera_matching = true;   ///< Enable feature matching across cameras
        bool use_spherical_model = true;            ///< Use spherical field of view model
        bool parallel_feature_extraction = true;    ///< Extract features in parallel across cameras
        float max_depth_difference = 0.1f;          ///< Maximum depth difference for cross-camera matching (ratio)
        float max_descriptor_distance = 50.0f;      ///< Maximum descriptor distance for cross-camera matching
        int min_cross_camera_matches = 10;          ///< Minimum number of cross-camera matches to consider
        float feature_sharing_overlap = 0.2f;       ///< Overlap region for feature sharing (ratio of FOV)
    };
    
    /**
     * @brief Constructor with multi-camera rig
     * 
     * @param pSys ORB-SLAM3 System
     * @param pVoc ORB vocabulary
     * @param pFrameDrawer Frame drawer
     * @param pMapDrawer Map drawer
     * @param pAtlas Atlas
     * @param pKFDB KeyFrame database
     * @param strSettingPath Path to settings file
     * @param sensor Sensor type
     * @param rig Multi-camera rig
     * @param config Configuration for multi-camera tracking
     */
    MultiCameraTracking(
        System* pSys,
        ORBVocabulary* pVoc,
        FrameDrawer* pFrameDrawer,
        MapDrawer* pMapDrawer,
        Atlas* pAtlas,
        KeyFrameDatabase* pKFDB,
        const std::string& strSettingPath,
        const int sensor,
        const MultiCameraRig& rig,
        const Config& config = Config());
    
    /**
     * @brief Destructor
     */
    ~MultiCameraTracking();
    
    /**
     * @brief Process multiple synchronized camera frames
     * 
     * @param images Vector of images, one per camera
     * @param timestamp Timestamp of the frame set
     * @param filenames Vector of filenames (optional)
     * @return Estimated camera pose
     */
    Sophus::SE3f GrabMultiCameraImages(
        const std::vector<cv::Mat>& images,
        const double& timestamp,
        const std::vector<std::string>& filenames = std::vector<std::string>());
    
    /**
     * @brief Get the best camera for viewing a specific 3D point
     * 
     * @param worldPoint 3D point in world coordinates
     * @return ID of the best camera, or -1 if no camera can see the point
     */
    int GetBestCameraForPoint(const cv::Point3f& worldPoint);
    
    /**
     * @brief Get the cameras that can see a specific 3D point
     * 
     * @param worldPoint 3D point in world coordinates
     * @return Vector of camera IDs that can see the point
     */
    std::vector<int> GetCamerasForPoint(const cv::Point3f& worldPoint);
    
    /**
     * @brief Get the current visibility map for all map points
     * 
     * @return Map from MapPoint* to vector of camera IDs that can see it
     */
    std::map<MapPoint*, std::vector<int>> GetMapPointVisibility();
    
    /**
     * @brief Get the multi-camera rig
     * 
     * @return Reference to the multi-camera rig
     */
    const MultiCameraRig& GetMultiCameraRig() const;
    
    /**
     * @brief Set the multi-camera rig
     * 
     * @param rig New multi-camera rig
     */
    void SetMultiCameraRig(const MultiCameraRig& rig);
    
    /**
     * @brief Get the configuration
     * 
     * @return Current configuration
     */
    Config GetConfig() const;
    
    /**
     * @brief Set the configuration
     * 
     * @param config New configuration
     */
    void SetConfig(const Config& config);
    
    /**
     * @brief Get the current active camera ID
     * 
     * @return ID of the currently active camera
     */
    int GetActiveCameraId() const;
    
    /**
     * @brief Set the active camera ID
     * 
     * @param camera_id ID of the camera to set as active
     */
    void SetActiveCameraId(int camera_id);
    
    /**
     * @brief Get the feature extractors for all cameras
     * 
     * @return Vector of feature extractors
     */
    std::vector<TPUFeatureExtractor*> GetFeatureExtractors() const;
    
protected:
    /**
     * @brief Main tracking function for multi-camera setup
     * 
     * This overrides the Track() method from the base Tracking class
     */
    void Track() override;
    
    /**
     * @brief Extract features from all cameras
     * 
     * @param images Vector of images, one per camera
     */
    void ExtractFeaturesFromAllCameras(const std::vector<cv::Mat>& images);
    
    /**
     * @brief Match features across cameras
     * 
     * @return Number of cross-camera matches found
     */
    int MatchFeaturesAcrossCameras();
    
    /**
     * @brief Track local map with multiple cameras
     * 
     * @return true if tracking was successful, false otherwise
     */
    bool TrackLocalMapWithMultiCameras();
    
    /**
     * @brief Relocalization with multiple cameras
     * 
     * @return true if relocalization was successful, false otherwise
     */
    bool RelocalizationWithMultiCameras();
    
    /**
     * @brief Create new keyframe with multi-camera data
     */
    void CreateNewMultiCameraKeyFrame();
    
    /**
     * @brief Project map points to all cameras
     * 
     * @param vpMapPoints Vector of map points to project
     * @return Map from camera ID to vector of projected points
     */
    std::map<int, std::vector<cv::Point2f>> ProjectMapPointsToAllCameras(
        const std::vector<MapPoint*>& vpMapPoints);
    
    /**
     * @brief Transform point from world to camera coordinates
     * 
     * @param worldPoint 3D point in world coordinates
     * @param camera_id ID of the camera
     * @return 3D point in camera coordinates
     */
    cv::Point3f WorldToCameraPoint(const cv::Point3f& worldPoint, int camera_id);
    
    /**
     * @brief Transform point from camera to world coordinates
     * 
     * @param cameraPoint 3D point in camera coordinates
     * @param camera_id ID of the camera
     * @return 3D point in world coordinates
     */
    cv::Point3f CameraToWorldPoint(const cv::Point3f& cameraPoint, int camera_id);
    
    /**
     * @brief Check if a point is visible to a camera
     * 
     * @param worldPoint 3D point in world coordinates
     * @param camera_id ID of the camera
     * @return true if the point is visible, false otherwise
     */
    bool IsPointVisibleToCamera(const cv::Point3f& worldPoint, int camera_id);
    
    /**
     * @brief Update camera poses based on reference camera pose
     * 
     * @param T_w_ref Transform from world to reference camera
     */
    void UpdateCameraPoses(const Sophus::SE3f& T_w_ref);
    
private:
    // Multi-camera rig
    MultiCameraRig mRig;
    
    // Configuration
    Config mConfig;
    
    // Currently active camera ID
    int mActiveCameraId;
    
    // Feature extractors for each camera
    std::vector<TPUFeatureExtractor*> mvpFeatureExtractors;
    
    // Frames for each camera
    std::vector<Frame> mvCameraFrames;
    
    // Cross-camera matches
    std::vector<std::pair<size_t, size_t>> mvCrossCameraMatches;
    
    // Camera poses (T_w_cam)
    std::vector<Sophus::SE3f> mvCameraPoses;
    
    // Mutex for protecting multi-camera data
    std::mutex mMutexMultiCamera;
    
    // Thread for parallel feature extraction
    std::vector<std::thread> mvFeatureExtractionThreads;
    std::mutex mMutexFeatureExtraction;
    std::condition_variable mCondVarFeatureExtraction;
    std::atomic<int> mNumCamerasProcessed;
    
    // Helper methods
    
    /**
     * @brief Initialize feature extractors for all cameras
     */
    void InitializeFeatureExtractors();
    
    /**
     * @brief Extract features from a single camera
     * 
     * @param camera_id ID of the camera
     * @param image Image from the camera
     */
    void ExtractFeaturesFromCamera(int camera_id, const cv::Mat& image);
    
    /**
     * @brief Find matches between two cameras
     * 
     * @param camera_id1 ID of the first camera
     * @param camera_id2 ID of the second camera
     * @return Vector of matches (indices in respective frames)
     */
    std::vector<std::pair<size_t, size_t>> FindMatchesBetweenCameras(
        int camera_id1, int camera_id2);
    
    /**
     * @brief Merge map points from cross-camera matches
     * 
     * @param matches Vector of matches (indices in respective frames)
     * @param camera_id1 ID of the first camera
     * @param camera_id2 ID of the second camera
     */
    void MergeMapPointsFromMatches(
        const std::vector<std::pair<size_t, size_t>>& matches,
        int camera_id1, int camera_id2);
};

} // namespace ORB_SLAM3

#endif // MULTI_CAMERA_TRACKING_HPP

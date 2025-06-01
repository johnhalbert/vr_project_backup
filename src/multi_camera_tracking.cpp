#include "include/multi_camera_tracking.hpp"
#include <opencv2/core/core.hpp>
#include <opencv2/features2d/features2d.hpp>
#include <opencv2/calib3d/calib3d.hpp>
#include <thread>
#include <algorithm>
#include <chrono>

namespace ORB_SLAM3
{

MultiCameraTracking::MultiCameraTracking(
    System* pSys,
    ORBVocabulary* pVoc,
    FrameDrawer* pFrameDrawer,
    MapDrawer* pMapDrawer,
    Atlas* pAtlas,
    KeyFrameDatabase* pKFDB,
    const std::string& strSettingPath,
    const int sensor,
    const MultiCameraRig& rig,
    const Config& config)
    : Tracking(pSys, pVoc, pFrameDrawer, pMapDrawer, pAtlas, pKFDB, strSettingPath, sensor),
      mRig(rig),
      mConfig(config),
      mActiveCameraId(rig.GetReferenceCameraId()),
      mNumCamerasProcessed(0)
{
    std::cout << "Initializing Multi-Camera Tracking with " << mRig.GetAllCameras().size() << " cameras" << std::endl;
    
    // Initialize feature extractors for all cameras
    InitializeFeatureExtractors();
    
    // Initialize camera frames
    mvCameraFrames.resize(mRig.GetAllCameras().size());
    
    // Initialize camera poses
    mvCameraPoses.resize(mRig.GetAllCameras().size());
    for (size_t i = 0; i < mvCameraPoses.size(); i++) {
        mvCameraPoses[i] = Sophus::SE3f();
    }
    
    std::cout << "Multi-Camera Tracking initialized successfully" << std::endl;
}

MultiCameraTracking::~MultiCameraTracking()
{
    // Clean up feature extractors
    for (auto extractor : mvpFeatureExtractors) {
        delete extractor;
    }
    mvpFeatureExtractors.clear();
}

Sophus::SE3f MultiCameraTracking::GrabMultiCameraImages(
    const std::vector<cv::Mat>& images,
    const double& timestamp,
    const std::vector<std::string>& filenames)
{
    // Check if number of images matches number of cameras
    if (images.size() != mRig.GetAllCameras().size()) {
        std::cerr << "Error: Number of images (" << images.size() 
                  << ") does not match number of cameras (" << mRig.GetAllCameras().size() << ")" << std::endl;
        return Sophus::SE3f();
    }
    
    // Extract features from all cameras
    ExtractFeaturesFromAllCameras(images);
    
    // Match features across cameras
    int nCrossCameraMatches = 0;
    if (mConfig.enable_cross_camera_matching) {
        nCrossCameraMatches = MatchFeaturesAcrossCameras();
        std::cout << "Found " << nCrossCameraMatches << " cross-camera matches" << std::endl;
    }
    
    // Set the main frame to the active camera's frame
    mCurrentFrame = mvCameraFrames[mActiveCameraId];
    
    // Set the timestamp for the current frame
    mCurrentFrame.mTimeStamp = timestamp;
    
    // Track with the active camera
    Track();
    
    // Update poses for all cameras based on the active camera's pose
    UpdateCameraPoses(mCurrentFrame.GetPose());
    
    // Return the pose of the active camera
    return mCurrentFrame.GetPose();
}

int MultiCameraTracking::GetBestCameraForPoint(const cv::Point3f& worldPoint)
{
    // Convert world point to reference frame
    Sophus::SE3f T_w_ref = mvCameraPoses[mRig.GetReferenceCameraId()];
    Sophus::SE3f T_ref_w = T_w_ref.inverse();
    
    Eigen::Vector3f point_ref = T_ref_w * Eigen::Vector3f(worldPoint.x, worldPoint.y, worldPoint.z);
    cv::Point3f refPoint(point_ref(0), point_ref(1), point_ref(2));
    
    // Use the rig's method to find the best camera
    return mRig.FindBestCameraForPoint(refPoint);
}

std::vector<int> MultiCameraTracking::GetCamerasForPoint(const cv::Point3f& worldPoint)
{
    std::vector<int> visibleCameras;
    
    // Convert world point to reference frame
    Sophus::SE3f T_w_ref = mvCameraPoses[mRig.GetReferenceCameraId()];
    Sophus::SE3f T_ref_w = T_w_ref.inverse();
    
    Eigen::Vector3f point_ref = T_ref_w * Eigen::Vector3f(worldPoint.x, worldPoint.y, worldPoint.z);
    cv::Point3f refPoint(point_ref(0), point_ref(1), point_ref(2));
    
    // Check visibility for each camera
    for (const auto& camera : mRig.GetAllCameras()) {
        if (mRig.IsPointVisibleToCamera(refPoint, camera.id)) {
            visibleCameras.push_back(camera.id);
        }
    }
    
    return visibleCameras;
}

std::map<MapPoint*, std::vector<int>> MultiCameraTracking::GetMapPointVisibility()
{
    std::map<MapPoint*, std::vector<int>> visibility;
    
    // Get local map points
    std::vector<MapPoint*> vpMapPoints = GetLocalMapMPS();
    
    // Check visibility for each map point
    for (auto pMP : vpMapPoints) {
        if (!pMP || pMP->isBad()) continue;
        
        // Get world position
        cv::Mat pos = pMP->GetWorldPos();
        cv::Point3f worldPoint(pos.at<float>(0), pos.at<float>(1), pos.at<float>(2));
        
        // Get cameras that can see this point
        std::vector<int> cameras = GetCamerasForPoint(worldPoint);
        
        if (!cameras.empty()) {
            visibility[pMP] = cameras;
        }
    }
    
    return visibility;
}

const MultiCameraRig& MultiCameraTracking::GetMultiCameraRig() const
{
    return mRig;
}

void MultiCameraTracking::SetMultiCameraRig(const MultiCameraRig& rig)
{
    std::unique_lock<std::mutex> lock(mMutexMultiCamera);
    mRig = rig;
    
    // Update active camera ID if necessary
    if (mActiveCameraId >= static_cast<int>(mRig.GetAllCameras().size())) {
        mActiveCameraId = mRig.GetReferenceCameraId();
    }
    
    // Reinitialize feature extractors
    for (auto extractor : mvpFeatureExtractors) {
        delete extractor;
    }
    mvpFeatureExtractors.clear();
    InitializeFeatureExtractors();
    
    // Resize camera frames and poses
    mvCameraFrames.resize(mRig.GetAllCameras().size());
    mvCameraPoses.resize(mRig.GetAllCameras().size());
    for (size_t i = 0; i < mvCameraPoses.size(); i++) {
        mvCameraPoses[i] = Sophus::SE3f();
    }
}

MultiCameraTracking::Config MultiCameraTracking::GetConfig() const
{
    return mConfig;
}

void MultiCameraTracking::SetConfig(const Config& config)
{
    mConfig = config;
}

int MultiCameraTracking::GetActiveCameraId() const
{
    return mActiveCameraId;
}

void MultiCameraTracking::SetActiveCameraId(int camera_id)
{
    if (camera_id >= 0 && camera_id < static_cast<int>(mRig.GetAllCameras().size())) {
        mActiveCameraId = camera_id;
    } else {
        std::cerr << "Invalid camera ID: " << camera_id << std::endl;
    }
}

std::vector<TPUFeatureExtractor*> MultiCameraTracking::GetFeatureExtractors() const
{
    return mvpFeatureExtractors;
}

void MultiCameraTracking::Track()
{
    // Call the base class Track method
    // This will use mCurrentFrame which we've set to the active camera's frame
    Tracking::Track();
}

void MultiCameraTracking::ExtractFeaturesFromAllCameras(const std::vector<cv::Mat>& images)
{
    // Reset counter for parallel processing
    mNumCamerasProcessed = 0;
    
    if (mConfig.parallel_feature_extraction) {
        // Extract features in parallel
        for (size_t i = 0; i < images.size(); i++) {
            mvFeatureExtractionThreads.push_back(
                std::thread(&MultiCameraTracking::ExtractFeaturesFromCamera, this, i, images[i]));
        }
        
        // Wait for all threads to finish
        for (auto& thread : mvFeatureExtractionThreads) {
            thread.join();
        }
        mvFeatureExtractionThreads.clear();
    } else {
        // Extract features sequentially
        for (size_t i = 0; i < images.size(); i++) {
            ExtractFeaturesFromCamera(i, images[i]);
        }
    }
}

int MultiCameraTracking::MatchFeaturesAcrossCameras()
{
    int totalMatches = 0;
    mvCrossCameraMatches.clear();
    
    // For each pair of cameras
    for (size_t i = 0; i < mvCameraFrames.size(); i++) {
        for (size_t j = i + 1; j < mvCameraFrames.size(); j++) {
            // Find matches between cameras i and j
            auto matches = FindMatchesBetweenCameras(i, j);
            
            // Add to total matches
            totalMatches += matches.size();
            
            // Store matches
            mvCrossCameraMatches.insert(mvCrossCameraMatches.end(), matches.begin(), matches.end());
            
            // Merge map points from matches
            MergeMapPointsFromMatches(matches, i, j);
        }
    }
    
    return totalMatches;
}

bool MultiCameraTracking::TrackLocalMapWithMultiCameras()
{
    // This is a placeholder for the implementation
    // In a full implementation, this would track the local map using all cameras
    
    // For now, just use the active camera's tracking
    return TrackLocalMap();
}

bool MultiCameraTracking::RelocalizationWithMultiCameras()
{
    // This is a placeholder for the implementation
    // In a full implementation, this would attempt relocalization with all cameras
    
    // For now, just use the active camera's relocalization
    return Relocalization();
}

void MultiCameraTracking::CreateNewMultiCameraKeyFrame()
{
    // This is a placeholder for the implementation
    // In a full implementation, this would create a new keyframe with data from all cameras
    
    // For now, just use the active camera's keyframe creation
    CreateNewKeyFrame();
}

std::map<int, std::vector<cv::Point2f>> MultiCameraTracking::ProjectMapPointsToAllCameras(
    const std::vector<MapPoint*>& vpMapPoints)
{
    std::map<int, std::vector<cv::Point2f>> projections;
    
    // Initialize projections for each camera
    for (const auto& camera : mRig.GetAllCameras()) {
        projections[camera.id] = std::vector<cv::Point2f>();
    }
    
    // Project each map point to each camera
    for (auto pMP : vpMapPoints) {
        if (!pMP || pMP->isBad()) continue;
        
        // Get world position
        cv::Mat pos = pMP->GetWorldPos();
        cv::Point3f worldPoint(pos.at<float>(0), pos.at<float>(1), pos.at<float>(2));
        
        // Project to each camera
        for (const auto& camera : mRig.GetAllCameras()) {
            if (IsPointVisibleToCamera(worldPoint, camera.id)) {
                // Convert world point to camera coordinates
                cv::Point3f cameraPoint = WorldToCameraPoint(worldPoint, camera.id);
                
                // Project to image
                cv::Point2f imagePoint;
                if (cameraPoint.z > 0) {
                    // Simple pinhole projection
                    imagePoint.x = cameraPoint.x / cameraPoint.z;
                    imagePoint.y = cameraPoint.y / cameraPoint.z;
                    
                    // Apply camera intrinsics
                    const auto& cameraInfo = mRig.GetCameraInfo(camera.id);
                    imagePoint.x = cameraInfo.K.at<float>(0, 0) * imagePoint.x + cameraInfo.K.at<float>(0, 2);
                    imagePoint.y = cameraInfo.K.at<float>(1, 1) * imagePoint.y + cameraInfo.K.at<float>(1, 2);
                    
                    // Add to projections
                    projections[camera.id].push_back(imagePoint);
                }
            }
        }
    }
    
    return projections;
}

cv::Point3f MultiCameraTracking::WorldToCameraPoint(const cv::Point3f& worldPoint, int camera_id)
{
    // Get camera pose
    Sophus::SE3f T_w_cam = mvCameraPoses[camera_id];
    Sophus::SE3f T_cam_w = T_w_cam.inverse();
    
    // Transform point
    Eigen::Vector3f point_cam = T_cam_w * Eigen::Vector3f(worldPoint.x, worldPoint.y, worldPoint.z);
    
    return cv::Point3f(point_cam(0), point_cam(1), point_cam(2));
}

cv::Point3f MultiCameraTracking::CameraToWorldPoint(const cv::Point3f& cameraPoint, int camera_id)
{
    // Get camera pose
    Sophus::SE3f T_w_cam = mvCameraPoses[camera_id];
    
    // Transform point
    Eigen::Vector3f point_world = T_w_cam * Eigen::Vector3f(cameraPoint.x, cameraPoint.y, cameraPoint.z);
    
    return cv::Point3f(point_world(0), point_world(1), point_world(2));
}

bool MultiCameraTracking::IsPointVisibleToCamera(const cv::Point3f& worldPoint, int camera_id)
{
    // Convert world point to camera coordinates
    cv::Point3f cameraPoint = WorldToCameraPoint(worldPoint, camera_id);
    
    // Check if point is in front of camera
    if (cameraPoint.z <= 0) {
        return false;
    }
    
    // Project to image
    const auto& cameraInfo = mRig.GetCameraInfo(camera_id);
    float x = cameraInfo.K.at<float>(0, 0) * cameraPoint.x / cameraPoint.z + cameraInfo.K.at<float>(0, 2);
    float y = cameraInfo.K.at<float>(1, 1) * cameraPoint.y / cameraPoint.z + cameraInfo.K.at<float>(1, 2);
    
    // Check if point is within image bounds
    return (x >= 0 && x < cameraInfo.width && y >= 0 && y < cameraInfo.height);
}

void MultiCameraTracking::UpdateCameraPoses(const Sophus::SE3f& T_w_ref)
{
    // Set the pose for the reference camera
    int refId = mRig.GetReferenceCameraId();
    mvCameraPoses[refId] = T_w_ref;
    
    // Update poses for all other cameras based on their relative transforms
    for (const auto& camera : mRig.GetAllCameras()) {
        if (camera.id == refId) continue;
        
        // Get transform from reference to this camera
        cv::Mat T_ref_cam = camera.T_ref_cam;
        
        // Convert to Sophus::SE3f
        Eigen::Matrix3f R;
        Eigen::Vector3f t;
        for (int i = 0; i < 3; i++) {
            for (int j = 0; j < 3; j++) {
                R(i, j) = T_ref_cam.at<float>(i, j);
            }
            t(i) = T_ref_cam.at<float>(i, 3);
        }
        Sophus::SE3f T_ref_cam_sophus(R, t);
        
        // Calculate world to camera transform
        mvCameraPoses[camera.id] = T_w_ref * T_ref_cam_sophus;
    }
}

void MultiCameraTracking::InitializeFeatureExtractors()
{
    // Clear existing extractors
    for (auto extractor : mvpFeatureExtractors) {
        delete extractor;
    }
    mvpFeatureExtractors.clear();
    
    // Create extractors for each camera
    for (const auto& camera : mRig.GetAllCameras()) {
        // Create a new feature extractor with the same parameters as the main one
        TPUFeatureExtractor* pExtractor = new TPUFeatureExtractor(
            mpORBextractorLeft->GetModelPath(),
            mpORBextractorLeft->GetDelegatePath(),
            mpORBextractorLeft->GetMaxFeatures(),
            mpORBextractorLeft->GetScaleFactor(),
            mpORBextractorLeft->GetLevels()
        );
        
        mvpFeatureExtractors.push_back(pExtractor);
    }
}

void MultiCameraTracking::ExtractFeaturesFromCamera(int camera_id, const cv::Mat& image)
{
    // Get camera info
    const auto& cameraInfo = mRig.GetCameraInfo(camera_id);
    
    // Convert image to grayscale if necessary
    cv::Mat grayImage;
    if (image.channels() == 3) {
        cv::cvtColor(image, grayImage, cv::COLOR_BGR2GRAY);
    } else {
        grayImage = image.clone();
    }
    
    // Create a mask (all pixels valid)
    cv::Mat mask = cv::Mat::ones(grayImage.size(), CV_8UC1);
    
    // Extract features
    std::vector<cv::KeyPoint> keypoints;
    cv::Mat descriptors;
    std::vector<int> lapping_area;
    
    // Use the appropriate feature extractor
    (*mvpFeatureExtractors[camera_id])(grayImage, mask, keypoints, descriptors, lapping_area);
    
    // Create a new frame
    Frame frame(
        grayImage,
        0.0, // timestamp will be set later
        mvpFeatureExtractors[camera_id],
        mpORBVocabulary,
        cameraInfo.K,
        cameraInfo.distCoef,
        0.0, // baseline for stereo (not used)
        0.0  // threshold depth for stereo (not used)
    );
    
    // Store the frame
    mvCameraFrames[camera_id] = frame;
    
    // Increment the counter for parallel processing
    mNumCamerasProcessed++;
}

std::vector<std::pair<size_t, size_t>> MultiCameraTracking::FindMatchesBetweenCameras(
    int camera_id1, int camera_id2)
{
    std::vector<std::pair<size_t, size_t>> matches;
    
    // Get frames
    const Frame& frame1 = mvCameraFrames[camera_id1];
    const Frame& frame2 = mvCameraFrames[camera_id2];
    
    // Get keypoints and descriptors
    const std::vector<cv::KeyPoint>& keypoints1 = frame1.mvKeysUn;
    const std::vector<cv::KeyPoint>& keypoints2 = frame2.mvKeysUn;
    const cv::Mat& descriptors1 = frame1.mDescriptors;
    const cv::Mat& descriptors2 = frame2.mDescriptors;
    
    // Check if we have keypoints and descriptors
    if (keypoints1.empty() || keypoints2.empty() || descriptors1.empty() || descriptors2.empty()) {
        return matches;
    }
    
    // Get camera info
    const auto& cameraInfo1 = mRig.GetCameraInfo(camera_id1);
    const auto& cameraInfo2 = mRig.GetCameraInfo(camera_id2);
    
    // Get transform between cameras
    cv::Mat T_cam1_cam2 = mRig.GetTransform(camera_id2, camera_id1);
    
    // For each keypoint in frame1
    for (size_t i = 0; i < keypoints1.size(); i++) {
        // Get 3D point in camera1 coordinates (assuming depth = 1)
        cv::Point3f point1(
            (keypoints1[i].pt.x - cameraInfo1.K.at<float>(0, 2)) / cameraInfo1.K.at<float>(0, 0),
            (keypoints1[i].pt.y - cameraInfo1.K.at<float>(1, 2)) / cameraInfo1.K.at<float>(1, 1),
            1.0f
        );
        
        // Transform to camera2 coordinates
        cv::Mat point1_mat = (cv::Mat_<float>(4, 1) << point1.x, point1.y, point1.z, 1.0f);
        cv::Mat point2_mat = T_cam1_cam2 * point1_mat;
        cv::Point3f point2(
            point2_mat.at<float>(0, 0),
            point2_mat.at<float>(1, 0),
            point2_mat.at<float>(2, 0)
        );
        
        // Check if point is in front of camera2
        if (point2.z <= 0) {
            continue;
        }
        
        // Project to camera2 image
        cv::Point2f proj2(
            cameraInfo2.K.at<float>(0, 0) * point2.x / point2.z + cameraInfo2.K.at<float>(0, 2),
            cameraInfo2.K.at<float>(1, 1) * point2.y / point2.z + cameraInfo2.K.at<float>(1, 2)
        );
        
        // Check if projection is within image bounds
        if (proj2.x < 0 || proj2.x >= cameraInfo2.width || proj2.y < 0 || proj2.y >= cameraInfo2.height) {
            continue;
        }
        
        // Find the closest keypoint in frame2
        float minDist = std::numeric_limits<float>::max();
        size_t bestIdx = 0;
        
        for (size_t j = 0; j < keypoints2.size(); j++) {
            // Check distance between projection and keypoint
            float dist = cv::norm(proj2 - keypoints2[j].pt);
            
            // If too far, skip
            if (dist > 10.0f) {
                continue;
            }
            
            // Check descriptor distance
            float descDist = 0.0f;
            for (int k = 0; k < descriptors1.cols; k++) {
                descDist += std::abs(descriptors1.at<float>(i, k) - descriptors2.at<float>(j, k));
            }
            
            // If descriptor distance is too large, skip
            if (descDist > mConfig.max_descriptor_distance) {
                continue;
            }
            
            // Update best match
            if (descDist < minDist) {
                minDist = descDist;
                bestIdx = j;
            }
        }
        
        // If we found a match
        if (minDist < mConfig.max_descriptor_distance) {
            matches.push_back(std::make_pair(i, bestIdx));
        }
    }
    
    return matches;
}

void MultiCameraTracking::MergeMapPointsFromMatches(
    const std::vector<std::pair<size_t, size_t>>& matches,
    int camera_id1, int camera_id2)
{
    // Get frames
    Frame& frame1 = mvCameraFrames[camera_id1];
    Frame& frame2 = mvCameraFrames[camera_id2];
    
    // For each match
    for (const auto& match : matches) {
        size_t idx1 = match.first;
        size_t idx2 = match.second;
        
        // Get map points
        MapPoint* pMP1 = frame1.mvpMapPoints[idx1];
        MapPoint* pMP2 = frame2.mvpMapPoints[idx2];
        
        // If both have map points, merge them
        if (pMP1 && pMP2 && !pMP1->isBad() && !pMP2->isBad()) {
            // Keep the one with more observations
            if (pMP1->Observations() > pMP2->Observations()) {
                frame2.mvpMapPoints[idx2] = pMP1;
                pMP2->Replace(pMP1);
            } else {
                frame1.mvpMapPoints[idx1] = pMP2;
                pMP1->Replace(pMP2);
            }
        }
        // If only one has a map point, share it
        else if (pMP1 && !pMP1->isBad()) {
            frame2.mvpMapPoints[idx2] = pMP1;
        }
        else if (pMP2 && !pMP2->isBad()) {
            frame1.mvpMapPoints[idx1] = pMP2;
        }
    }
}

} // namespace ORB_SLAM3

#include "include/multi_camera_rig.hpp"
#include <fstream>
#include <iostream>
#include <opencv2/imgproc.hpp>
#include <opencv2/calib3d.hpp>
#include <opencv2/highgui.hpp>
#include <cmath>
#include <json/json.h> // For JSON serialization

namespace ORB_SLAM3
{

MultiCameraRig::MultiCameraRig() : reference_camera_id_(-1)
{
}

MultiCameraRig::MultiCameraRig(int reference_camera_id) : reference_camera_id_(reference_camera_id)
{
}

MultiCameraRig::~MultiCameraRig()
{
}

bool MultiCameraRig::AddCamera(const CameraInfo& camera)
{
    // Check if camera with this ID already exists
    if (cameras_.find(camera.id) != cameras_.end()) {
        std::cerr << "Camera with ID " << camera.id << " already exists in the rig." << std::endl;
        return false;
    }
    
    // Add camera to the map
    cameras_[camera.id] = camera;
    
    // If this is the first camera, set it as reference by default
    if (cameras_.size() == 1 && reference_camera_id_ == -1) {
        reference_camera_id_ = camera.id;
    }
    
    return true;
}

bool MultiCameraRig::RemoveCamera(int camera_id)
{
    // Check if camera exists
    if (cameras_.find(camera_id) == cameras_.end()) {
        std::cerr << "Camera with ID " << camera_id << " does not exist in the rig." << std::endl;
        return false;
    }
    
    // Remove camera from the map
    cameras_.erase(camera_id);
    
    // If the reference camera was removed, set a new reference if possible
    if (camera_id == reference_camera_id_) {
        if (!cameras_.empty()) {
            reference_camera_id_ = cameras_.begin()->first;
        } else {
            reference_camera_id_ = -1;
        }
    }
    
    return true;
}

MultiCameraRig::CameraInfo MultiCameraRig::GetCameraInfo(int camera_id) const
{
    // Check if camera exists
    auto it = cameras_.find(camera_id);
    if (it == cameras_.end()) {
        std::cerr << "Camera with ID " << camera_id << " does not exist in the rig." << std::endl;
        return CameraInfo(); // Return empty camera info
    }
    
    return it->second;
}

std::vector<MultiCameraRig::CameraInfo> MultiCameraRig::GetAllCameras() const
{
    std::vector<CameraInfo> result;
    for (const auto& pair : cameras_) {
        result.push_back(pair.second);
    }
    return result;
}

int MultiCameraRig::GetReferenceCameraId() const
{
    return reference_camera_id_;
}

bool MultiCameraRig::SetReferenceCameraId(int camera_id)
{
    // Check if camera exists
    if (cameras_.find(camera_id) == cameras_.end()) {
        std::cerr << "Camera with ID " << camera_id << " does not exist in the rig." << std::endl;
        return false;
    }
    
    // Set as reference camera
    reference_camera_id_ = camera_id;
    
    // Update transforms for all cameras relative to the new reference
    if (cameras_.size() > 1) {
        // Get the transform from old reference to new reference
        cv::Mat T_old_new = cameras_[camera_id].T_ref_cam.inv();
        
        // Update transforms for all cameras
        for (auto& pair : cameras_) {
            if (pair.first != camera_id) {
                // T_new_cam = T_old_new * T_old_cam
                pair.second.T_ref_cam = T_old_new * pair.second.T_ref_cam;
            }
        }
        
        // Set the reference camera's transform to identity
        cameras_[camera_id].T_ref_cam = cv::Mat::eye(4, 4, CV_32F);
    }
    
    return true;
}

bool MultiCameraRig::CalibrateRig(
    const std::vector<std::vector<cv::Mat>>& calibration_images,
    const cv::Size& pattern_size,
    float square_size)
{
    // Check if we have enough cameras
    if (cameras_.empty()) {
        std::cerr << "No cameras in the rig to calibrate." << std::endl;
        return false;
    }
    
    // Check if we have enough calibration images
    if (calibration_images.size() != cameras_.size()) {
        std::cerr << "Number of image sets (" << calibration_images.size() 
                  << ") does not match number of cameras (" << cameras_.size() << ")." << std::endl;
        return false;
    }
    
    // Step 1: Calibrate individual cameras
    if (!CalibrateIndividualCameras(calibration_images, pattern_size, square_size)) {
        std::cerr << "Failed to calibrate individual cameras." << std::endl;
        return false;
    }
    
    // Step 2: Calibrate camera pairs to establish relative poses
    if (cameras_.size() > 1) {
        if (!CalibrateCameraPairs(calibration_images, pattern_size)) {
            std::cerr << "Failed to calibrate camera pairs." << std::endl;
            return false;
        }
    }
    
    // Step 3: Optimize the entire rig calibration
    if (cameras_.size() > 2) {
        if (!OptimizeRigCalibration()) {
            std::cerr << "Failed to optimize rig calibration." << std::endl;
            return false;
        }
    }
    
    return true;
}

bool MultiCameraRig::LoadCalibration(const std::string& filename)
{
    try {
        // Open file
        std::ifstream file(filename);
        if (!file.is_open()) {
            std::cerr << "Failed to open calibration file: " << filename << std::endl;
            return false;
        }
        
        // Parse JSON
        Json::Value root;
        file >> root;
        
        // Clear existing cameras
        cameras_.clear();
        
        // Read reference camera ID
        reference_camera_id_ = root["reference_camera_id"].asInt();
        
        // Read cameras
        const Json::Value& cameras = root["cameras"];
        for (const auto& camera : cameras) {
            CameraInfo info;
            
            // Read basic info
            info.id = camera["id"].asInt();
            info.fps = camera["fps"].asFloat();
            info.width = camera["width"].asInt();
            info.height = camera["height"].asInt();
            info.model = camera["model"].asString();
            info.fov_horizontal = camera["fov_horizontal"].asFloat();
            info.fov_vertical = camera["fov_vertical"].asFloat();
            
            // Read intrinsic matrix
            const Json::Value& K = camera["K"];
            info.K = cv::Mat::zeros(3, 3, CV_32F);
            for (int i = 0; i < 3; ++i) {
                for (int j = 0; j < 3; ++j) {
                    info.K.at<float>(i, j) = K[i * 3 + j].asFloat();
                }
            }
            
            // Read distortion coefficients
            const Json::Value& distCoef = camera["distCoef"];
            info.distCoef = cv::Mat::zeros(1, distCoef.size(), CV_32F);
            for (int i = 0; i < distCoef.size(); ++i) {
                info.distCoef.at<float>(0, i) = distCoef[i].asFloat();
            }
            
            // Read transform
            const Json::Value& T = camera["T_ref_cam"];
            info.T_ref_cam = cv::Mat::zeros(4, 4, CV_32F);
            for (int i = 0; i < 4; ++i) {
                for (int j = 0; j < 4; ++j) {
                    info.T_ref_cam.at<float>(i, j) = T[i * 4 + j].asFloat();
                }
            }
            
            // Add camera to the rig
            cameras_[info.id] = info;
        }
        
        return true;
    } catch (const std::exception& e) {
        std::cerr << "Error loading calibration: " << e.what() << std::endl;
        return false;
    }
}

bool MultiCameraRig::SaveCalibration(const std::string& filename) const
{
    try {
        // Create JSON root
        Json::Value root;
        
        // Write reference camera ID
        root["reference_camera_id"] = reference_camera_id_;
        
        // Write cameras
        Json::Value cameras(Json::arrayValue);
        for (const auto& pair : cameras_) {
            const CameraInfo& info = pair.second;
            Json::Value camera;
            
            // Write basic info
            camera["id"] = info.id;
            camera["fps"] = info.fps;
            camera["width"] = info.width;
            camera["height"] = info.height;
            camera["model"] = info.model;
            camera["fov_horizontal"] = info.fov_horizontal;
            camera["fov_vertical"] = info.fov_vertical;
            
            // Write intrinsic matrix
            Json::Value K(Json::arrayValue);
            for (int i = 0; i < 3; ++i) {
                for (int j = 0; j < 3; ++j) {
                    K.append(info.K.at<float>(i, j));
                }
            }
            camera["K"] = K;
            
            // Write distortion coefficients
            Json::Value distCoef(Json::arrayValue);
            for (int i = 0; i < info.distCoef.cols; ++i) {
                distCoef.append(info.distCoef.at<float>(0, i));
            }
            camera["distCoef"] = distCoef;
            
            // Write transform
            Json::Value T(Json::arrayValue);
            for (int i = 0; i < 4; ++i) {
                for (int j = 0; j < 4; ++j) {
                    T.append(info.T_ref_cam.at<float>(i, j));
                }
            }
            camera["T_ref_cam"] = T;
            
            cameras.append(camera);
        }
        root["cameras"] = cameras;
        
        // Write to file
        std::ofstream file(filename);
        if (!file.is_open()) {
            std::cerr << "Failed to open file for writing: " << filename << std::endl;
            return false;
        }
        
        file << root;
        return true;
    } catch (const std::exception& e) {
        std::cerr << "Error saving calibration: " << e.what() << std::endl;
        return false;
    }
}

cv::Mat MultiCameraRig::ProjectToSpherical(
    const std::vector<cv::Mat>& images,
    const cv::Size& resolution)
{
    // Check if we have enough cameras and images
    if (cameras_.empty()) {
        std::cerr << "No cameras in the rig for spherical projection." << std::endl;
        return cv::Mat();
    }
    
    if (images.size() != cameras_.size()) {
        std::cerr << "Number of images (" << images.size() 
                  << ") does not match number of cameras (" << cameras_.size() << ")." << std::endl;
        return cv::Mat();
    }
    
    // Create output panorama
    cv::Mat panorama = cv::Mat::zeros(resolution.height, resolution.width, CV_8UC3);
    
    // Create maps for each camera
    std::vector<cv::Mat> maps;
    for (const auto& pair : cameras_) {
        maps.push_back(CreateSphericalMap(pair.first, resolution));
    }
    
    // Blend images into panorama
    int camera_idx = 0;
    for (const auto& pair : cameras_) {
        // Get camera info and image
        const CameraInfo& info = pair.second;
        const cv::Mat& image = images[camera_idx];
        
        // Check if image dimensions match camera info
        if (image.rows != info.height || image.cols != info.width) {
            std::cerr << "Image dimensions (" << image.cols << "x" << image.rows 
                      << ") do not match camera info (" << info.width << "x" << info.height 
                      << ") for camera " << info.id << std::endl;
            return cv::Mat();
        }
        
        // Remap image to panorama
        cv::Mat remapped;
        cv::remap(image, remapped, maps[camera_idx], cv::Mat(), cv::INTER_LINEAR);
        
        // Blend into panorama (simple overwrite for now)
        // TODO: Implement proper blending with weights based on viewing angle
        remapped.copyTo(panorama, remapped > 0);
        
        camera_idx++;
    }
    
    return panorama;
}

std::vector<cv::Point3f> MultiCameraRig::ProjectPointsToSphere(
    const std::vector<cv::Point2f>& points,
    int camera_id)
{
    std::vector<cv::Point3f> sphere_points;
    
    // Check if camera exists
    auto it = cameras_.find(camera_id);
    if (it == cameras_.end()) {
        std::cerr << "Camera with ID " << camera_id << " does not exist in the rig." << std::endl;
        return sphere_points;
    }
    
    // Project each point to the sphere
    for (const auto& point : points) {
        sphere_points.push_back(CameraToSphere(point, camera_id));
    }
    
    return sphere_points;
}

std::vector<cv::Point2f> MultiCameraRig::ProjectSphericalPointsToCamera(
    const std::vector<cv::Point3f>& sphere_points,
    int camera_id)
{
    std::vector<cv::Point2f> image_points;
    
    // Check if camera exists
    auto it = cameras_.find(camera_id);
    if (it == cameras_.end()) {
        std::cerr << "Camera with ID " << camera_id << " does not exist in the rig." << std::endl;
        return image_points;
    }
    
    // Project each sphere point to the camera
    for (const auto& point : sphere_points) {
        image_points.push_back(SphereToCameraProjection(point, camera_id));
    }
    
    return image_points;
}

bool MultiCameraRig::IsPointVisibleToCamera(
    const cv::Point3f& sphere_point,
    int camera_id)
{
    // Check if camera exists
    auto it = cameras_.find(camera_id);
    if (it == cameras_.end()) {
        std::cerr << "Camera with ID " << camera_id << " does not exist in the rig." << std::endl;
        return false;
    }
    
    // Get camera info
    const CameraInfo& info = it->second;
    
    // Transform sphere point to camera coordinates
    cv::Mat sphere_point_mat = (cv::Mat_<float>(4, 1) << 
        sphere_point.x, sphere_point.y, sphere_point.z, 1.0f);
    cv::Mat camera_point_mat = info.T_ref_cam * sphere_point_mat;
    
    // Check if point is in front of camera
    if (camera_point_mat.at<float>(2, 0) <= 0) {
        return false;
    }
    
    // Project point to image
    cv::Point2f image_point = SphereToCameraProjection(sphere_point, camera_id);
    
    // Check if point is within image bounds
    return (image_point.x >= 0 && image_point.x < info.width &&
            image_point.y >= 0 && image_point.y < info.height);
}

int MultiCameraRig::FindBestCameraForPoint(const cv::Point3f& sphere_point)
{
    int best_camera_id = -1;
    float best_dot_product = -1.0f;
    
    // Check each camera
    for (const auto& pair : cameras_) {
        int camera_id = pair.first;
        const CameraInfo& info = pair.second;
        
        // Transform sphere point to camera coordinates
        cv::Mat sphere_point_mat = (cv::Mat_<float>(4, 1) << 
            sphere_point.x, sphere_point.y, sphere_point.z, 1.0f);
        cv::Mat camera_point_mat = info.T_ref_cam * sphere_point_mat;
        
        // Check if point is in front of camera
        if (camera_point_mat.at<float>(2, 0) <= 0) {
            continue;
        }
        
        // Normalize camera direction vector (0, 0, 1) and point vector
        cv::Point3f camera_dir(0, 0, 1);
        cv::Point3f point_dir(
            camera_point_mat.at<float>(0, 0),
            camera_point_mat.at<float>(1, 0),
            camera_point_mat.at<float>(2, 0));
        float point_norm = std::sqrt(
            point_dir.x * point_dir.x + 
            point_dir.y * point_dir.y + 
            point_dir.z * point_dir.z);
        point_dir.x /= point_norm;
        point_dir.y /= point_norm;
        point_dir.z /= point_norm;
        
        // Calculate dot product (cosine of angle)
        float dot_product = camera_dir.x * point_dir.x + 
                           camera_dir.y * point_dir.y + 
                           camera_dir.z * point_dir.z;
        
        // Check if this camera is better
        if (dot_product > best_dot_product) {
            best_dot_product = dot_product;
            best_camera_id = camera_id;
        }
    }
    
    return best_camera_id;
}

cv::Point3f MultiCameraRig::TransformPoint(
    const cv::Point3f& point,
    int source_camera_id,
    int target_camera_id)
{
    // Check if cameras exist
    auto source_it = cameras_.find(source_camera_id);
    auto target_it = cameras_.find(target_camera_id);
    if (source_it == cameras_.end() || target_it == cameras_.end()) {
        std::cerr << "Source or target camera does not exist in the rig." << std::endl;
        return cv::Point3f();
    }
    
    // Get transforms
    const cv::Mat& T_ref_source = source_it->second.T_ref_cam;
    const cv::Mat& T_ref_target = target_it->second.T_ref_cam;
    
    // Calculate transform from source to target
    cv::Mat T_source_target = T_ref_target.inv() * T_ref_source;
    
    // Transform point
    cv::Mat point_mat = (cv::Mat_<float>(4, 1) << point.x, point.y, point.z, 1.0f);
    cv::Mat transformed_point_mat = T_source_target * point_mat;
    
    return cv::Point3f(
        transformed_point_mat.at<float>(0, 0),
        transformed_point_mat.at<float>(1, 0),
        transformed_point_mat.at<float>(2, 0));
}

cv::Mat MultiCameraRig::GetTransform(int source_camera_id, int target_camera_id)
{
    // Check if cameras exist
    auto source_it = cameras_.find(source_camera_id);
    auto target_it = cameras_.find(target_camera_id);
    if (source_it == cameras_.end() || target_it == cameras_.end()) {
        std::cerr << "Source or target camera does not exist in the rig." << std::endl;
        return cv::Mat();
    }
    
    // Get transforms
    const cv::Mat& T_ref_source = source_it->second.T_ref_cam;
    const cv::Mat& T_ref_target = target_it->second.T_ref_cam;
    
    // Calculate transform from source to target
    return T_ref_target.inv() * T_ref_source;
}

bool MultiCameraRig::UpdateTransform(
    int source_camera_id,
    int target_camera_id,
    const cv::Mat& transform)
{
    // Check if cameras exist
    auto source_it = cameras_.find(source_camera_id);
    auto target_it = cameras_.find(target_camera_id);
    if (source_it == cameras_.end() || target_it == cameras_.end()) {
        std::cerr << "Source or target camera does not exist in the rig." << std::endl;
        return false;
    }
    
    // Get target transform
    const cv::Mat& T_ref_target = target_it->second.T_ref_cam;
    
    // Calculate new source transform
    cv::Mat T_ref_source = T_ref_target * transform;
    
    // Update source transform
    source_it->second.T_ref_cam = T_ref_source;
    
    return true;
}

// Private helper methods

bool MultiCameraRig::CalibrateIndividualCameras(
    const std::vector<std::vector<cv::Mat>>& calibration_images,
    const cv::Size& pattern_size,
    float square_size)
{
    // Calibrate each camera individually
    int camera_idx = 0;
    for (auto& pair : cameras_) {
        CameraInfo& info = pair.second;
        const std::vector<cv::Mat>& images = calibration_images[camera_idx];
        
        // Prepare object points (3D points in real world space)
        std::vector<std::vector<cv::Point3f>> object_points;
        std::vector<cv::Point3f> pattern_points;
        for (int i = 0; i < pattern_size.height; ++i) {
            for (int j = 0; j < pattern_size.width; ++j) {
                pattern_points.push_back(cv::Point3f(j * square_size, i * square_size, 0));
            }
        }
        
        // Detect chessboard corners
        std::vector<std::vector<cv::Point2f>> image_points;
        for (const auto& image : images) {
            std::vector<cv::Point2f> corners;
            bool found = cv::findChessboardCorners(image, pattern_size, corners);
            if (found) {
                // Refine corner locations
                cv::Mat gray;
                if (image.channels() == 3) {
                    cv::cvtColor(image, gray, cv::COLOR_BGR2GRAY);
                } else {
                    gray = image.clone();
                }
                cv::cornerSubPix(gray, corners, cv::Size(11, 11), cv::Size(-1, -1),
                                cv::TermCriteria(cv::TermCriteria::EPS + cv::TermCriteria::COUNT, 30, 0.1));
                
                image_points.push_back(corners);
                object_points.push_back(pattern_points);
            }
        }
        
        // Check if we have enough points
        if (image_points.size() < 3) {
            std::cerr << "Not enough calibration images with detected pattern for camera " 
                      << info.id << std::endl;
            return false;
        }
        
        // Calibrate camera
        cv::Mat K, distCoef;
        std::vector<cv::Mat> rvecs, tvecs;
        double rms = cv::calibrateCamera(object_points, image_points, images[0].size(),
                                        K, distCoef, rvecs, tvecs);
        
        // Update camera info
        info.K = K;
        info.distCoef = distCoef;
        
        // Calculate field of view
        info.fov_horizontal = 2 * atan(info.width / (2 * K.at<double>(0, 0))) * 180 / CV_PI;
        info.fov_vertical = 2 * atan(info.height / (2 * K.at<double>(1, 1))) * 180 / CV_PI;
        
        std::cout << "Camera " << info.id << " calibrated with RMS error: " << rms << std::endl;
        std::cout << "Field of view: " << info.fov_horizontal << "° x " 
                  << info.fov_vertical << "°" << std::endl;
        
        camera_idx++;
    }
    
    return true;
}

bool MultiCameraRig::CalibrateCameraPairs(
    const std::vector<std::vector<cv::Mat>>& calibration_images,
    const cv::Size& pattern_size)
{
    // If we only have one camera, nothing to do
    if (cameras_.size() <= 1) {
        return true;
    }
    
    // Set reference camera transform to identity
    cameras_[reference_camera_id_].T_ref_cam = cv::Mat::eye(4, 4, CV_32F);
    
    // For each camera pair (reference and other)
    int camera_idx = 0;
    for (auto& pair : cameras_) {
        int camera_id = pair.first;
        CameraInfo& info = pair.second;
        
        // Skip reference camera
        if (camera_id == reference_camera_id_) {
            camera_idx++;
            continue;
        }
        
        // Get calibration images for this camera and reference
        const std::vector<cv::Mat>& ref_images = calibration_images[0]; // Assuming reference is first
        const std::vector<cv::Mat>& cam_images = calibration_images[camera_idx];
        
        // Find common frames where pattern is visible in both cameras
        std::vector<int> common_frames;
        for (size_t i = 0; i < std::min(ref_images.size(), cam_images.size()); ++i) {
            std::vector<cv::Point2f> ref_corners, cam_corners;
            bool ref_found = cv::findChessboardCorners(ref_images[i], pattern_size, ref_corners);
            bool cam_found = cv::findChessboardCorners(cam_images[i], pattern_size, cam_corners);
            
            if (ref_found && cam_found) {
                common_frames.push_back(i);
            }
        }
        
        // Check if we have enough common frames
        if (common_frames.size() < 5) {
            std::cerr << "Not enough common frames with detected pattern for camera pair "
                      << reference_camera_id_ << " and " << camera_id << std::endl;
            return false;
        }
        
        // Prepare object points and image points
        std::vector<std::vector<cv::Point3f>> object_points;
        std::vector<std::vector<cv::Point2f>> ref_image_points, cam_image_points;
        
        for (int frame : common_frames) {
            // Prepare object points
            std::vector<cv::Point3f> pattern_points;
            for (int i = 0; i < pattern_size.height; ++i) {
                for (int j = 0; j < pattern_size.width; ++j) {
                    pattern_points.push_back(cv::Point3f(j, i, 0)); // Unit square size
                }
            }
            object_points.push_back(pattern_points);
            
            // Detect and refine corners
            std::vector<cv::Point2f> ref_corners, cam_corners;
            cv::findChessboardCorners(ref_images[frame], pattern_size, ref_corners);
            cv::findChessboardCorners(cam_images[frame], pattern_size, cam_corners);
            
            cv::Mat ref_gray, cam_gray;
            if (ref_images[frame].channels() == 3) {
                cv::cvtColor(ref_images[frame], ref_gray, cv::COLOR_BGR2GRAY);
                cv::cvtColor(cam_images[frame], cam_gray, cv::COLOR_BGR2GRAY);
            } else {
                ref_gray = ref_images[frame].clone();
                cam_gray = cam_images[frame].clone();
            }
            
            cv::cornerSubPix(ref_gray, ref_corners, cv::Size(11, 11), cv::Size(-1, -1),
                            cv::TermCriteria(cv::TermCriteria::EPS + cv::TermCriteria::COUNT, 30, 0.1));
            cv::cornerSubPix(cam_gray, cam_corners, cv::Size(11, 11), cv::Size(-1, -1),
                            cv::TermCriteria(cv::TermCriteria::EPS + cv::TermCriteria::COUNT, 30, 0.1));
            
            ref_image_points.push_back(ref_corners);
            cam_image_points.push_back(cam_corners);
        }
        
        // Get camera matrices and distortion coefficients
        const cv::Mat& K_ref = cameras_[reference_camera_id_].K;
        const cv::Mat& D_ref = cameras_[reference_camera_id_].distCoef;
        const cv::Mat& K_cam = info.K;
        const cv::Mat& D_cam = info.distCoef;
        
        // Stereo calibration
        cv::Mat R, T, E, F;
        double rms = cv::stereoCalibrate(
            object_points, ref_image_points, cam_image_points,
            K_ref, D_ref, K_cam, D_cam,
            ref_images[0].size(), R, T, E, F,
            cv::CALIB_FIX_INTRINSIC,
            cv::TermCriteria(cv::TermCriteria::COUNT + cv::TermCriteria::EPS, 100, 1e-5));
        
        // Create 4x4 transformation matrix
        cv::Mat T_ref_cam = cv::Mat::eye(4, 4, CV_32F);
        R.copyTo(T_ref_cam(cv::Rect(0, 0, 3, 3)));
        T.copyTo(T_ref_cam(cv::Rect(3, 0, 1, 3)));
        
        // Update camera info
        info.T_ref_cam = T_ref_cam;
        
        std::cout << "Camera pair " << reference_camera_id_ << " and " << camera_id
                  << " calibrated with RMS error: " << rms << std::endl;
        
        camera_idx++;
    }
    
    return true;
}

bool MultiCameraRig::OptimizeRigCalibration()
{
    // For now, just a placeholder for future implementation
    // This would involve bundle adjustment or similar global optimization
    
    std::cout << "Rig calibration optimization not implemented yet." << std::endl;
    return true;
}

cv::Mat MultiCameraRig::CreateSphericalMap(
    int camera_id,
    const cv::Size& panorama_size)
{
    // Check if camera exists
    auto it = cameras_.find(camera_id);
    if (it == cameras_.end()) {
        std::cerr << "Camera with ID " << camera_id << " does not exist in the rig." << std::endl;
        return cv::Mat();
    }
    
    // Get camera info
    const CameraInfo& info = it->second;
    
    // Create map
    cv::Mat map = cv::Mat::zeros(panorama_size, CV_32FC2);
    
    // For each pixel in the panorama
    for (int y = 0; y < panorama_size.height; ++y) {
        for (int x = 0; x < panorama_size.width; ++x) {
            // Convert panorama coordinates to spherical coordinates
            float phi = 2 * CV_PI * x / panorama_size.width;
            float theta = CV_PI * y / panorama_size.height;
            
            // Convert spherical coordinates to 3D point on unit sphere
            cv::Point3f sphere_point(
                sin(theta) * cos(phi),
                sin(theta) * sin(phi),
                cos(theta));
            
            // Check if point is visible to camera
            if (IsPointVisibleToCamera(sphere_point, camera_id)) {
                // Project to camera image
                cv::Point2f image_point = SphereToCameraProjection(sphere_point, camera_id);
                
                // Store mapping
                map.at<cv::Vec2f>(y, x) = cv::Vec2f(image_point.x, image_point.y);
            } else {
                // Point not visible, mark with negative coordinates
                map.at<cv::Vec2f>(y, x) = cv::Vec2f(-1, -1);
            }
        }
    }
    
    return map;
}

cv::Point3f MultiCameraRig::CameraToSphere(
    const cv::Point2f& point,
    int camera_id)
{
    // Check if camera exists
    auto it = cameras_.find(camera_id);
    if (it == cameras_.end()) {
        std::cerr << "Camera with ID " << camera_id << " does not exist in the rig." << std::endl;
        return cv::Point3f();
    }
    
    // Get camera info
    const CameraInfo& info = it->second;
    
    // Undistort point
    std::vector<cv::Point2f> points_in = {point};
    std::vector<cv::Point2f> points_out;
    cv::undistortPoints(points_in, points_out, info.K, info.distCoef);
    
    // Convert to normalized camera coordinates
    cv::Point3f camera_point(points_out[0].x, points_out[0].y, 1.0f);
    
    // Normalize to unit vector
    float norm = std::sqrt(
        camera_point.x * camera_point.x + 
        camera_point.y * camera_point.y + 
        camera_point.z * camera_point.z);
    camera_point.x /= norm;
    camera_point.y /= norm;
    camera_point.z /= norm;
    
    // Transform to reference frame
    cv::Mat camera_point_mat = (cv::Mat_<float>(4, 1) << 
        camera_point.x, camera_point.y, camera_point.z, 1.0f);
    cv::Mat ref_point_mat = info.T_ref_cam.inv() * camera_point_mat;
    
    // Return as unit vector on sphere
    cv::Point3f ref_point(
        ref_point_mat.at<float>(0, 0),
        ref_point_mat.at<float>(1, 0),
        ref_point_mat.at<float>(2, 0));
    
    norm = std::sqrt(
        ref_point.x * ref_point.x + 
        ref_point.y * ref_point.y + 
        ref_point.z * ref_point.z);
    
    return cv::Point3f(
        ref_point.x / norm,
        ref_point.y / norm,
        ref_point.z / norm);
}

cv::Point2f MultiCameraRig::SphereToCameraProjection(
    const cv::Point3f& sphere_point,
    int camera_id)
{
    // Check if camera exists
    auto it = cameras_.find(camera_id);
    if (it == cameras_.end()) {
        std::cerr << "Camera with ID " << camera_id << " does not exist in the rig." << std::endl;
        return cv::Point2f();
    }
    
    // Get camera info
    const CameraInfo& info = it->second;
    
    // Transform sphere point to camera coordinates
    cv::Mat sphere_point_mat = (cv::Mat_<float>(4, 1) << 
        sphere_point.x, sphere_point.y, sphere_point.z, 1.0f);
    cv::Mat camera_point_mat = info.T_ref_cam * sphere_point_mat;
    
    // Convert to 3D point
    cv::Point3f camera_point(
        camera_point_mat.at<float>(0, 0),
        camera_point_mat.at<float>(1, 0),
        camera_point_mat.at<float>(2, 0));
    
    // Check if point is in front of camera
    if (camera_point.z <= 0) {
        return cv::Point2f(-1, -1); // Point is behind camera
    }
    
    // Project to normalized image coordinates
    cv::Point2f normalized_point(
        camera_point.x / camera_point.z,
        camera_point.y / camera_point.z);
    
    // Apply distortion and intrinsic matrix
    std::vector<cv::Point2f> points_in = {normalized_point};
    std::vector<cv::Point2f> points_out;
    
    // Project points
    cv::Mat rvec = cv::Mat::zeros(3, 1, CV_64F); // No rotation
    cv::Mat tvec = cv::Mat::zeros(3, 1, CV_64F); // No translation
    cv::projectPoints(points_in, rvec, tvec, info.K, info.distCoef, points_out);
    
    return points_out[0];
}

} // namespace ORB_SLAM3

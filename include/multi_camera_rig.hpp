#ifndef MULTI_CAMERA_RIG_HPP
#define MULTI_CAMERA_RIG_HPP

#include <vector>
#include <string>
#include <map>
#include <memory>
#include "opencv2/core/mat.hpp"
#include "opencv2/calib3d.hpp"

namespace ORB_SLAM3
{

/**
 * @brief Class for managing a multi-camera rig configuration
 * 
 * This class handles the configuration, calibration, and coordinate
 * transformations for a multi-camera setup, such as the 4-camera
 * arrangement in a VR headset.
 */
class MultiCameraRig
{
public:
    /**
     * @brief Camera information structure
     * 
     * Contains all parameters needed to define a camera in the rig
     */
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
    
    /**
     * @brief Default constructor
     */
    MultiCameraRig();
    
    /**
     * @brief Constructor with reference camera ID
     * 
     * @param reference_camera_id ID of the camera to use as reference frame
     */
    explicit MultiCameraRig(int reference_camera_id);
    
    /**
     * @brief Destructor
     */
    ~MultiCameraRig();
    
    /**
     * @brief Add a camera to the rig
     * 
     * @param camera Camera information
     * @return true if camera was added successfully, false otherwise
     */
    bool AddCamera(const CameraInfo& camera);
    
    /**
     * @brief Remove a camera from the rig
     * 
     * @param camera_id ID of the camera to remove
     * @return true if camera was removed successfully, false otherwise
     */
    bool RemoveCamera(int camera_id);
    
    /**
     * @brief Get information about a specific camera
     * 
     * @param camera_id ID of the camera
     * @return Camera information
     */
    CameraInfo GetCameraInfo(int camera_id) const;
    
    /**
     * @brief Get all cameras in the rig
     * 
     * @return Vector of camera information structures
     */
    std::vector<CameraInfo> GetAllCameras() const;
    
    /**
     * @brief Get the reference camera ID
     * 
     * @return ID of the reference camera
     */
    int GetReferenceCameraId() const;
    
    /**
     * @brief Set the reference camera ID
     * 
     * @param camera_id ID of the camera to use as reference
     * @return true if reference was set successfully, false otherwise
     */
    bool SetReferenceCameraId(int camera_id);
    
    /**
     * @brief Calibrate the camera rig using a set of calibration images
     * 
     * @param calibration_images Vector of image sets, one set per camera
     * @param pattern_size Size of the calibration pattern (e.g., chessboard)
     * @param square_size Physical size of each square in the pattern
     * @return true if calibration was successful, false otherwise
     */
    bool CalibrateRig(
        const std::vector<std::vector<cv::Mat>>& calibration_images,
        const cv::Size& pattern_size,
        float square_size);
    
    /**
     * @brief Load camera rig calibration from a file
     * 
     * @param filename Path to the calibration file
     * @return true if calibration was loaded successfully, false otherwise
     */
    bool LoadCalibration(const std::string& filename);
    
    /**
     * @brief Save camera rig calibration to a file
     * 
     * @param filename Path to the calibration file
     * @return true if calibration was saved successfully, false otherwise
     */
    bool SaveCalibration(const std::string& filename) const;
    
    /**
     * @brief Project a set of images to a spherical panorama
     * 
     * @param images Vector of images, one per camera
     * @param resolution Resolution of the output panorama
     * @return Spherical panorama image
     */
    cv::Mat ProjectToSpherical(
        const std::vector<cv::Mat>& images,
        const cv::Size& resolution = cv::Size(4096, 2048));
    
    /**
     * @brief Project 2D points from a camera to 3D points on a unit sphere
     * 
     * @param points 2D points in camera image coordinates
     * @param camera_id ID of the camera
     * @return 3D points on the unit sphere
     */
    std::vector<cv::Point3f> ProjectPointsToSphere(
        const std::vector<cv::Point2f>& points,
        int camera_id);
    
    /**
     * @brief Project 3D points on a unit sphere to 2D points in a camera
     * 
     * @param sphere_points 3D points on the unit sphere
     * @param camera_id ID of the camera
     * @return 2D points in camera image coordinates
     */
    std::vector<cv::Point2f> ProjectSphericalPointsToCamera(
        const std::vector<cv::Point3f>& sphere_points,
        int camera_id);
    
    /**
     * @brief Check if a 3D point on the unit sphere is visible to a camera
     * 
     * @param sphere_point 3D point on the unit sphere
     * @param camera_id ID of the camera
     * @return true if the point is visible, false otherwise
     */
    bool IsPointVisibleToCamera(
        const cv::Point3f& sphere_point,
        int camera_id);
    
    /**
     * @brief Find which camera can best observe a 3D point on the unit sphere
     * 
     * @param sphere_point 3D point on the unit sphere
     * @return ID of the best camera, or -1 if no camera can see the point
     */
    int FindBestCameraForPoint(const cv::Point3f& sphere_point);
    
    /**
     * @brief Transform a point from one camera's coordinate system to another
     * 
     * @param point 3D point in source camera coordinates
     * @param source_camera_id ID of the source camera
     * @param target_camera_id ID of the target camera
     * @return 3D point in target camera coordinates
     */
    cv::Point3f TransformPoint(
        const cv::Point3f& point,
        int source_camera_id,
        int target_camera_id);
    
    /**
     * @brief Get the transform from one camera to another
     * 
     * @param source_camera_id ID of the source camera
     * @param target_camera_id ID of the target camera
     * @return Transformation matrix from source to target
     */
    cv::Mat GetTransform(int source_camera_id, int target_camera_id);
    
    /**
     * @brief Update the transform between two cameras
     * 
     * @param source_camera_id ID of the source camera
     * @param target_camera_id ID of the target camera
     * @param transform Transformation matrix from source to target
     * @return true if transform was updated successfully, false otherwise
     */
    bool UpdateTransform(
        int source_camera_id,
        int target_camera_id,
        const cv::Mat& transform);
    
private:
    // Map of camera ID to camera information
    std::map<int, CameraInfo> cameras_;
    
    // ID of the reference camera
    int reference_camera_id_;
    
    // Helper methods for calibration
    bool CalibrateIndividualCameras(
        const std::vector<std::vector<cv::Mat>>& calibration_images,
        const cv::Size& pattern_size,
        float square_size);
    
    bool CalibrateCameraPairs(
        const std::vector<std::vector<cv::Mat>>& calibration_images,
        const cv::Size& pattern_size);
    
    bool OptimizeRigCalibration();
    
    // Helper methods for projection
    cv::Mat CreateSphericalMap(
        int camera_id,
        const cv::Size& panorama_size);
    
    // Helper methods for coordinate transformations
    cv::Point3f CameraToSphere(
        const cv::Point2f& point,
        int camera_id);
    
    cv::Point2f SphereToCameraProjection(
        const cv::Point3f& sphere_point,
        int camera_id);
};

} // namespace ORB_SLAM3

#endif // MULTI_CAMERA_RIG_HPP

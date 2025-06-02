#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <memory>
#include <vector>
#include <opencv2/core/core.hpp>
#include <opencv2/highgui/highgui.hpp>

#include "../include/multi_camera_tracking.hpp"
#include "../include/multi_camera_rig.hpp"
#include "../ORB_SLAM3/include/System.h"

using namespace ORB_SLAM3;
using namespace testing;

// Mock classes
class MockORBVocabulary : public ORBVocabulary {
public:
    MockORBVocabulary() {}
};

class MockFrameDrawer : public FrameDrawer {
public:
    MockFrameDrawer(Atlas* atlas) : FrameDrawer(atlas) {}
};

class MockMapDrawer : public MapDrawer {
public:
    MockMapDrawer(Atlas* atlas, const string& strSettingPath) : MapDrawer(atlas, strSettingPath) {}
};

class MockKeyFrameDatabase : public KeyFrameDatabase {
public:
    MockKeyFrameDatabase(const ORBVocabulary& voc) : KeyFrameDatabase(voc) {}
};

class MultiCameraTrackingTest : public ::testing::Test {
protected:
    void SetUp() override {
        // Create a multi-camera rig
        setupMultiCameraRig();
        
        // Create mock objects for ORB-SLAM3
        atlas_ = std::make_shared<Atlas>(0);
        vocabulary_ = std::make_shared<MockORBVocabulary>();
        frame_drawer_ = std::make_shared<MockFrameDrawer>(atlas_.get());
        map_drawer_ = std::make_shared<MockMapDrawer>(atlas_.get(), "");
        kf_database_ = std::make_shared<MockKeyFrameDatabase>(*vocabulary_);
        
        // Create configuration for multi-camera tracking
        MultiCameraTracking::Config config;
        config.enable_cross_camera_matching = true;
        config.use_spherical_model = true;
        config.parallel_feature_extraction = true;
        
        // Create multi-camera tracking
        tracking_ = std::make_unique<MultiCameraTracking>(
            nullptr,  // System* (not needed for tests)
            vocabulary_.get(),
            frame_drawer_.get(),
            map_drawer_.get(),
            atlas_.get(),
            kf_database_.get(),
            "",  // Settings path (not needed for tests)
            System::MONOCULAR,
            rig_,
            config
        );
    }
    
    void TearDown() override {
        tracking_.reset();
        kf_database_.reset();
        map_drawer_.reset();
        frame_drawer_.reset();
        vocabulary_.reset();
        atlas_.reset();
    }
    
    void setupMultiCameraRig() {
        // Create a 4-camera rig for VR headset
        rig_ = MultiCameraRig(0);  // Reference camera ID = 0
        
        // Front camera (reference)
        MultiCameraRig::CameraInfo frontCamera;
        frontCamera.id = 0;
        frontCamera.K = (cv::Mat_<float>(3, 3) << 
            500.0f, 0.0f, 320.0f,
            0.0f, 500.0f, 240.0f,
            0.0f, 0.0f, 1.0f);
        frontCamera.distCoef = cv::Mat::zeros(1, 5, CV_32F);
        frontCamera.T_ref_cam = cv::Mat::eye(4, 4, CV_32F);
        frontCamera.fps = 30.0f;
        frontCamera.width = 640;
        frontCamera.height = 480;
        frontCamera.model = "pinhole";
        frontCamera.fov_horizontal = 90.0f;
        frontCamera.fov_vertical = 70.0f;
        rig_.AddCamera(frontCamera);
        
        // Right camera
        MultiCameraRig::CameraInfo rightCamera;
        rightCamera.id = 1;
        rightCamera.K = frontCamera.K.clone();
        rightCamera.distCoef = frontCamera.distCoef.clone();
        rightCamera.T_ref_cam = (cv::Mat_<float>(4, 4) << 
            0.0f, 0.0f, 1.0f, 0.1f,
            0.0f, 1.0f, 0.0f, 0.0f,
            -1.0f, 0.0f, 0.0f, 0.0f,
            0.0f, 0.0f, 0.0f, 1.0f);
        rightCamera.fps = 30.0f;
        rightCamera.width = 640;
        rightCamera.height = 480;
        rightCamera.model = "pinhole";
        rightCamera.fov_horizontal = 90.0f;
        rightCamera.fov_vertical = 70.0f;
        rig_.AddCamera(rightCamera);
        
        // Back camera
        MultiCameraRig::CameraInfo backCamera;
        backCamera.id = 2;
        backCamera.K = frontCamera.K.clone();
        backCamera.distCoef = frontCamera.distCoef.clone();
        backCamera.T_ref_cam = (cv::Mat_<float>(4, 4) << 
            -1.0f, 0.0f, 0.0f, 0.0f,
            0.0f, 1.0f, 0.0f, 0.0f,
            0.0f, 0.0f, -1.0f, -0.1f,
            0.0f, 0.0f, 0.0f, 1.0f);
        backCamera.fps = 30.0f;
        backCamera.width = 640;
        backCamera.height = 480;
        backCamera.model = "pinhole";
        backCamera.fov_horizontal = 90.0f;
        backCamera.fov_vertical = 70.0f;
        rig_.AddCamera(backCamera);
        
        // Left camera
        MultiCameraRig::CameraInfo leftCamera;
        leftCamera.id = 3;
        leftCamera.K = frontCamera.K.clone();
        leftCamera.distCoef = frontCamera.distCoef.clone();
        leftCamera.T_ref_cam = (cv::Mat_<float>(4, 4) << 
            0.0f, 0.0f, -1.0f, -0.1f,
            0.0f, 1.0f, 0.0f, 0.0f,
            1.0f, 0.0f, 0.0f, 0.0f,
            0.0f, 0.0f, 0.0f, 1.0f);
        leftCamera.fps = 30.0f;
        leftCamera.width = 640;
        leftCamera.height = 480;
        leftCamera.model = "pinhole";
        leftCamera.fov_horizontal = 90.0f;
        leftCamera.fov_vertical = 70.0f;
        rig_.AddCamera(leftCamera);
    }
    
    // Helper method to create test images
    std::vector<cv::Mat> createTestImages() {
        std::vector<cv::Mat> images;
        
        // Create a test pattern for each camera
        for (int i = 0; i < 4; ++i) {
            cv::Mat image(480, 640, CV_8UC1);
            
            // Draw a checkerboard pattern
            for (int y = 0; y < image.rows; y += 40) {
                for (int x = 0; x < image.cols; x += 40) {
                    cv::Rect rect(x, y, 20, 20);
                    image(rect).setTo(255);
                    
                    rect = cv::Rect(x + 20, y + 20, 20, 20);
                    image(rect).setTo(255);
                }
            }
            
            // Draw camera ID
            cv::putText(image, std::to_string(i), cv::Point(320, 240),
                       cv::FONT_HERSHEY_SIMPLEX, 5.0, cv::Scalar(128), 10);
            
            images.push_back(image);
        }
        
        return images;
    }
    
    MultiCameraRig rig_;
    std::unique_ptr<MultiCameraTracking> tracking_;
    std::shared_ptr<Atlas> atlas_;
    std::shared_ptr<MockORBVocabulary> vocabulary_;
    std::shared_ptr<MockFrameDrawer> frame_drawer_;
    std::shared_ptr<MockMapDrawer> map_drawer_;
    std::shared_ptr<MockKeyFrameDatabase> kf_database_;
};

// Test initialization
TEST_F(MultiCameraTrackingTest, Initialization) {
    // Verify rig configuration
    EXPECT_EQ(tracking_->GetMultiCameraRig().GetAllCameras().size(), 4);
    EXPECT_EQ(tracking_->GetMultiCameraRig().GetReferenceCameraId(), 0);
    
    // Verify active camera
    EXPECT_EQ(tracking_->GetActiveCameraId(), 0);
    
    // Verify feature extractors
    EXPECT_EQ(tracking_->GetFeatureExtractors().size(), 4);
}

// Test camera selection
TEST_F(MultiCameraTrackingTest, CameraSelection) {
    // Test setting active camera
    tracking_->SetActiveCameraId(1);
    EXPECT_EQ(tracking_->GetActiveCameraId(), 1);
    
    tracking_->SetActiveCameraId(2);
    EXPECT_EQ(tracking_->GetActiveCameraId(), 2);
    
    // Test invalid camera ID
    tracking_->SetActiveCameraId(10);  // Should not change
    EXPECT_EQ(tracking_->GetActiveCameraId(), 2);
}

// Test configuration
TEST_F(MultiCameraTrackingTest, Configuration) {
    // Get current configuration
    MultiCameraTracking::Config config = tracking_->GetConfig();
    
    // Modify configuration
    config.enable_cross_camera_matching = false;
    config.parallel_feature_extraction = false;
    
    // Set new configuration
    tracking_->SetConfig(config);
    
    // Verify configuration
    MultiCameraTracking::Config new_config = tracking_->GetConfig();
    EXPECT_FALSE(new_config.enable_cross_camera_matching);
    EXPECT_FALSE(new_config.parallel_feature_extraction);
}

// Test camera visibility
TEST_F(MultiCameraTrackingTest, CameraVisibility) {
    // Create a 3D point in front of the reference camera
    cv::Point3f world_point(0.0f, 0.0f, 1.0f);
    
    // Get cameras that can see this point
    std::vector<int> visible_cameras = tracking_->GetCamerasForPoint(world_point);
    
    // Front camera should see this point
    EXPECT_TRUE(std::find(visible_cameras.begin(), visible_cameras.end(), 0) != visible_cameras.end());
    
    // Back camera should not see this point
    EXPECT_TRUE(std::find(visible_cameras.begin(), visible_cameras.end(), 2) == visible_cameras.end());
    
    // Create a point to the right
    cv::Point3f right_point(0.0f, 0.0f, -1.0f);
    
    // Get best camera for this point
    int best_camera = tracking_->GetBestCameraForPoint(right_point);
    
    // Back camera should be best for this point
    EXPECT_EQ(best_camera, 2);
}

// Test camera transforms
TEST_F(MultiCameraTrackingTest, CameraTransforms) {
    // Get transforms between cameras
    cv::Mat T_0_1 = rig_.GetTransform(0, 1);  // Transform from camera 0 to camera 1
    cv::Mat T_1_0 = rig_.GetTransform(1, 0);  // Transform from camera 1 to camera 0
    
    // Verify that transforms are inverses
    cv::Mat identity = T_0_1 * T_1_0;
    
    // Check that result is close to identity
    for (int i = 0; i < 3; ++i) {
        for (int j = 0; j < 3; ++j) {
            if (i == j) {
                EXPECT_NEAR(identity.at<float>(i, j), 1.0f, 0.01f);
            } else {
                EXPECT_NEAR(identity.at<float>(i, j), 0.0f, 0.01f);
            }
        }
    }
}

// Test spherical projection
TEST_F(MultiCameraTrackingTest, SphericalProjection) {
    // Create a 3D point on the unit sphere
    cv::Point3f sphere_point(1.0f, 0.0f, 0.0f);  // Point along X axis
    
    // Project to camera 0 (front camera)
    std::vector<cv::Point2f> projected_points = rig_.ProjectSphericalPointsToCamera({sphere_point}, 0);
    
    // Point should be outside the field of view of camera 0
    EXPECT_TRUE(projected_points.empty() || 
               projected_points[0].x < 0 || projected_points[0].x >= 640 ||
               projected_points[0].y < 0 || projected_points[0].y >= 480);
    
    // Project to camera 1 (right camera)
    projected_points = rig_.ProjectSphericalPointsToCamera({sphere_point}, 1);
    
    // Point should be visible in camera 1
    EXPECT_FALSE(projected_points.empty());
    if (!projected_points.empty()) {
        EXPECT_GE(projected_points[0].x, 0);
        EXPECT_LT(projected_points[0].x, 640);
        EXPECT_GE(projected_points[0].y, 0);
        EXPECT_LT(projected_points[0].y, 480);
    }
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}

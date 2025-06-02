#include <iostream>
#include <vector>
#include <string>
#include <chrono>
#include <thread>
#include <opencv2/core/core.hpp>
#include <opencv2/highgui/highgui.hpp>
#include <opencv2/imgproc/imgproc.hpp>

#include "../include/multi_camera_tracking.hpp"
#include "../include/multi_camera_rig.hpp"

using namespace std;
using namespace ORB_SLAM3;

void LoadImages(const string &strPathToSequence, vector<string> &vstrImageFilenames,
                vector<double> &vTimestamps);

int main(int argc, char **argv)
{
    if(argc != 3)
    {
        cerr << endl << "Usage: ./test_multi_camera_tracking path_to_settings path_to_sequence" << endl;
        return 1;
    }

    // Load settings
    string strSettingsFile = argv[1];
    string strSequence = argv[2];

    // Create multi-camera rig
    MultiCameraRig rig;

    // Add cameras to the rig
    // This is a simplified example with 4 cameras in a VR headset configuration
    
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
    rig.AddCamera(frontCamera);
    
    // Right camera
    MultiCameraRig::CameraInfo rightCamera;
    rightCamera.id = 1;
    rightCamera.K = frontCamera.K.clone();
    rightCamera.distCoef = frontCamera.distCoef.clone();
    rightCamera.T_ref_cam = (cv::Mat_<float>(4, 4) << 
        0.0f, 0.0f, 1.0f, 0.0f,
        0.0f, 1.0f, 0.0f, 0.0f,
        -1.0f, 0.0f, 0.0f, 0.0f,
        0.0f, 0.0f, 0.0f, 1.0f);
    rightCamera.fps = 30.0f;
    rightCamera.width = 640;
    rightCamera.height = 480;
    rightCamera.model = "pinhole";
    rightCamera.fov_horizontal = 90.0f;
    rightCamera.fov_vertical = 70.0f;
    rig.AddCamera(rightCamera);
    
    // Back camera
    MultiCameraRig::CameraInfo backCamera;
    backCamera.id = 2;
    backCamera.K = frontCamera.K.clone();
    backCamera.distCoef = frontCamera.distCoef.clone();
    backCamera.T_ref_cam = (cv::Mat_<float>(4, 4) << 
        -1.0f, 0.0f, 0.0f, 0.0f,
        0.0f, 1.0f, 0.0f, 0.0f,
        0.0f, 0.0f, -1.0f, 0.0f,
        0.0f, 0.0f, 0.0f, 1.0f);
    backCamera.fps = 30.0f;
    backCamera.width = 640;
    backCamera.height = 480;
    backCamera.model = "pinhole";
    backCamera.fov_horizontal = 90.0f;
    backCamera.fov_vertical = 70.0f;
    rig.AddCamera(backCamera);
    
    // Left camera
    MultiCameraRig::CameraInfo leftCamera;
    leftCamera.id = 3;
    leftCamera.K = frontCamera.K.clone();
    leftCamera.distCoef = frontCamera.distCoef.clone();
    leftCamera.T_ref_cam = (cv::Mat_<float>(4, 4) << 
        0.0f, 0.0f, -1.0f, 0.0f,
        0.0f, 1.0f, 0.0f, 0.0f,
        1.0f, 0.0f, 0.0f, 0.0f,
        0.0f, 0.0f, 0.0f, 1.0f);
    leftCamera.fps = 30.0f;
    leftCamera.width = 640;
    leftCamera.height = 480;
    leftCamera.model = "pinhole";
    leftCamera.fov_horizontal = 90.0f;
    leftCamera.fov_vertical = 70.0f;
    rig.AddCamera(leftCamera);
    
    // Set reference camera
    rig.SetReferenceCameraId(0);
    
    // Save calibration
    rig.SaveCalibration("multi_camera_calibration.json");
    
    cout << "Multi-camera rig created with " << rig.GetAllCameras().size() << " cameras" << endl;
    
    // Create ORB-SLAM3 system
    // Note: This is a simplified example and would need to be adapted for a real test
    // In a real test, we would create a full ORB-SLAM3 system and replace the Tracking module
    
    // For this test, we'll just simulate the processing of a sequence of images
    
    // Load image filenames and timestamps
    vector<string> vstrImageFilenames;
    vector<double> vTimestamps;
    LoadImages(strSequence, vstrImageFilenames, vTimestamps);
    
    // Main loop
    int nImages = vstrImageFilenames.size();
    
    cout << "Starting processing sequence with " << nImages << " images..." << endl;
    
    // Process each frame
    for (int i = 0; i < nImages; i++)
    {
        // Load images for all cameras
        // In a real test, we would load actual images from all cameras
        // For this test, we'll just simulate by using the same image for all cameras
        
        cv::Mat imRGB = cv::imread(vstrImageFilenames[i], cv::IMREAD_COLOR);
        
        if (imRGB.empty())
        {
            cerr << "Failed to load image: " << vstrImageFilenames[i] << endl;
            return 1;
        }
        
        // Create a vector of images, one for each camera
        vector<cv::Mat> vImages(4, imRGB);
        
        // In a real test, we would process these images with the MultiCameraTracking class
        // For this test, we'll just print some information
        
        cout << "Processing frame " << i << "/" << nImages - 1 << " with timestamp " << vTimestamps[i] << endl;
        
        // Simulate some processing time
        std::this_thread::sleep_for(std::chrono::milliseconds(30));
    }
    
    cout << "Sequence processing finished!" << endl;
    
    return 0;
}

void LoadImages(const string &strPathToSequence, vector<string> &vstrImageFilenames, vector<double> &vTimestamps)
{
    // This is a simplified version that assumes a specific directory structure
    // In a real test, this would be adapted to the actual dataset format
    
    ifstream fTimes;
    string strPathTimeFile = strPathToSequence + "/times.txt";
    fTimes.open(strPathTimeFile.c_str());
    
    if (!fTimes.is_open())
    {
        cerr << "Could not open times file: " << strPathTimeFile << endl;
        return;
    }
    
    while (!fTimes.eof())
    {
        string s;
        getline(fTimes, s);
        if (!s.empty())
        {
            stringstream ss;
            ss << s;
            double t;
            ss >> t;
            vTimestamps.push_back(t);
        }
    }
    
    fTimes.close();
    
    string strPrefixLeft = strPathToSequence + "/image_0/";
    
    const int nTimes = vTimestamps.size();
    vstrImageFilenames.resize(nTimes);
    
    for (int i = 0; i < nTimes; i++)
    {
        stringstream ss;
        ss << setfill('0') << setw(6) << i;
        vstrImageFilenames[i] = strPrefixLeft + ss.str() + ".png";
    }
}

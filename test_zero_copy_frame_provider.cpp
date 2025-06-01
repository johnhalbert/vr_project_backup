#include "include/zero_copy_frame_provider.hpp"
#include <iostream>
#include <chrono>
#include <thread>
#include <opencv2/highgui.hpp>
#include <opencv2/imgproc.hpp>

using namespace ORB_SLAM3;

// Test configuration
const std::string TEST_DEVICE = "/dev/video0";
const int TEST_WIDTH = 640;
const int TEST_HEIGHT = 480;
const int TEST_FPS = 30;
const std::string TEST_FORMAT = "YUYV";
const int TEST_BUFFER_COUNT = 4;
const bool TEST_ZERO_COPY = true;
const int TEST_DURATION_SEC = 10;

void displayFrameInfo(const ZeroCopyFrameProvider::FrameMetadata& metadata) {
    std::cout << "Frame ID: " << metadata.frame_id
              << ", Camera: " << metadata.camera_id
              << ", Timestamp: " << metadata.timestamp
              << ", Size: " << metadata.width << "x" << metadata.height
              << ", Format: " << metadata.pixel_format
              << ", DMA FD: " << metadata.dma_fd
              << ", Is Keyframe: " << (metadata.is_keyframe ? "Yes" : "No")
              << std::endl;
}

int main(int argc, char** argv) {
    std::cout << "ZeroCopyFrameProvider Test Application" << std::endl;
    std::cout << "=====================================" << std::endl;
    
    // Check if device exists
    std::string device_path = TEST_DEVICE;
    if (argc > 1) {
        device_path = argv[1];
    }
    
    std::cout << "Using device: " << device_path << std::endl;
    
    // Create camera configuration
    ZeroCopyFrameProvider::CameraConfig config;
    config.device_path = device_path;
    config.width = TEST_WIDTH;
    config.height = TEST_HEIGHT;
    config.fps = TEST_FPS;
    config.pixel_format = TEST_FORMAT;
    config.zero_copy_enabled = TEST_ZERO_COPY;
    config.buffer_count = TEST_BUFFER_COUNT;
    
    // Set camera intrinsics (example values)
    config.fx = 500.0f;
    config.fy = 500.0f;
    config.cx = TEST_WIDTH / 2.0f;
    config.cy = TEST_HEIGHT / 2.0f;
    config.distortion_coeffs = {0.0f, 0.0f, 0.0f, 0.0f, 0.0f};
    
    // Set camera extrinsics (identity transform for single camera)
    config.T_ref_cam = cv::Mat::eye(4, 4, CV_32F);
    
    // Create frame provider with single camera
    std::vector<ZeroCopyFrameProvider::CameraConfig> configs = {config};
    ZeroCopyFrameProvider provider(configs);
    
    // Initialize provider
    std::cout << "Initializing frame provider..." << std::endl;
    if (!provider.Initialize()) {
        std::cerr << "Failed to initialize frame provider: " << provider.GetLastErrorMessage() << std::endl;
        return 1;
    }
    
    // Check if zero-copy is supported
    bool zero_copy_supported = provider.IsZeroCopySupported(0);
    std::cout << "Zero-copy supported: " << (zero_copy_supported ? "Yes" : "No") << std::endl;
    
    // Register frame callback
    provider.RegisterFrameCallback([](const ZeroCopyFrameProvider::FrameMetadata& metadata) {
        // This callback is called from the acquisition thread
        // Keep it lightweight to avoid blocking the thread
        std::cout << "Frame received in callback: " << metadata.frame_id << std::endl;
    });
    
    // Start acquisition
    std::cout << "Starting acquisition..." << std::endl;
    if (!provider.StartAcquisition()) {
        std::cerr << "Failed to start acquisition: " << provider.GetLastErrorMessage() << std::endl;
        return 1;
    }
    
    // Create window for display
    cv::namedWindow("ZeroCopyFrameProvider Test", cv::WINDOW_NORMAL);
    
    // Acquisition loop
    std::cout << "Running for " << TEST_DURATION_SEC << " seconds..." << std::endl;
    auto start_time = std::chrono::steady_clock::now();
    int frame_count = 0;
    
    while (true) {
        // Check if test duration has elapsed
        auto current_time = std::chrono::steady_clock::now();
        auto elapsed = std::chrono::duration_cast<std::chrono::seconds>(current_time - start_time).count();
        if (elapsed >= TEST_DURATION_SEC) {
            break;
        }
        
        // Get next frame
        ZeroCopyFrameProvider::FrameMetadata metadata;
        if (!provider.GetNextFrame(0, metadata, 100)) {
            std::cerr << "Failed to get frame: " << provider.GetLastErrorMessage() << std::endl;
            continue;
        }
        
        // Display frame info
        displayFrameInfo(metadata);
        
        // Get OpenCV Mat for frame
        cv::Mat frame = provider.GetMatForFrame(metadata);
        if (frame.empty()) {
            std::cerr << "Failed to get Mat for frame" << std::endl;
            provider.ReleaseFrame(metadata);
            continue;
        }
        
        // Display frame
        cv::imshow("ZeroCopyFrameProvider Test", frame);
        
        // Release frame
        provider.ReleaseFrame(metadata);
        
        // Process keyboard input
        int key = cv::waitKey(1);
        if (key == 27) { // ESC key
            break;
        }
        
        frame_count++;
    }
    
    // Calculate frame rate
    auto end_time = std::chrono::steady_clock::now();
    auto elapsed_sec = std::chrono::duration_cast<std::chrono::milliseconds>(end_time - start_time).count() / 1000.0;
    double fps = frame_count / elapsed_sec;
    
    std::cout << "Test completed" << std::endl;
    std::cout << "Frames captured: " << frame_count << std::endl;
    std::cout << "Elapsed time: " << elapsed_sec << " seconds" << std::endl;
    std::cout << "Average frame rate: " << fps << " fps" << std::endl;
    std::cout << "Provider reported frame rate: " << provider.GetCurrentFrameRate(0) << " fps" << std::endl;
    
    // Stop acquisition
    std::cout << "Stopping acquisition..." << std::endl;
    provider.StopAcquisition();
    
    // Clean up
    cv::destroyAllWindows();
    
    std::cout << "Test finished successfully" << std::endl;
    return 0;
}

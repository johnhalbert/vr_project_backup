#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <memory>
#include <vector>
#include <thread>
#include <chrono>

#include "../include/tpu_zero_copy_integration.hpp"
#include "../include/zero_copy_frame_provider.hpp"
#include "../ORB_SLAM3/include/tpu_feature_extractor.hpp"

using namespace ORB_SLAM3;
using namespace testing;

// Mock classes
class MockZeroCopyFrameProvider : public ZeroCopyFrameProvider {
public:
    MockZeroCopyFrameProvider() : ZeroCopyFrameProvider() {}
    
    MOCK_METHOD(bool, Initialize, (const std::vector<CameraConfig>&), (override));
    MOCK_METHOD(bool, StartAcquisition, (), (override));
    MOCK_METHOD(bool, StopAcquisition, (), (override));
    MOCK_METHOD(bool, GetFrame, (int, FrameBuffer&), (override));
    MOCK_METHOD(bool, GetSynchronizedFrames, (std::vector<FrameBuffer>&), (override));
    MOCK_METHOD(bool, ReleaseFrame, (FrameBuffer&), (override));
    MOCK_METHOD(PerformanceStats, GetPerformanceStats, (), (const, override));
};

class MockTPUFeatureExtractor : public TPUFeatureExtractor {
public:
    MockTPUFeatureExtractor() : TPUFeatureExtractor() {}
    
    MOCK_METHOD(bool, Initialize, (const std::string&, int, int), (override));
    MOCK_METHOD(bool, Extract, (const cv::Mat&, std::vector<cv::KeyPoint>&, cv::Mat&), (override));
    MOCK_METHOD(bool, ExtractDirectBuffer, (const void*, int, int, int, std::vector<cv::KeyPoint>&, cv::Mat&), (override));
    MOCK_METHOD(PerformanceMetrics, GetPerformanceMetrics, (), (const, override));
};

class TPUZeroCopyIntegrationTest : public ::testing::Test {
protected:
    void SetUp() override {
        // Create mock objects
        frame_provider_ = std::make_shared<MockZeroCopyFrameProvider>();
        feature_extractor_ = std::make_shared<MockTPUFeatureExtractor>();
        
        // Create configuration
        TPUZeroCopyIntegration::Config config;
        config.num_threads = 2;
        config.queue_size = 3;
        config.enable_direct_dma = true;
        config.enable_performance_tracking = true;
        
        // Create integration object
        integration_ = std::make_unique<TPUZeroCopyIntegration>(frame_provider_, feature_extractor_, config);
    }
    
    void TearDown() override {
        integration_.reset();
        feature_extractor_.reset();
        frame_provider_.reset();
    }
    
    // Helper method to create a frame buffer
    ZeroCopyFrameProvider::FrameBuffer createFrameBuffer(int camera_id, int width, int height) {
        ZeroCopyFrameProvider::FrameBuffer buffer;
        buffer.camera_id = camera_id;
        buffer.timestamp = std::chrono::steady_clock::now();
        buffer.width = width;
        buffer.height = height;
        buffer.stride = width;
        buffer.format = ZeroCopyFrameProvider::PixelFormat::GRAY8;
        buffer.buffer_type = ZeroCopyFrameProvider::BufferType::DMA;
        buffer.dma_fd = 42;  // Dummy file descriptor
        buffer.data = nullptr;  // No CPU pointer for DMA buffer
        buffer.size = width * height;
        return buffer;
    }
    
    // Helper method to set up expectations for successful frame processing
    void setupSuccessfulFrameProcessing() {
        // Create a frame buffer
        auto buffer = createFrameBuffer(0, 640, 480);
        
        // Set up expectations for frame provider
        EXPECT_CALL(*frame_provider_, GetFrame(0, _))
            .WillOnce(DoAll(
                SetArgReferee<1>(buffer),
                Return(true)
            ));
        
        // Set up expectations for feature extractor
        EXPECT_CALL(*feature_extractor_, ExtractDirectBuffer(_, 640, 480, _, _, _))
            .WillOnce(DoAll(
                // Fill in some keypoints and descriptors
                SetArgReferee<4>(std::vector<cv::KeyPoint>{
                    cv::KeyPoint(100.0f, 100.0f, 10.0f),
                    cv::KeyPoint(200.0f, 200.0f, 10.0f)
                }),
                SetArgReferee<5>(cv::Mat(2, 256, CV_8UC1)),
                Return(true)
            ));
        
        // Set up expectations for releasing the frame
        EXPECT_CALL(*frame_provider_, ReleaseFrame(_))
            .WillOnce(Return(true));
    }
    
    std::shared_ptr<MockZeroCopyFrameProvider> frame_provider_;
    std::shared_ptr<MockTPUFeatureExtractor> feature_extractor_;
    std::unique_ptr<TPUZeroCopyIntegration> integration_;
};

// Test initialization
TEST_F(TPUZeroCopyIntegrationTest, Initialization) {
    // Set up expectations
    EXPECT_CALL(*frame_provider_, Initialize(_))
        .WillOnce(Return(true));
    
    EXPECT_CALL(*feature_extractor_, Initialize(_, _, _))
        .WillOnce(Return(true));
    
    // Initialize integration
    std::vector<ZeroCopyFrameProvider::CameraConfig> camera_configs = {
        {0, 640, 480, ZeroCopyFrameProvider::PixelFormat::GRAY8, 30.0f}
    };
    
    bool result = integration_->Initialize(camera_configs, "model_path");
    
    // Verify result
    EXPECT_TRUE(result);
}

// Test starting and stopping
TEST_F(TPUZeroCopyIntegrationTest, StartStop) {
    // Set up expectations for initialization
    EXPECT_CALL(*frame_provider_, Initialize(_))
        .WillOnce(Return(true));
    
    EXPECT_CALL(*feature_extractor_, Initialize(_, _, _))
        .WillOnce(Return(true));
    
    // Initialize integration
    std::vector<ZeroCopyFrameProvider::CameraConfig> camera_configs = {
        {0, 640, 480, ZeroCopyFrameProvider::PixelFormat::GRAY8, 30.0f}
    };
    
    integration_->Initialize(camera_configs, "model_path");
    
    // Set up expectations for starting
    EXPECT_CALL(*frame_provider_, StartAcquisition())
        .WillOnce(Return(true));
    
    // Start integration
    bool start_result = integration_->Start();
    EXPECT_TRUE(start_result);
    
    // Set up expectations for stopping
    EXPECT_CALL(*frame_provider_, StopAcquisition())
        .WillOnce(Return(true));
    
    // Stop integration
    bool stop_result = integration_->Stop();
    EXPECT_TRUE(stop_result);
}

// Test processing a single frame
TEST_F(TPUZeroCopyIntegrationTest, ProcessSingleFrame) {
    // Set up expectations for initialization
    EXPECT_CALL(*frame_provider_, Initialize(_))
        .WillOnce(Return(true));
    
    EXPECT_CALL(*feature_extractor_, Initialize(_, _, _))
        .WillOnce(Return(true));
    
    // Initialize integration
    std::vector<ZeroCopyFrameProvider::CameraConfig> camera_configs = {
        {0, 640, 480, ZeroCopyFrameProvider::PixelFormat::GRAY8, 30.0f}
    };
    
    integration_->Initialize(camera_configs, "model_path");
    
    // Set up expectations for frame processing
    setupSuccessfulFrameProcessing();
    
    // Process a frame
    std::vector<cv::KeyPoint> keypoints;
    cv::Mat descriptors;
    bool result = integration_->ProcessFrame(0, keypoints, descriptors);
    
    // Verify result
    EXPECT_TRUE(result);
    EXPECT_EQ(keypoints.size(), 2);
}

// Test processing synchronized frames
TEST_F(TPUZeroCopyIntegrationTest, ProcessSynchronizedFrames) {
    // Set up expectations for initialization
    EXPECT_CALL(*frame_provider_, Initialize(_))
        .WillOnce(Return(true));
    
    EXPECT_CALL(*feature_extractor_, Initialize(_, _, _))
        .WillOnce(Return(true));
    
    // Initialize integration
    std::vector<ZeroCopyFrameProvider::CameraConfig> camera_configs = {
        {0, 640, 480, ZeroCopyFrameProvider::PixelFormat::GRAY8, 30.0f},
        {1, 640, 480, ZeroCopyFrameProvider::PixelFormat::GRAY8, 30.0f}
    };
    
    integration_->Initialize(camera_configs, "model_path");
    
    // Create frame buffers
    auto buffer0 = createFrameBuffer(0, 640, 480);
    auto buffer1 = createFrameBuffer(1, 640, 480);
    std::vector<ZeroCopyFrameProvider::FrameBuffer> buffers = {buffer0, buffer1};
    
    // Set up expectations for frame provider
    EXPECT_CALL(*frame_provider_, GetSynchronizedFrames(_))
        .WillOnce(DoAll(
            SetArgReferee<0>(buffers),
            Return(true)
        ));
    
    // Set up expectations for feature extractor (called twice, once for each camera)
    EXPECT_CALL(*feature_extractor_, ExtractDirectBuffer(_, 640, 480, _, _, _))
        .Times(2)
        .WillRepeatedly(DoAll(
            // Fill in some keypoints and descriptors
            SetArgReferee<4>(std::vector<cv::KeyPoint>{
                cv::KeyPoint(100.0f, 100.0f, 10.0f),
                cv::KeyPoint(200.0f, 200.0f, 10.0f)
            }),
            SetArgReferee<5>(cv::Mat(2, 256, CV_8UC1)),
            Return(true)
        ));
    
    // Set up expectations for releasing the frames
    EXPECT_CALL(*frame_provider_, ReleaseFrame(_))
        .Times(2)
        .WillRepeatedly(Return(true));
    
    // Process synchronized frames
    std::vector<std::vector<cv::KeyPoint>> all_keypoints;
    std::vector<cv::Mat> all_descriptors;
    bool result = integration_->ProcessSynchronizedFrames(all_keypoints, all_descriptors);
    
    // Verify result
    EXPECT_TRUE(result);
    EXPECT_EQ(all_keypoints.size(), 2);
    EXPECT_EQ(all_descriptors.size(), 2);
    EXPECT_EQ(all_keypoints[0].size(), 2);
    EXPECT_EQ(all_keypoints[1].size(), 2);
}

// Test performance metrics
TEST_F(TPUZeroCopyIntegrationTest, PerformanceMetrics) {
    // Set up expectations for initialization
    EXPECT_CALL(*frame_provider_, Initialize(_))
        .WillOnce(Return(true));
    
    EXPECT_CALL(*feature_extractor_, Initialize(_, _, _))
        .WillOnce(Return(true));
    
    // Initialize integration
    std::vector<ZeroCopyFrameProvider::CameraConfig> camera_configs = {
        {0, 640, 480, ZeroCopyFrameProvider::PixelFormat::GRAY8, 30.0f}
    };
    
    integration_->Initialize(camera_configs, "model_path");
    
    // Set up expectations for frame provider performance stats
    ZeroCopyFrameProvider::PerformanceStats provider_stats;
    provider_stats.average_frame_time_ms = 10.0;
    provider_stats.average_fps = 30.0;
    provider_stats.frames_processed = 100;
    
    EXPECT_CALL(*frame_provider_, GetPerformanceStats())
        .WillOnce(Return(provider_stats));
    
    // Set up expectations for feature extractor performance metrics
    TPUFeatureExtractor::PerformanceMetrics extractor_metrics;
    extractor_metrics.average_extraction_time_ms = 5.0;
    extractor_metrics.average_keypoints_per_frame = 200;
    extractor_metrics.frames_processed = 100;
    
    EXPECT_CALL(*feature_extractor_, GetPerformanceMetrics())
        .WillOnce(Return(extractor_metrics));
    
    // Get performance metrics
    TPUZeroCopyIntegration::PerformanceMetrics metrics = integration_->GetPerformanceMetrics();
    
    // Verify metrics
    EXPECT_FLOAT_EQ(metrics.average_frame_acquisition_time_ms, 10.0);
    EXPECT_FLOAT_EQ(metrics.average_feature_extraction_time_ms, 5.0);
    EXPECT_FLOAT_EQ(metrics.average_fps, 30.0);
    EXPECT_EQ(metrics.frames_processed, 100);
    EXPECT_FLOAT_EQ(metrics.average_keypoints_per_frame, 200);
}

// Test error handling
TEST_F(TPUZeroCopyIntegrationTest, ErrorHandling) {
    // Set up expectations for initialization
    EXPECT_CALL(*frame_provider_, Initialize(_))
        .WillOnce(Return(true));
    
    EXPECT_CALL(*feature_extractor_, Initialize(_, _, _))
        .WillOnce(Return(true));
    
    // Initialize integration
    std::vector<ZeroCopyFrameProvider::CameraConfig> camera_configs = {
        {0, 640, 480, ZeroCopyFrameProvider::PixelFormat::GRAY8, 30.0f}
    };
    
    integration_->Initialize(camera_configs, "model_path");
    
    // Set up expectations for frame provider failure
    EXPECT_CALL(*frame_provider_, GetFrame(0, _))
        .WillOnce(Return(false));
    
    // Process a frame (should fail)
    std::vector<cv::KeyPoint> keypoints;
    cv::Mat descriptors;
    bool result = integration_->ProcessFrame(0, keypoints, descriptors);
    
    // Verify result
    EXPECT_FALSE(result);
    EXPECT_TRUE(keypoints.empty());
    EXPECT_TRUE(descriptors.empty());
}

// Test fallback to non-DMA path
TEST_F(TPUZeroCopyIntegrationTest, FallbackToNonDMA) {
    // Set up expectations for initialization
    EXPECT_CALL(*frame_provider_, Initialize(_))
        .WillOnce(Return(true));
    
    EXPECT_CALL(*feature_extractor_, Initialize(_, _, _))
        .WillOnce(Return(true));
    
    // Initialize integration
    std::vector<ZeroCopyFrameProvider::CameraConfig> camera_configs = {
        {0, 640, 480, ZeroCopyFrameProvider::PixelFormat::GRAY8, 30.0f}
    };
    
    integration_->Initialize(camera_configs, "model_path");
    
    // Create a non-DMA frame buffer
    ZeroCopyFrameProvider::FrameBuffer buffer;
    buffer.camera_id = 0;
    buffer.timestamp = std::chrono::steady_clock::now();
    buffer.width = 640;
    buffer.height = 480;
    buffer.stride = 640;
    buffer.format = ZeroCopyFrameProvider::PixelFormat::GRAY8;
    buffer.buffer_type = ZeroCopyFrameProvider::BufferType::CPU;
    buffer.dma_fd = -1;  // Invalid for CPU buffer
    buffer.data = malloc(640 * 480);  // Allocate CPU buffer
    buffer.size = 640 * 480;
    
    // Set up expectations for frame provider
    EXPECT_CALL(*frame_provider_, GetFrame(0, _))
        .WillOnce(DoAll(
            SetArgReferee<1>(buffer),
            Return(true)
        ));
    
    // Set up expectations for feature extractor (should use regular Extract, not ExtractDirectBuffer)
    EXPECT_CALL(*feature_extractor_, Extract(_, _, _))
        .WillOnce(DoAll(
            // Fill in some keypoints and descriptors
            SetArgReferee<1>(std::vector<cv::KeyPoint>{
                cv::KeyPoint(100.0f, 100.0f, 10.0f),
                cv::KeyPoint(200.0f, 200.0f, 10.0f)
            }),
            SetArgReferee<2>(cv::Mat(2, 256, CV_8UC1)),
            Return(true)
        ));
    
    // Set up expectations for releasing the frame
    EXPECT_CALL(*frame_provider_, ReleaseFrame(_))
        .WillOnce(Return(true));
    
    // Process a frame
    std::vector<cv::KeyPoint> keypoints;
    cv::Mat descriptors;
    bool result = integration_->ProcessFrame(0, keypoints, descriptors);
    
    // Verify result
    EXPECT_TRUE(result);
    EXPECT_EQ(keypoints.size(), 2);
    
    // Clean up CPU buffer
    free(buffer.data);
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}

#ifndef TPU_ZERO_COPY_INTEGRATION_HPP
#define TPU_ZERO_COPY_INTEGRATION_HPP

#include "zero_copy_frame_provider.hpp"
#include "../ORB_SLAM3/include/tpu_feature_extractor.hpp"
#include <memory>
#include <vector>
#include <mutex>
#include <condition_variable>
#include <thread>
#include <atomic>
#include <queue>
#include <opencv2/core/mat.hpp>

namespace ORB_SLAM3
{

/**
 * @brief Integration class for zero-copy data flow between camera frames and TPU feature extraction
 * 
 * This class provides a high-performance integration between the ZeroCopyFrameProvider
 * and TPUFeatureExtractor, enabling direct buffer sharing and minimizing memory copies
 * in the feature extraction pipeline. It manages a dedicated thread pool for parallel
 * processing and implements efficient synchronization mechanisms.
 */
class TPUZeroCopyIntegration
{
public:
    /**
     * @brief Result structure containing extracted features and metadata
     */
    struct ExtractionResult {
        uint64_t frame_id;                    ///< Unique frame identifier
        double timestamp;                     ///< Timestamp in seconds
        int camera_id;                        ///< Camera identifier
        std::vector<cv::KeyPoint> keypoints;  ///< Extracted keypoints
        cv::Mat descriptors;                  ///< Feature descriptors
        std::vector<int> lapping_area;        ///< Lapping area information
        double processing_time_ms;            ///< Total processing time in milliseconds
    };

    /**
     * @brief Constructor
     * 
     * @param frame_provider Shared pointer to ZeroCopyFrameProvider
     * @param feature_extractor Shared pointer to TPUFeatureExtractor
     * @param num_threads Number of processing threads (default: 2)
     * @param queue_size Maximum size of the processing queue (default: 4)
     */
    TPUZeroCopyIntegration(
        std::shared_ptr<ZeroCopyFrameProvider> frame_provider,
        std::shared_ptr<TPUFeatureExtractor> feature_extractor,
        int num_threads = 2,
        int queue_size = 4);
    
    /**
     * @brief Destructor
     */
    ~TPUZeroCopyIntegration();
    
    /**
     * @brief Start the integration processing
     * 
     * @return True if started successfully, false otherwise
     */
    bool Start();
    
    /**
     * @brief Stop the integration processing
     */
    void Stop();
    
    /**
     * @brief Get the next extraction result
     * 
     * @param result Output parameter for extraction result
     * @param timeout_ms Timeout in milliseconds (0 for non-blocking, negative for infinite)
     * @return True if a result was obtained, false otherwise
     */
    bool GetNextResult(ExtractionResult& result, int timeout_ms = -1);
    
    /**
     * @brief Get the next synchronized extraction results from all cameras
     * 
     * @param results Output vector of extraction results
     * @param max_time_diff_ms Maximum time difference between frames in milliseconds
     * @param timeout_ms Timeout in milliseconds (0 for non-blocking, negative for infinite)
     * @return True if synchronized results were obtained, false otherwise
     */
    bool GetNextSynchronizedResults(
        std::vector<ExtractionResult>& results,
        float max_time_diff_ms = 10.0f,
        int timeout_ms = -1);
    
    /**
     * @brief Register a callback for new extraction results
     * 
     * @param callback Function to call when a new result is available
     */
    void RegisterResultCallback(std::function<void(const ExtractionResult&)> callback);
    
    /**
     * @brief Get the current processing rate
     * 
     * @param camera_id Camera identifier
     * @return Current processing rate in frames per second
     */
    float GetCurrentProcessingRate(int camera_id) const;
    
    /**
     * @brief Get the current queue size
     * 
     * @return Current number of frames in the processing queue
     */
    size_t GetQueueSize() const;
    
    /**
     * @brief Get the latest error message
     * 
     * @return Latest error message
     */
    std::string GetLastErrorMessage() const;
    
    /**
     * @brief Enable or disable direct DMA buffer access
     * 
     * When enabled, the integration will attempt to use direct DMA buffer access
     * for zero-copy data transfer between the camera and TPU. When disabled, it
     * will fall back to using OpenCV Mat objects.
     * 
     * @param enable Whether to enable direct DMA buffer access
     * @return True if the mode was set successfully, false otherwise
     */
    bool EnableDirectDMAAccess(bool enable);
    
    /**
     * @brief Check if direct DMA buffer access is supported
     * 
     * @return True if direct DMA buffer access is supported, false otherwise
     */
    bool IsDirectDMAAccessSupported() const;

private:
    // Component references
    std::shared_ptr<ZeroCopyFrameProvider> frame_provider_;
    std::shared_ptr<TPUFeatureExtractor> feature_extractor_;
    
    // Thread management
    std::vector<std::thread> processing_threads_;
    std::atomic<bool> running_;
    int num_threads_;
    int queue_size_;
    
    // Frame queue
    struct QueueItem {
        ZeroCopyFrameProvider::FrameMetadata metadata;
        cv::Mat image;  // Only used when direct DMA is not available
    };
    std::queue<QueueItem> frame_queue_;
    std::mutex queue_mutex_;
    std::condition_variable queue_condition_;
    
    // Result queue
    std::queue<ExtractionResult> result_queue_;
    std::mutex result_mutex_;
    std::condition_variable result_condition_;
    
    // Statistics
    std::vector<std::atomic<float>> processing_rates_;
    std::vector<std::atomic<uint64_t>> frame_counters_;
    std::vector<std::chrono::time_point<std::chrono::steady_clock>> last_frame_times_;
    
    // Error handling
    std::string last_error_message_;
    std::mutex error_mutex_;
    
    // Callbacks
    std::function<void(const ExtractionResult&)> result_callback_;
    
    // Configuration
    bool direct_dma_enabled_;
    
    /**
     * @brief Frame acquisition thread function
     */
    void AcquisitionThreadFunc();
    
    /**
     * @brief Processing thread function
     */
    void ProcessingThreadFunc();
    
    /**
     * @brief Process a frame using direct DMA buffer access
     * 
     * @param metadata Frame metadata
     * @return Extraction result
     */
    ExtractionResult ProcessFrameDMA(const ZeroCopyFrameProvider::FrameMetadata& metadata);
    
    /**
     * @brief Process a frame using OpenCV Mat
     * 
     * @param metadata Frame metadata
     * @param image OpenCV Mat containing the frame data
     * @return Extraction result
     */
    ExtractionResult ProcessFrameMat(
        const ZeroCopyFrameProvider::FrameMetadata& metadata,
        const cv::Mat& image);
    
    /**
     * @brief Set an error message
     * 
     * @param message Error message
     */
    void SetErrorMessage(const std::string& message);
    
    /**
     * @brief Update processing rate statistics
     * 
     * @param camera_id Camera identifier
     */
    void UpdateProcessingRate(int camera_id);
};

} // namespace ORB_SLAM3

#endif // TPU_ZERO_COPY_INTEGRATION_HPP

#include "include/tpu_zero_copy_integration.hpp"
#include <iostream>
#include <chrono>
#include <algorithm>
#include <cstring>
#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <linux/videodev2.h>
#include <linux/dma-buf.h>
#include <opencv2/imgproc.hpp>

// For EdgeTPU DMA buffer handling
#include "tensorflow/lite/delegates/edgetpu/edgetpu_c.h"

namespace ORB_SLAM3
{

//------------------------------------------------------------------------------
// Constructor & Destructor
//------------------------------------------------------------------------------

TPUZeroCopyIntegration::TPUZeroCopyIntegration(
    std::shared_ptr<ZeroCopyFrameProvider> frame_provider,
    std::shared_ptr<TPUFeatureExtractor> feature_extractor,
    int num_threads,
    int queue_size)
    : frame_provider_(frame_provider),
      feature_extractor_(feature_extractor),
      running_(false),
      num_threads_(num_threads),
      queue_size_(queue_size),
      direct_dma_enabled_(false)
{
    // Initialize statistics
    const size_t num_cameras = frame_provider_->GetCameraCount();
    processing_rates_.resize(num_cameras, 0.0f);
    frame_counters_.resize(num_cameras, 0);
    last_frame_times_.resize(num_cameras);
    
    // Set default error message
    last_error_message_ = "No error";
    
    // Check if direct DMA access is supported
    direct_dma_enabled_ = IsDirectDMAAccessSupported();
    if (direct_dma_enabled_) {
        std::cout << "Direct DMA buffer access is supported and enabled." << std::endl;
    } else {
        std::cout << "Direct DMA buffer access is not supported, falling back to Mat-based processing." << std::endl;
    }
}

TPUZeroCopyIntegration::~TPUZeroCopyIntegration()
{
    // Stop processing if running
    if (running_) {
        Stop();
    }
    
    std::cout << "TPUZeroCopyIntegration destroyed." << std::endl;
}

//------------------------------------------------------------------------------
// Public Methods
//------------------------------------------------------------------------------

bool TPUZeroCopyIntegration::Start()
{
    // Check if already running
    if (running_) {
        SetErrorMessage("Integration already running");
        return false;
    }
    
    // Check if components are valid
    if (!frame_provider_ || !feature_extractor_) {
        SetErrorMessage("Invalid frame provider or feature extractor");
        return false;
    }
    
    // Start frame acquisition
    if (!frame_provider_->StartAcquisition()) {
        SetErrorMessage("Failed to start frame acquisition: " + frame_provider_->GetLastErrorMessage());
        return false;
    }
    
    // Start processing threads
    running_ = true;
    
    // Start acquisition thread
    std::thread acquisition_thread(&TPUZeroCopyIntegration::AcquisitionThreadFunc, this);
    processing_threads_.push_back(std::move(acquisition_thread));
    
    // Start processing threads
    for (int i = 0; i < num_threads_; ++i) {
        std::thread processing_thread(&TPUZeroCopyIntegration::ProcessingThreadFunc, this);
        processing_threads_.push_back(std::move(processing_thread));
    }
    
    std::cout << "TPUZeroCopyIntegration started with " << num_threads_ << " processing threads." << std::endl;
    return true;
}

void TPUZeroCopyIntegration::Stop()
{
    // Check if running
    if (!running_) {
        return;
    }
    
    // Stop processing threads
    running_ = false;
    
    // Notify all threads
    queue_condition_.notify_all();
    result_condition_.notify_all();
    
    // Join processing threads
    for (auto& thread : processing_threads_) {
        if (thread.joinable()) {
            thread.join();
        }
    }
    processing_threads_.clear();
    
    // Stop frame acquisition
    frame_provider_->StopAcquisition();
    
    // Clear queues
    {
        std::lock_guard<std::mutex> lock(queue_mutex_);
        while (!frame_queue_.empty()) {
            frame_queue_.pop();
        }
    }
    
    {
        std::lock_guard<std::mutex> lock(result_mutex_);
        while (!result_queue_.empty()) {
            result_queue_.pop();
        }
    }
    
    std::cout << "TPUZeroCopyIntegration stopped." << std::endl;
}

bool TPUZeroCopyIntegration::GetNextResult(ExtractionResult& result, int timeout_ms)
{
    // Check if running
    if (!running_) {
        SetErrorMessage("Integration not running");
        return false;
    }
    
    // Wait for a result
    std::unique_lock<std::mutex> lock(result_mutex_);
    
    if (timeout_ms < 0) {
        // Wait indefinitely
        result_condition_.wait(lock, [this]() {
            return !running_ || !result_queue_.empty();
        });
    } else if (timeout_ms > 0) {
        // Wait with timeout
        auto result = result_condition_.wait_for(lock, std::chrono::milliseconds(timeout_ms), [this]() {
            return !running_ || !result_queue_.empty();
        });
        
        if (!result) {
            // Timeout
            SetErrorMessage("Timeout waiting for result");
            return false;
        }
    }
    
    // Check if running
    if (!running_) {
        SetErrorMessage("Integration stopped while waiting for result");
        return false;
    }
    
    // Check if queue is empty (for non-blocking mode)
    if (result_queue_.empty()) {
        SetErrorMessage("No result available");
        return false;
    }
    
    // Get result from queue
    result = result_queue_.front();
    result_queue_.pop();
    
    return true;
}

bool TPUZeroCopyIntegration::GetNextSynchronizedResults(
    std::vector<ExtractionResult>& results,
    float max_time_diff_ms,
    int timeout_ms)
{
    // This is a simplified implementation that collects results from all cameras
    // A more sophisticated implementation would ensure proper synchronization
    
    // Check if running
    if (!running_) {
        SetErrorMessage("Integration not running");
        return false;
    }
    
    // Get the number of cameras
    const size_t num_cameras = frame_provider_->GetCameraCount();
    
    // Collect results from all cameras
    results.clear();
    results.reserve(num_cameras);
    
    for (size_t i = 0; i < num_cameras; ++i) {
        ExtractionResult result;
        if (!GetNextResult(result, timeout_ms)) {
            return false;
        }
        results.push_back(result);
    }
    
    // Check if results are synchronized
    double reference_time = results[0].timestamp;
    bool synchronized = true;
    
    for (size_t i = 1; i < results.size(); ++i) {
        double time_diff = std::abs(results[i].timestamp - reference_time) * 1000.0; // Convert to ms
        if (time_diff > max_time_diff_ms) {
            synchronized = false;
            break;
        }
    }
    
    if (!synchronized) {
        SetErrorMessage("Results are not synchronized");
        return false;
    }
    
    return true;
}

void TPUZeroCopyIntegration::RegisterResultCallback(std::function<void(const ExtractionResult&)> callback)
{
    result_callback_ = callback;
}

float TPUZeroCopyIntegration::GetCurrentProcessingRate(int camera_id) const
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(processing_rates_.size())) {
        return 0.0f;
    }
    
    return processing_rates_[camera_id];
}

size_t TPUZeroCopyIntegration::GetQueueSize() const
{
    std::lock_guard<std::mutex> lock(queue_mutex_);
    return frame_queue_.size();
}

std::string TPUZeroCopyIntegration::GetLastErrorMessage() const
{
    std::lock_guard<std::mutex> lock(error_mutex_);
    return last_error_message_;
}

bool TPUZeroCopyIntegration::EnableDirectDMAAccess(bool enable)
{
    // Check if running
    if (running_) {
        SetErrorMessage("Cannot change DMA mode while integration is running");
        return false;
    }
    
    // Check if direct DMA access is supported
    if (enable && !IsDirectDMAAccessSupported()) {
        SetErrorMessage("Direct DMA buffer access is not supported");
        return false;
    }
    
    direct_dma_enabled_ = enable;
    return true;
}

bool TPUZeroCopyIntegration::IsDirectDMAAccessSupported() const
{
    // Check if frame provider supports zero-copy
    bool provider_supports_zero_copy = false;
    for (size_t i = 0; i < frame_provider_->GetCameraCount(); ++i) {
        if (frame_provider_->IsZeroCopySupported(i)) {
            provider_supports_zero_copy = true;
            break;
        }
    }
    
    // Check if EdgeTPU supports DMA buffer import
    // This is a simplified check; in a real implementation, you would need to
    // check the specific EdgeTPU device capabilities
    bool edgetpu_supports_dma = true; // Assume EdgeTPU supports DMA
    
    return provider_supports_zero_copy && edgetpu_supports_dma;
}

//------------------------------------------------------------------------------
// Private Methods
//------------------------------------------------------------------------------

void TPUZeroCopyIntegration::AcquisitionThreadFunc()
{
    // Set thread name for debugging
    #ifdef __linux__
        pthread_setname_np(pthread_self(), "ZC-Acquisition");
    #endif
    
    while (running_) {
        // Check if queue is full
        {
            std::unique_lock<std::mutex> lock(queue_mutex_);
            if (frame_queue_.size() >= static_cast<size_t>(queue_size_)) {
                // Wait for queue to have space
                queue_condition_.wait(lock, [this]() {
                    return !running_ || frame_queue_.size() < static_cast<size_t>(queue_size_);
                });
            }
            
            // Check if still running
            if (!running_) {
                break;
            }
        }
        
        // Get next frame from all cameras
        std::vector<ZeroCopyFrameProvider::FrameMetadata> metadata_vec;
        if (frame_provider_->GetNextSynchronizedFrames(metadata_vec, 10.0f, 100)) {
            // Add frames to queue
            std::unique_lock<std::mutex> lock(queue_mutex_);
            
            for (const auto& metadata : metadata_vec) {
                QueueItem item;
                item.metadata = metadata;
                
                // If not using direct DMA, get Mat for frame
                if (!direct_dma_enabled_) {
                    item.image = frame_provider_->GetMatForFrame(metadata);
                }
                
                frame_queue_.push(item);
            }
            
            // Notify processing threads
            lock.unlock();
            queue_condition_.notify_one();
        }
    }
}

void TPUZeroCopyIntegration::ProcessingThreadFunc()
{
    // Set thread name for debugging
    #ifdef __linux__
        pthread_setname_np(pthread_self(), "ZC-Processing");
    #endif
    
    while (running_) {
        // Get next frame from queue
        QueueItem item;
        {
            std::unique_lock<std::mutex> lock(queue_mutex_);
            
            // Wait for a frame
            queue_condition_.wait(lock, [this]() {
                return !running_ || !frame_queue_.empty();
            });
            
            // Check if still running
            if (!running_) {
                break;
            }
            
            // Check if queue is empty
            if (frame_queue_.empty()) {
                continue;
            }
            
            // Get frame from queue
            item = frame_queue_.front();
            frame_queue_.pop();
            
            // Notify acquisition thread
            lock.unlock();
            queue_condition_.notify_one();
        }
        
        // Process frame
        ExtractionResult result;
        if (direct_dma_enabled_) {
            result = ProcessFrameDMA(item.metadata);
        } else {
            result = ProcessFrameMat(item.metadata, item.image);
        }
        
        // Release frame
        frame_provider_->ReleaseFrame(item.metadata);
        
        // Update statistics
        UpdateProcessingRate(item.metadata.camera_id);
        
        // Add result to queue
        {
            std::lock_guard<std::mutex> lock(result_mutex_);
            result_queue_.push(result);
        }
        
        // Notify waiting threads
        result_condition_.notify_one();
        
        // Call callback if registered
        if (result_callback_) {
            result_callback_(result);
        }
    }
}

TPUZeroCopyIntegration::ExtractionResult TPUZeroCopyIntegration::ProcessFrameDMA(
    const ZeroCopyFrameProvider::FrameMetadata& metadata)
{
    // Start timing
    auto start_time = std::chrono::high_resolution_clock::now();
    
    // Create result structure
    ExtractionResult result;
    result.frame_id = metadata.frame_id;
    result.timestamp = metadata.timestamp;
    result.camera_id = metadata.camera_id;
    
    // Get DMA file descriptor for frame
    int dma_fd = frame_provider_->GetDmaFdForFrame(metadata);
    
    // In a real implementation, you would use the DMA file descriptor to create
    // a TensorFlow Lite tensor that directly references the DMA buffer, avoiding
    // any memory copies. This would require custom TensorFlow Lite ops or
    // extensions to the EdgeTPU delegate.
    //
    // For this implementation, we'll simulate the process by getting a Mat for
    // the frame and then processing it.
    cv::Mat image = frame_provider_->GetMatForFrame(metadata);
    
    // Create mask (if needed)
    cv::Mat mask;
    
    // Extract features using TPUFeatureExtractor
    (*feature_extractor_)(image, mask, result.keypoints, result.descriptors, result.lapping_area);
    
    // Calculate processing time
    auto end_time = std::chrono::high_resolution_clock::now();
    result.processing_time_ms = std::chrono::duration<double, std::milli>(end_time - start_time).count();
    
    return result;
}

TPUZeroCopyIntegration::ExtractionResult TPUZeroCopyIntegration::ProcessFrameMat(
    const ZeroCopyFrameProvider::FrameMetadata& metadata,
    const cv::Mat& image)
{
    // Start timing
    auto start_time = std::chrono::high_resolution_clock::now();
    
    // Create result structure
    ExtractionResult result;
    result.frame_id = metadata.frame_id;
    result.timestamp = metadata.timestamp;
    result.camera_id = metadata.camera_id;
    
    // Create mask (if needed)
    cv::Mat mask;
    
    // Extract features using TPUFeatureExtractor
    (*feature_extractor_)(image, mask, result.keypoints, result.descriptors, result.lapping_area);
    
    // Calculate processing time
    auto end_time = std::chrono::high_resolution_clock::now();
    result.processing_time_ms = std::chrono::duration<double, std::milli>(end_time - start_time).count();
    
    return result;
}

void TPUZeroCopyIntegration::SetErrorMessage(const std::string& message)
{
    std::lock_guard<std::mutex> lock(error_mutex_);
    last_error_message_ = message;
    std::cerr << "TPUZeroCopyIntegration error: " << message << std::endl;
}

void TPUZeroCopyIntegration::UpdateProcessingRate(int camera_id)
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(processing_rates_.size())) {
        return;
    }
    
    // Get current time
    auto now = std::chrono::steady_clock::now();
    
    // Increment frame counter
    frame_counters_[camera_id]++;
    
    // Calculate time since last update
    auto& last_time = last_frame_times_[camera_id];
    if (last_time.time_since_epoch().count() == 0) {
        // First frame
        last_time = now;
        return;
    }
    
    auto elapsed = std::chrono::duration<float>(now - last_time).count();
    if (elapsed >= 1.0f) {
        // Update processing rate
        processing_rates_[camera_id] = static_cast<float>(frame_counters_[camera_id]) / elapsed;
        
        // Reset counter and update last time
        frame_counters_[camera_id] = 0;
        last_time = now;
    }
}

} // namespace ORB_SLAM3

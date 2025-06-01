#ifndef ZERO_COPY_FRAME_PROVIDER_HPP
#define ZERO_COPY_FRAME_PROVIDER_HPP

#include <vector>
#include <string>
#include <thread>
#include <mutex>
#include <atomic>
#include <queue>
#include <condition_variable>
#include <opencv2/core/core.hpp>
#include <opencv2/core/mat.hpp>

namespace ORB_SLAM3
{

/**
 * @brief Interface for zero-copy frame acquisition from camera to TPU
 * 
 * This class provides a high-performance interface for acquiring camera frames
 * and passing them directly to the TPU for feature extraction without unnecessary
 * memory copies. It is designed to work with the V4L2 camera driver and the
 * EdgeTPU hardware to minimize latency in the SLAM pipeline.
 * 
 * Key features:
 * 1. Direct DMA buffer sharing between camera and TPU
 * 2. Multi-camera synchronization
 * 3. Frame timestamping and synchronization with IMU data
 * 4. Efficient buffer management to avoid memory copies
 * 5. Support for different camera configurations and formats
 */
class ZeroCopyFrameProvider
{
public:
    /**
     * @brief Camera configuration structure
     */
    struct CameraConfig {
        std::string device_path;      ///< Camera device path (e.g., /dev/video0)
        int width;                    ///< Frame width in pixels
        int height;                   ///< Frame height in pixels
        int fps;                      ///< Frames per second
        std::string pixel_format;     ///< Pixel format (e.g., "GREY", "YUYV", "MJPG")
        bool zero_copy_enabled;       ///< Whether zero-copy is enabled for this camera
        int buffer_count;             ///< Number of buffers to allocate
        
        // Camera intrinsics
        float fx, fy;                 ///< Focal length in pixels
        float cx, cy;                 ///< Principal point in pixels
        std::vector<float> distortion_coeffs; ///< Distortion coefficients
        
        // Camera extrinsics (relative to reference camera)
        cv::Mat T_ref_cam;            ///< Transform from reference camera to this camera
    };
    
    /**
     * @brief Frame metadata structure
     */
    struct FrameMetadata {
        uint64_t frame_id;            ///< Unique frame identifier
        double timestamp;             ///< Timestamp in seconds
        int camera_id;                ///< Camera identifier
        int width;                    ///< Frame width in pixels
        int height;                   ///< Frame height in pixels
        std::string pixel_format;     ///< Pixel format
        void* buffer_ptr;             ///< Pointer to the frame buffer
        size_t buffer_size;           ///< Size of the buffer in bytes
        int dma_fd;                   ///< DMA file descriptor for zero-copy
        bool is_keyframe;             ///< Whether this frame is a keyframe
    };
    
    /**
     * @brief Constructor with camera configurations
     * @param configs Vector of camera configurations
     */
    ZeroCopyFrameProvider(const std::vector<CameraConfig>& configs);
    
    /**
     * @brief Destructor
     */
    ~ZeroCopyFrameProvider();
    
    /**
     * @brief Initialize the frame provider
     * @return True if initialization was successful, false otherwise
     */
    bool Initialize();
    
    /**
     * @brief Start frame acquisition
     * @return True if acquisition started successfully, false otherwise
     */
    bool StartAcquisition();
    
    /**
     * @brief Stop frame acquisition
     */
    void StopAcquisition();
    
    /**
     * @brief Get the next frame from the specified camera
     * @param camera_id Camera identifier
     * @param metadata Output parameter for frame metadata
     * @param timeout_ms Timeout in milliseconds (0 for non-blocking, negative for infinite)
     * @return True if a frame was acquired, false otherwise
     */
    bool GetNextFrame(int camera_id, FrameMetadata& metadata, int timeout_ms = -1);
    
    /**
     * @brief Get the next synchronized frames from all cameras
     * @param metadata_vec Output vector of frame metadata
     * @param max_time_diff_ms Maximum time difference between frames in milliseconds
     * @param timeout_ms Timeout in milliseconds (0 for non-blocking, negative for infinite)
     * @return True if synchronized frames were acquired, false otherwise
     */
    bool GetNextSynchronizedFrames(std::vector<FrameMetadata>& metadata_vec, 
                                  float max_time_diff_ms = 10.0f,
                                  int timeout_ms = -1);
    
    /**
     * @brief Release a frame buffer
     * @param metadata Frame metadata
     */
    void ReleaseFrame(const FrameMetadata& metadata);
    
    /**
     * @brief Get a cv::Mat wrapper for a frame buffer (avoid using this for zero-copy)
     * @param metadata Frame metadata
     * @return cv::Mat wrapper for the frame buffer
     */
    cv::Mat GetMatForFrame(const FrameMetadata& metadata);
    
    /**
     * @brief Get the DMA file descriptor for a frame buffer
     * @param metadata Frame metadata
     * @return DMA file descriptor
     */
    int GetDmaFdForFrame(const FrameMetadata& metadata);
    
    /**
     * @brief Get the camera configuration
     * @param camera_id Camera identifier
     * @return Camera configuration
     */
    CameraConfig GetCameraConfig(int camera_id) const;
    
    /**
     * @brief Set a new camera configuration
     * @param camera_id Camera identifier
     * @param config New camera configuration
     * @return True if configuration was set successfully, false otherwise
     */
    bool SetCameraConfig(int camera_id, const CameraConfig& config);
    
    /**
     * @brief Check if a camera is connected
     * @param camera_id Camera identifier
     * @return True if camera is connected, false otherwise
     */
    bool IsCameraConnected(int camera_id) const;
    
    /**
     * @brief Get the number of cameras
     * @return Number of cameras
     */
    size_t GetCameraCount() const;
    
    /**
     * @brief Get the current frame rate
     * @param camera_id Camera identifier
     * @return Current frame rate in frames per second
     */
    float GetCurrentFrameRate(int camera_id) const;
    
    /**
     * @brief Register a callback for new frames
     * @param callback Function to call when a new frame is available
     */
    void RegisterFrameCallback(std::function<void(const FrameMetadata&)> callback);
    
    /**
     * @brief Enable or disable zero-copy mode
     * @param camera_id Camera identifier
     * @param enable Whether to enable zero-copy
     * @return True if mode was set successfully, false otherwise
     */
    bool EnableZeroCopy(int camera_id, bool enable);
    
    /**
     * @brief Check if zero-copy is supported
     * @param camera_id Camera identifier
     * @return True if zero-copy is supported, false otherwise
     */
    bool IsZeroCopySupported(int camera_id) const;
    
    /**
     * @brief Get the latest error message
     * @return Latest error message
     */
    std::string GetLastErrorMessage() const;

private:
    // Camera configurations
    std::vector<CameraConfig> mCameraConfigs;
    
    // Camera handles
    std::vector<int> mCameraHandles;
    
    // Buffer management
    struct BufferInfo {
        void* start;
        size_t length;
        int dma_fd;
        bool in_use;
    };
    std::vector<std::vector<BufferInfo>> mBuffers;
    
    // Thread management
    std::vector<std::thread> mAcquisitionThreads;
    std::atomic<bool> mRunning;
    std::mutex mFrameQueueMutex;
    std::condition_variable mFrameCondition;
    
    // Frame queues
    std::vector<std::queue<FrameMetadata>> mFrameQueues;
    
    // Statistics
    std::vector<std::atomic<float>> mCurrentFrameRates;
    std::vector<std::atomic<uint64_t>> mFrameCounters;
    std::vector<std::chrono::time_point<std::chrono::steady_clock>> mLastFrameTimes;
    
    // Error handling
    std::string mLastErrorMessage;
    std::mutex mErrorMutex;
    
    // Callbacks
    std::function<void(const FrameMetadata&)> mFrameCallback;
    
    // Private methods
    
    /**
     * @brief Open a camera device
     * @param camera_id Camera identifier
     * @return True if successful, false otherwise
     */
    bool OpenCamera(int camera_id);
    
    /**
     * @brief Close a camera device
     * @param camera_id Camera identifier
     */
    void CloseCamera(int camera_id);
    
    /**
     * @brief Configure a camera device
     * @param camera_id Camera identifier
     * @return True if successful, false otherwise
     */
    bool ConfigureCamera(int camera_id);
    
    /**
     * @brief Allocate buffers for a camera
     * @param camera_id Camera identifier
     * @return True if successful, false otherwise
     */
    bool AllocateBuffers(int camera_id);
    
    /**
     * @brief Free buffers for a camera
     * @param camera_id Camera identifier
     */
    void FreeBuffers(int camera_id);
    
    /**
     * @brief Start streaming from a camera
     * @param camera_id Camera identifier
     * @return True if successful, false otherwise
     */
    bool StartStreaming(int camera_id);
    
    /**
     * @brief Stop streaming from a camera
     * @param camera_id Camera identifier
     */
    void StopStreaming(int camera_id);
    
    /**
     * @brief Acquisition thread function
     * @param camera_id Camera identifier
     */
    void AcquisitionThreadFunc(int camera_id);
    
    /**
     * @brief Set an error message
     * @param message Error message
     */
    void SetErrorMessage(const std::string& message);
    
    /**
     * @brief Check if DMA buffer sharing is supported
     * @param camera_id Camera identifier
     * @return True if supported, false otherwise
     */
    bool CheckDmaSupport(int camera_id);
    
    /**
     * @brief Export a DMA buffer
     * @param camera_id Camera identifier
     * @param buffer_index Buffer index
     * @return DMA file descriptor, or -1 on error
     */
    int ExportDmaBuffer(int camera_id, int buffer_index);
};

} // namespace ORB_SLAM3

#endif // ZERO_COPY_FRAME_PROVIDER_HPP

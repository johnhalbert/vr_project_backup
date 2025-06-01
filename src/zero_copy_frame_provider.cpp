#include "include/zero_copy_frame_provider.hpp"
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

namespace ORB_SLAM3
{

//------------------------------------------------------------------------------
// Constructor & Destructor
//------------------------------------------------------------------------------

ZeroCopyFrameProvider::ZeroCopyFrameProvider(const std::vector<CameraConfig>& configs)
    : mCameraConfigs(configs),
      mRunning(false)
{
    // Initialize camera handles, buffers, and statistics
    const size_t num_cameras = configs.size();
    mCameraHandles.resize(num_cameras, -1);
    mBuffers.resize(num_cameras);
    mFrameQueues.resize(num_cameras);
    mCurrentFrameRates.resize(num_cameras, 0.0f);
    mFrameCounters.resize(num_cameras, 0);
    mLastFrameTimes.resize(num_cameras);
    
    // Set default error message
    mLastErrorMessage = "No error";
}

ZeroCopyFrameProvider::~ZeroCopyFrameProvider()
{
    // Stop acquisition if running
    if (mRunning) {
        StopAcquisition();
    }
    
    // Close all cameras
    for (size_t i = 0; i < mCameraHandles.size(); ++i) {
        if (mCameraHandles[i] >= 0) {
            CloseCamera(i);
        }
    }
}

//------------------------------------------------------------------------------
// Public Methods
//------------------------------------------------------------------------------

bool ZeroCopyFrameProvider::Initialize()
{
    // Check if already initialized
    if (std::any_of(mCameraHandles.begin(), mCameraHandles.end(), 
                    [](int handle) { return handle >= 0; })) {
        SetErrorMessage("Already initialized");
        return false;
    }
    
    // Initialize each camera
    bool all_success = true;
    for (size_t i = 0; i < mCameraConfigs.size(); ++i) {
        if (!OpenCamera(i)) {
            all_success = false;
            break;
        }
        
        if (!ConfigureCamera(i)) {
            CloseCamera(i);
            all_success = false;
            break;
        }
        
        if (!AllocateBuffers(i)) {
            CloseCamera(i);
            all_success = false;
            break;
        }
    }
    
    // If any camera failed to initialize, close all cameras
    if (!all_success) {
        for (size_t i = 0; i < mCameraHandles.size(); ++i) {
            if (mCameraHandles[i] >= 0) {
                FreeBuffers(i);
                CloseCamera(i);
            }
        }
        return false;
    }
    
    return true;
}

bool ZeroCopyFrameProvider::StartAcquisition()
{
    // Check if already running
    if (mRunning) {
        SetErrorMessage("Acquisition already running");
        return false;
    }
    
    // Check if initialized
    if (std::none_of(mCameraHandles.begin(), mCameraHandles.end(), 
                     [](int handle) { return handle >= 0; })) {
        SetErrorMessage("Not initialized");
        return false;
    }
    
    // Start streaming on all cameras
    bool all_success = true;
    for (size_t i = 0; i < mCameraHandles.size(); ++i) {
        if (mCameraHandles[i] >= 0) {
            if (!StartStreaming(i)) {
                all_success = false;
                break;
            }
        }
    }
    
    // If any camera failed to start streaming, stop all cameras
    if (!all_success) {
        for (size_t i = 0; i < mCameraHandles.size(); ++i) {
            if (mCameraHandles[i] >= 0) {
                StopStreaming(i);
            }
        }
        return false;
    }
    
    // Start acquisition threads
    mRunning = true;
    for (size_t i = 0; i < mCameraHandles.size(); ++i) {
        if (mCameraHandles[i] >= 0) {
            mAcquisitionThreads.emplace_back(&ZeroCopyFrameProvider::AcquisitionThreadFunc, this, i);
        }
    }
    
    return true;
}

void ZeroCopyFrameProvider::StopAcquisition()
{
    // Check if running
    if (!mRunning) {
        return;
    }
    
    // Stop acquisition threads
    mRunning = false;
    mFrameCondition.notify_all();
    
    // Join acquisition threads
    for (auto& thread : mAcquisitionThreads) {
        if (thread.joinable()) {
            thread.join();
        }
    }
    mAcquisitionThreads.clear();
    
    // Stop streaming on all cameras
    for (size_t i = 0; i < mCameraHandles.size(); ++i) {
        if (mCameraHandles[i] >= 0) {
            StopStreaming(i);
        }
    }
    
    // Clear frame queues
    std::lock_guard<std::mutex> lock(mFrameQueueMutex);
    for (auto& queue : mFrameQueues) {
        while (!queue.empty()) {
            queue.pop();
        }
    }
}

bool ZeroCopyFrameProvider::GetNextFrame(int camera_id, FrameMetadata& metadata, int timeout_ms)
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(mCameraHandles.size())) {
        SetErrorMessage("Invalid camera ID");
        return false;
    }
    
    // Check if camera is initialized
    if (mCameraHandles[camera_id] < 0) {
        SetErrorMessage("Camera not initialized");
        return false;
    }
    
    // Check if running
    if (!mRunning) {
        SetErrorMessage("Acquisition not running");
        return false;
    }
    
    // Wait for a frame
    std::unique_lock<std::mutex> lock(mFrameQueueMutex);
    
    if (timeout_ms < 0) {
        // Wait indefinitely
        mFrameCondition.wait(lock, [this, camera_id]() {
            return !mRunning || !mFrameQueues[camera_id].empty();
        });
    } else if (timeout_ms > 0) {
        // Wait with timeout
        auto result = mFrameCondition.wait_for(lock, std::chrono::milliseconds(timeout_ms), [this, camera_id]() {
            return !mRunning || !mFrameQueues[camera_id].empty();
        });
        
        if (!result) {
            // Timeout
            SetErrorMessage("Timeout waiting for frame");
            return false;
        }
    }
    
    // Check if running
    if (!mRunning) {
        SetErrorMessage("Acquisition stopped while waiting for frame");
        return false;
    }
    
    // Check if queue is empty (for non-blocking mode)
    if (mFrameQueues[camera_id].empty()) {
        SetErrorMessage("No frame available");
        return false;
    }
    
    // Get frame from queue
    metadata = mFrameQueues[camera_id].front();
    mFrameQueues[camera_id].pop();
    
    return true;
}

bool ZeroCopyFrameProvider::GetNextSynchronizedFrames(std::vector<FrameMetadata>& metadata_vec, 
                                                     float max_time_diff_ms,
                                                     int timeout_ms)
{
    // Check if running
    if (!mRunning) {
        SetErrorMessage("Acquisition not running");
        return false;
    }
    
    // Wait for frames from all cameras
    std::unique_lock<std::mutex> lock(mFrameQueueMutex);
    
    auto wait_predicate = [this]() {
        return !mRunning || std::all_of(mFrameQueues.begin(), mFrameQueues.end(),
                                       [](const std::queue<FrameMetadata>& queue) {
                                           return !queue.empty();
                                       });
    };
    
    if (timeout_ms < 0) {
        // Wait indefinitely
        mFrameCondition.wait(lock, wait_predicate);
    } else if (timeout_ms > 0) {
        // Wait with timeout
        auto result = mFrameCondition.wait_for(lock, std::chrono::milliseconds(timeout_ms), wait_predicate);
        
        if (!result) {
            // Timeout
            SetErrorMessage("Timeout waiting for synchronized frames");
            return false;
        }
    }
    
    // Check if running
    if (!mRunning) {
        SetErrorMessage("Acquisition stopped while waiting for synchronized frames");
        return false;
    }
    
    // Check if any queue is empty (for non-blocking mode)
    if (std::any_of(mFrameQueues.begin(), mFrameQueues.end(),
                   [](const std::queue<FrameMetadata>& queue) {
                       return queue.empty();
                   })) {
        SetErrorMessage("Not all cameras have frames available");
        return false;
    }
    
    // Get frames from all queues
    metadata_vec.clear();
    metadata_vec.reserve(mFrameQueues.size());
    
    for (auto& queue : mFrameQueues) {
        metadata_vec.push_back(queue.front());
    }
    
    // Check if frames are synchronized
    double reference_time = metadata_vec[0].timestamp;
    bool synchronized = true;
    
    for (size_t i = 1; i < metadata_vec.size(); ++i) {
        double time_diff = std::abs(metadata_vec[i].timestamp - reference_time) * 1000.0; // Convert to ms
        if (time_diff > max_time_diff_ms) {
            synchronized = false;
            break;
        }
    }
    
    if (!synchronized) {
        // Frames are not synchronized, try to find a better set
        // This is a simple algorithm that tries to find a set of frames that are synchronized
        // A more sophisticated algorithm could be implemented if needed
        
        // Pop the oldest frame
        size_t oldest_idx = 0;
        double oldest_time = metadata_vec[0].timestamp;
        
        for (size_t i = 1; i < metadata_vec.size(); ++i) {
            if (metadata_vec[i].timestamp < oldest_time) {
                oldest_time = metadata_vec[i].timestamp;
                oldest_idx = i;
            }
        }
        
        mFrameQueues[oldest_idx].pop();
        
        // Release the lock and try again
        lock.unlock();
        return GetNextSynchronizedFrames(metadata_vec, max_time_diff_ms, timeout_ms);
    }
    
    // Pop frames from queues
    for (auto& queue : mFrameQueues) {
        queue.pop();
    }
    
    return true;
}

void ZeroCopyFrameProvider::ReleaseFrame(const FrameMetadata& metadata)
{
    // Check if camera_id is valid
    if (metadata.camera_id < 0 || metadata.camera_id >= static_cast<int>(mCameraHandles.size())) {
        SetErrorMessage("Invalid camera ID in metadata");
        return;
    }
    
    // Check if camera is initialized
    if (mCameraHandles[metadata.camera_id] < 0) {
        SetErrorMessage("Camera not initialized");
        return;
    }
    
    // Find the buffer
    for (auto& buffer : mBuffers[metadata.camera_id]) {
        if (buffer.start == metadata.buffer_ptr) {
            // Mark buffer as not in use
            buffer.in_use = false;
            
            // Re-queue the buffer for capture
            struct v4l2_buffer buf;
            memset(&buf, 0, sizeof(buf));
            buf.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
            buf.memory = V4L2_MEMORY_MMAP;
            buf.index = &buffer - &mBuffers[metadata.camera_id][0]; // Calculate buffer index
            
            if (ioctl(mCameraHandles[metadata.camera_id], VIDIOC_QBUF, &buf) < 0) {
                SetErrorMessage("Failed to re-queue buffer: " + std::string(strerror(errno)));
            }
            
            return;
        }
    }
    
    SetErrorMessage("Buffer not found");
}

cv::Mat ZeroCopyFrameProvider::GetMatForFrame(const FrameMetadata& metadata)
{
    // Check if buffer_ptr is valid
    if (!metadata.buffer_ptr) {
        SetErrorMessage("Invalid buffer pointer in metadata");
        return cv::Mat();
    }
    
    // Create cv::Mat wrapper for the buffer
    cv::Mat mat;
    
    if (metadata.pixel_format == "GREY") {
        // Grayscale image
        mat = cv::Mat(metadata.height, metadata.width, CV_8UC1, metadata.buffer_ptr);
    } else if (metadata.pixel_format == "YUYV") {
        // YUYV format (YUV 4:2:2)
        cv::Mat yuyv(metadata.height, metadata.width, CV_8UC2, metadata.buffer_ptr);
        cv::cvtColor(yuyv, mat, cv::COLOR_YUV2GRAY_YUYV);
    } else if (metadata.pixel_format == "MJPG") {
        // MJPEG format
        std::vector<uint8_t> buffer(static_cast<uint8_t*>(metadata.buffer_ptr),
                                   static_cast<uint8_t*>(metadata.buffer_ptr) + metadata.buffer_size);
        mat = cv::imdecode(buffer, cv::IMREAD_GRAYSCALE);
    } else {
        SetErrorMessage("Unsupported pixel format: " + metadata.pixel_format);
        return cv::Mat();
    }
    
    return mat;
}

int ZeroCopyFrameProvider::GetDmaFdForFrame(const FrameMetadata& metadata)
{
    return metadata.dma_fd;
}

ZeroCopyFrameProvider::CameraConfig ZeroCopyFrameProvider::GetCameraConfig(int camera_id) const
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(mCameraConfigs.size())) {
        throw std::out_of_range("Invalid camera ID");
    }
    
    return mCameraConfigs[camera_id];
}

bool ZeroCopyFrameProvider::SetCameraConfig(int camera_id, const CameraConfig& config)
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(mCameraConfigs.size())) {
        SetErrorMessage("Invalid camera ID");
        return false;
    }
    
    // Check if running
    if (mRunning) {
        SetErrorMessage("Cannot change configuration while acquisition is running");
        return false;
    }
    
    // Update configuration
    mCameraConfigs[camera_id] = config;
    
    // If camera is already initialized, reconfigure it
    if (mCameraHandles[camera_id] >= 0) {
        // Free buffers
        FreeBuffers(camera_id);
        
        // Reconfigure camera
        if (!ConfigureCamera(camera_id)) {
            return false;
        }
        
        // Allocate buffers
        if (!AllocateBuffers(camera_id)) {
            return false;
        }
    }
    
    return true;
}

bool ZeroCopyFrameProvider::IsCameraConnected(int camera_id) const
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(mCameraHandles.size())) {
        return false;
    }
    
    return mCameraHandles[camera_id] >= 0;
}

size_t ZeroCopyFrameProvider::GetCameraCount() const
{
    return mCameraConfigs.size();
}

float ZeroCopyFrameProvider::GetCurrentFrameRate(int camera_id) const
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(mCurrentFrameRates.size())) {
        return 0.0f;
    }
    
    return mCurrentFrameRates[camera_id];
}

void ZeroCopyFrameProvider::RegisterFrameCallback(std::function<void(const FrameMetadata&)> callback)
{
    mFrameCallback = callback;
}

bool ZeroCopyFrameProvider::EnableZeroCopy(int camera_id, bool enable)
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(mCameraConfigs.size())) {
        SetErrorMessage("Invalid camera ID");
        return false;
    }
    
    // Check if running
    if (mRunning) {
        SetErrorMessage("Cannot change zero-copy mode while acquisition is running");
        return false;
    }
    
    // Check if zero-copy is supported
    if (enable && !IsZeroCopySupported(camera_id)) {
        SetErrorMessage("Zero-copy not supported for this camera");
        return false;
    }
    
    // Update configuration
    mCameraConfigs[camera_id].zero_copy_enabled = enable;
    
    return true;
}

bool ZeroCopyFrameProvider::IsZeroCopySupported(int camera_id) const
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(mCameraHandles.size())) {
        return false;
    }
    
    // Check if camera is initialized
    if (mCameraHandles[camera_id] < 0) {
        return false;
    }
    
    // Check if DMA is supported
    return CheckDmaSupport(camera_id);
}

std::string ZeroCopyFrameProvider::GetLastErrorMessage() const
{
    std::lock_guard<std::mutex> lock(mErrorMutex);
    return mLastErrorMessage;
}

//------------------------------------------------------------------------------
// Private Methods
//------------------------------------------------------------------------------

bool ZeroCopyFrameProvider::OpenCamera(int camera_id)
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(mCameraConfigs.size())) {
        SetErrorMessage("Invalid camera ID");
        return false;
    }
    
    // Check if camera is already open
    if (mCameraHandles[camera_id] >= 0) {
        SetErrorMessage("Camera already open");
        return false;
    }
    
    // Open camera device
    const std::string& device_path = mCameraConfigs[camera_id].device_path;
    int fd = open(device_path.c_str(), O_RDWR);
    if (fd < 0) {
        SetErrorMessage("Failed to open camera device: " + device_path + " - " + std::string(strerror(errno)));
        return false;
    }
    
    // Check if device is a video capture device
    struct v4l2_capability cap;
    if (ioctl(fd, VIDIOC_QUERYCAP, &cap) < 0) {
        SetErrorMessage("Failed to query camera capabilities: " + std::string(strerror(errno)));
        close(fd);
        return false;
    }
    
    if (!(cap.capabilities & V4L2_CAP_VIDEO_CAPTURE)) {
        SetErrorMessage("Device is not a video capture device: " + device_path);
        close(fd);
        return false;
    }
    
    if (!(cap.capabilities & V4L2_CAP_STREAMING)) {
        SetErrorMessage("Device does not support streaming: " + device_path);
        close(fd);
        return false;
    }
    
    // Store camera handle
    mCameraHandles[camera_id] = fd;
    
    return true;
}

void ZeroCopyFrameProvider::CloseCamera(int camera_id)
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(mCameraHandles.size())) {
        return;
    }
    
    // Check if camera is open
    if (mCameraHandles[camera_id] < 0) {
        return;
    }
    
    // Close camera device
    close(mCameraHandles[camera_id]);
    mCameraHandles[camera_id] = -1;
}

bool ZeroCopyFrameProvider::ConfigureCamera(int camera_id)
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(mCameraConfigs.size())) {
        SetErrorMessage("Invalid camera ID");
        return false;
    }
    
    // Check if camera is open
    if (mCameraHandles[camera_id] < 0) {
        SetErrorMessage("Camera not open");
        return false;
    }
    
    // Get camera configuration
    const CameraConfig& config = mCameraConfigs[camera_id];
    
    // Set video format
    struct v4l2_format fmt;
    memset(&fmt, 0, sizeof(fmt));
    fmt.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    fmt.fmt.pix.width = config.width;
    fmt.fmt.pix.height = config.height;
    
    // Set pixel format
    if (config.pixel_format == "GREY") {
        fmt.fmt.pix.pixelformat = V4L2_PIX_FMT_GREY;
    } else if (config.pixel_format == "YUYV") {
        fmt.fmt.pix.pixelformat = V4L2_PIX_FMT_YUYV;
    } else if (config.pixel_format == "MJPG") {
        fmt.fmt.pix.pixelformat = V4L2_PIX_FMT_MJPEG;
    } else {
        SetErrorMessage("Unsupported pixel format: " + config.pixel_format);
        return false;
    }
    
    fmt.fmt.pix.field = V4L2_FIELD_NONE;
    
    if (ioctl(mCameraHandles[camera_id], VIDIOC_S_FMT, &fmt) < 0) {
        SetErrorMessage("Failed to set video format: " + std::string(strerror(errno)));
        return false;
    }
    
    // Check if format was set correctly
    if (fmt.fmt.pix.width != static_cast<unsigned int>(config.width) ||
        fmt.fmt.pix.height != static_cast<unsigned int>(config.height)) {
        SetErrorMessage("Camera does not support requested resolution: " +
                       std::to_string(config.width) + "x" + std::to_string(config.height) +
                       ", got: " + std::to_string(fmt.fmt.pix.width) + "x" + std::to_string(fmt.fmt.pix.height));
        return false;
    }
    
    // Set frame rate
    struct v4l2_streamparm parm;
    memset(&parm, 0, sizeof(parm));
    parm.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    
    if (ioctl(mCameraHandles[camera_id], VIDIOC_G_PARM, &parm) < 0) {
        SetErrorMessage("Failed to get stream parameters: " + std::string(strerror(errno)));
        return false;
    }
    
    if (!(parm.parm.capture.capability & V4L2_CAP_TIMEPERFRAME)) {
        SetErrorMessage("Camera does not support setting frame rate");
        return false;
    }
    
    parm.parm.capture.timeperframe.numerator = 1;
    parm.parm.capture.timeperframe.denominator = config.fps;
    
    if (ioctl(mCameraHandles[camera_id], VIDIOC_S_PARM, &parm) < 0) {
        SetErrorMessage("Failed to set frame rate: " + std::string(strerror(errno)));
        return false;
    }
    
    return true;
}

bool ZeroCopyFrameProvider::AllocateBuffers(int camera_id)
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(mCameraConfigs.size())) {
        SetErrorMessage("Invalid camera ID");
        return false;
    }
    
    // Check if camera is open
    if (mCameraHandles[camera_id] < 0) {
        SetErrorMessage("Camera not open");
        return false;
    }
    
    // Get camera configuration
    const CameraConfig& config = mCameraConfigs[camera_id];
    
    // Request buffers
    struct v4l2_requestbuffers req;
    memset(&req, 0, sizeof(req));
    req.count = config.buffer_count;
    req.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    req.memory = V4L2_MEMORY_MMAP;
    
    if (ioctl(mCameraHandles[camera_id], VIDIOC_REQBUFS, &req) < 0) {
        SetErrorMessage("Failed to request buffers: " + std::string(strerror(errno)));
        return false;
    }
    
    if (req.count < 2) {
        SetErrorMessage("Insufficient buffer memory");
        return false;
    }
    
    // Allocate buffer info structures
    mBuffers[camera_id].resize(req.count);
    
    // Map buffers
    for (unsigned int i = 0; i < req.count; ++i) {
        struct v4l2_buffer buf;
        memset(&buf, 0, sizeof(buf));
        buf.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
        buf.memory = V4L2_MEMORY_MMAP;
        buf.index = i;
        
        if (ioctl(mCameraHandles[camera_id], VIDIOC_QUERYBUF, &buf) < 0) {
            SetErrorMessage("Failed to query buffer: " + std::string(strerror(errno)));
            FreeBuffers(camera_id);
            return false;
        }
        
        mBuffers[camera_id][i].length = buf.length;
        mBuffers[camera_id][i].start = mmap(nullptr, buf.length,
                                          PROT_READ | PROT_WRITE, MAP_SHARED,
                                          mCameraHandles[camera_id], buf.m.offset);
        
        if (mBuffers[camera_id][i].start == MAP_FAILED) {
            SetErrorMessage("Failed to map buffer: " + std::string(strerror(errno)));
            FreeBuffers(camera_id);
            return false;
        }
        
        mBuffers[camera_id][i].in_use = false;
        
        // Export DMA file descriptor if zero-copy is enabled
        if (config.zero_copy_enabled) {
            mBuffers[camera_id][i].dma_fd = ExportDmaBuffer(camera_id, i);
            if (mBuffers[camera_id][i].dma_fd < 0) {
                SetErrorMessage("Failed to export DMA buffer");
                FreeBuffers(camera_id);
                return false;
            }
        } else {
            mBuffers[camera_id][i].dma_fd = -1;
        }
    }
    
    return true;
}

void ZeroCopyFrameProvider::FreeBuffers(int camera_id)
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(mBuffers.size())) {
        return;
    }
    
    // Unmap buffers
    for (auto& buffer : mBuffers[camera_id]) {
        if (buffer.start != nullptr && buffer.start != MAP_FAILED) {
            munmap(buffer.start, buffer.length);
        }
        
        if (buffer.dma_fd >= 0) {
            close(buffer.dma_fd);
        }
    }
    
    // Clear buffer info
    mBuffers[camera_id].clear();
}

bool ZeroCopyFrameProvider::StartStreaming(int camera_id)
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(mCameraHandles.size())) {
        SetErrorMessage("Invalid camera ID");
        return false;
    }
    
    // Check if camera is open
    if (mCameraHandles[camera_id] < 0) {
        SetErrorMessage("Camera not open");
        return false;
    }
    
    // Queue buffers
    for (size_t i = 0; i < mBuffers[camera_id].size(); ++i) {
        struct v4l2_buffer buf;
        memset(&buf, 0, sizeof(buf));
        buf.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
        buf.memory = V4L2_MEMORY_MMAP;
        buf.index = i;
        
        if (ioctl(mCameraHandles[camera_id], VIDIOC_QBUF, &buf) < 0) {
            SetErrorMessage("Failed to queue buffer: " + std::string(strerror(errno)));
            return false;
        }
    }
    
    // Start streaming
    enum v4l2_buf_type type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    if (ioctl(mCameraHandles[camera_id], VIDIOC_STREAMON, &type) < 0) {
        SetErrorMessage("Failed to start streaming: " + std::string(strerror(errno)));
        return false;
    }
    
    return true;
}

void ZeroCopyFrameProvider::StopStreaming(int camera_id)
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(mCameraHandles.size())) {
        return;
    }
    
    // Check if camera is open
    if (mCameraHandles[camera_id] < 0) {
        return;
    }
    
    // Stop streaming
    enum v4l2_buf_type type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    ioctl(mCameraHandles[camera_id], VIDIOC_STREAMOFF, &type);
}

void ZeroCopyFrameProvider::AcquisitionThreadFunc(int camera_id)
{
    // Set thread name
    pthread_setname_np(pthread_self(), ("ZeroCopyAcq" + std::to_string(camera_id)).c_str());
    
    // Initialize frame counter and timer
    uint64_t frame_counter = 0;
    auto last_fps_update = std::chrono::steady_clock::now();
    auto last_frame_time = last_fps_update;
    
    // Acquisition loop
    while (mRunning) {
        // Wait for a buffer
        fd_set fds;
        FD_ZERO(&fds);
        FD_SET(mCameraHandles[camera_id], &fds);
        
        struct timeval tv;
        tv.tv_sec = 1;
        tv.tv_usec = 0;
        
        int r = select(mCameraHandles[camera_id] + 1, &fds, nullptr, nullptr, &tv);
        
        if (r < 0) {
            if (errno == EINTR) {
                continue;
            }
            
            SetErrorMessage("Select error: " + std::string(strerror(errno)));
            break;
        }
        
        if (r == 0) {
            // Timeout
            continue;
        }
        
        // Dequeue buffer
        struct v4l2_buffer buf;
        memset(&buf, 0, sizeof(buf));
        buf.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
        buf.memory = V4L2_MEMORY_MMAP;
        
        if (ioctl(mCameraHandles[camera_id], VIDIOC_DQBUF, &buf) < 0) {
            if (errno == EAGAIN) {
                continue;
            }
            
            SetErrorMessage("Failed to dequeue buffer: " + std::string(strerror(errno)));
            break;
        }
        
        // Mark buffer as in use
        mBuffers[camera_id][buf.index].in_use = true;
        
        // Create frame metadata
        FrameMetadata metadata;
        metadata.frame_id = frame_counter++;
        metadata.timestamp = buf.timestamp.tv_sec + buf.timestamp.tv_usec / 1000000.0;
        metadata.camera_id = camera_id;
        metadata.width = mCameraConfigs[camera_id].width;
        metadata.height = mCameraConfigs[camera_id].height;
        metadata.pixel_format = mCameraConfigs[camera_id].pixel_format;
        metadata.buffer_ptr = mBuffers[camera_id][buf.index].start;
        metadata.buffer_size = buf.bytesused;
        metadata.dma_fd = mBuffers[camera_id][buf.index].dma_fd;
        metadata.is_keyframe = (buf.flags & V4L2_BUF_FLAG_KEYFRAME) != 0;
        
        // Update frame rate
        auto now = std::chrono::steady_clock::now();
        auto frame_interval = std::chrono::duration_cast<std::chrono::microseconds>(now - last_frame_time).count();
        last_frame_time = now;
        
        auto fps_update_interval = std::chrono::duration_cast<std::chrono::milliseconds>(now - last_fps_update).count();
        if (fps_update_interval > 1000) {
            // Update FPS every second
            mCurrentFrameRates[camera_id] = mFrameCounters[camera_id] * 1000.0f / fps_update_interval;
            mFrameCounters[camera_id] = 0;
            last_fps_update = now;
        } else {
            mFrameCounters[camera_id]++;
        }
        
        // Add frame to queue
        {
            std::lock_guard<std::mutex> lock(mFrameQueueMutex);
            mFrameQueues[camera_id].push(metadata);
        }
        
        // Notify waiting threads
        mFrameCondition.notify_all();
        
        // Call frame callback if registered
        if (mFrameCallback) {
            mFrameCallback(metadata);
        }
    }
}

void ZeroCopyFrameProvider::SetErrorMessage(const std::string& message)
{
    std::lock_guard<std::mutex> lock(mErrorMutex);
    mLastErrorMessage = message;
    std::cerr << "ZeroCopyFrameProvider error: " << message << std::endl;
}

bool ZeroCopyFrameProvider::CheckDmaSupport(int camera_id) const
{
    // This is a simplified check for DMA support
    // In a real implementation, this would involve checking for specific V4L2 capabilities
    // and possibly testing if DMA export works
    
    // For now, we'll assume DMA is supported if the camera is open
    return mCameraHandles[camera_id] >= 0;
}

int ZeroCopyFrameProvider::ExportDmaBuffer(int camera_id, int buffer_index)
{
    // Check if camera_id is valid
    if (camera_id < 0 || camera_id >= static_cast<int>(mCameraHandles.size())) {
        SetErrorMessage("Invalid camera ID");
        return -1;
    }
    
    // Check if camera is open
    if (mCameraHandles[camera_id] < 0) {
        SetErrorMessage("Camera not open");
        return -1;
    }
    
    // Check if buffer_index is valid
    if (buffer_index < 0 || buffer_index >= static_cast<int>(mBuffers[camera_id].size())) {
        SetErrorMessage("Invalid buffer index");
        return -1;
    }
    
    // Export DMA buffer
    struct v4l2_exportbuffer expbuf;
    memset(&expbuf, 0, sizeof(expbuf));
    expbuf.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    expbuf.index = buffer_index;
    expbuf.flags = O_RDONLY;
    
    if (ioctl(mCameraHandles[camera_id], VIDIOC_EXPBUF, &expbuf) < 0) {
        SetErrorMessage("Failed to export buffer: " + std::string(strerror(errno)));
        return -1;
    }
    
    return expbuf.fd;
}

} // namespace ORB_SLAM3

#include "include/tpu_feature_extractor.hpp"
#include <iostream> // For std::cerr, std::cout
#include <fstream>  // For std::ifstream
#include <algorithm> // For std::min, std::max
#include <chrono> // For performance measurement
#include <thread> // For parallel processing
#include <mutex> // For thread synchronization

// TensorFlow Lite and EdgeTPU Delegate headers
#include "tensorflow/lite/interpreter.h"
#include "tensorflow/lite/kernels/register.h"
#include "tensorflow/lite/model.h"
#include "tensorflow/lite/optional_debug_tools.h"

// EdgeTPU delegate header
#if defined(__APPLE__)
    #include "tensorflow/lite/delegates/coreml/coreml_delegate.h"
#elif defined(_WIN32)
    // Windows specific, if any, or generic TFLite delegate API
#else
    #include "tensorflow/lite/delegates/edgetpu/edgetpu_delegate.h"
#endif

#include "opencv2/imgproc.hpp" // For cv::resize, cv::cvtColor
#include "opencv2/highgui.hpp" // For cv::imread (if testing standalone)

// OpenMP for parallel processing if available
#ifdef _OPENMP
    #include <omp.h>
#endif

namespace ORB_SLAM3
{

// Performance tracking variables
static double total_preprocess_time = 0;
static double total_inference_time = 0;
static double total_postprocess_time = 0;
static int frame_count = 0;

TPUFeatureExtractor::TPUFeatureExtractor(
    const std::string& model_path,
    const std::string& delegate_path,
    int n_features_target,
    float scale_factor,
    int n_levels)
    : model_path_(model_path),
      delegate_path_(delegate_path),
      n_features_target_(n_features_target),
      scale_factor_(scale_factor),
      n_levels_(n_levels),
      edgetpu_delegate_(nullptr),
      descriptor_output_index_(-1),
      semi_output_index_(-1),
      nms_radius_(4.0f),
      confidence_threshold_(0.005f)
{
    try {
        if (!loadModel()) {
            throw std::runtime_error("Failed to load EdgeTPU model: " + model_path_);
        }
        initializeScaleFactors();
        std::cout << "TPUFeatureExtractor initialized with model: " << model_path_ << std::endl;
    } catch (const std::exception& e) {
        std::cerr << "Error initializing TPUFeatureExtractor: " << e.what() << std::endl;
        throw; // Re-throw to notify caller
    }
}

TPUFeatureExtractor::~TPUFeatureExtractor()
{
    // Print performance statistics
    if (frame_count > 0) {
        std::cout << "TPUFeatureExtractor Performance Statistics:" << std::endl;
        std::cout << "  Average preprocessing time: " << (total_preprocess_time / frame_count) << " ms" << std::endl;
        std::cout << "  Average inference time: " << (total_inference_time / frame_count) << " ms" << std::endl;
        std::cout << "  Average postprocessing time: " << (total_postprocess_time / frame_count) << " ms" << std::endl;
        std::cout << "  Total average time per frame: " 
                  << ((total_preprocess_time + total_inference_time + total_postprocess_time) / frame_count) 
                  << " ms" << std::endl;
        std::cout << "  Frames processed: " << frame_count << std::endl;
    }
    
    // Cleanup EdgeTPU delegate if needed
    #if !defined(__APPLE__) && !defined(_WIN32)
        if (edgetpu_delegate_) {
            edgetpu_free_delegate(edgetpu_delegate_);
            edgetpu_delegate_ = nullptr;
        }
    #endif
    
    std::cout << "TPUFeatureExtractor destroyed." << std::endl;
}

bool TPUFeatureExtractor::loadModel()
{
    // 1. Load the TFLite model
    model_ = tflite::FlatBufferModel::BuildFromFile(model_path_.c_str());
    if (!model_) {
        std::cerr << "Failed to load TFLite model from: " << model_path_ << std::endl;
        return false;
    }
    std::cout << "Successfully loaded TFLite model: " << model_path_ << std::endl;

    // 2. Create TFLite Interpreter
    tflite::ops::builtin::BuiltinOpResolver resolver;
    tflite::InterpreterBuilder builder(*model_, resolver);

    // 3. Prepare and apply EdgeTPU delegate
    #if !defined(__APPLE__) && !defined(_WIN32)
        // For Linux/EdgeTPU: Attempt to load the EdgeTPU delegate
        auto* delegate = edgetpu_create_delegate(nullptr, nullptr, 0); // Use default device
        if (delegate) {
            std::cout << "EdgeTPU delegate created successfully." << std::endl;
            builder.AddDelegate(delegate);
            edgetpu_delegate_ = delegate;
        } else {
            std::cerr << "Warning: EdgeTPU delegate not available or failed to load. Will run on CPU." << std::endl;
        }
    #elif defined(__APPLE__)
        std::cout << "Note: EdgeTPU delegate is not typically used on macOS." << std::endl;
    #else
        std::cout << "Note: EdgeTPU delegate is not typically used on Windows." << std::endl;
    #endif

    // 4. Build interpreter
    if (builder(&interpreter_) != kTfLiteOk) {
        std::cerr << "Failed to build TFLite interpreter." << std::endl;
        return false;
    }
    
    // 5. Set number of threads for CPU fallback
    interpreter_->SetNumThreads(4); // Use multiple threads for CPU operations
    
    // 6. Allocate tensors
    if (interpreter_->AllocateTensors() != kTfLiteOk) {
        std::cerr << "Failed to allocate TFLite tensors." << std::endl;
        return false;
    }
    std::cout << "TFLite tensors allocated successfully." << std::endl;

    // 7. Get input tensor details
    const auto& inputs = interpreter_->inputs();
    if (inputs.empty()) {
        std::cerr << "Model has no input tensors." << std::endl;
        return false;
    }
    int input_tensor_index = inputs[0];
    TfLiteIntArray* input_dims = interpreter_->tensor(input_tensor_index)->dims;
    if (input_dims->size < 3) {
         std::cerr << "Input tensor has unexpected dimensions: " << input_dims->size << std::endl;
         return false;
    }
    
    // Parse input dimensions
    if (input_dims->size == 4) { // BHWC
        input_tensor_height_ = input_dims->data[1];
        input_tensor_width_ = input_dims->data[2];
        input_tensor_channels_ = input_dims->data[3];
    } else if (input_dims->size == 3) { // HWC
        input_tensor_height_ = input_dims->data[0];
        input_tensor_width_ = input_dims->data[1];
        input_tensor_channels_ = input_dims->data[2];
    } else {
        std::cerr << "Unsupported input tensor dimension size: " << input_dims->size << std::endl;
        return false;
    }

    std::cout << "Model Input Details: Height=" << input_tensor_height_ 
              << ", Width=" << input_tensor_width_ 
              << ", Channels=" << input_tensor_channels_ << std::endl;
    
    // 8. Get output tensor details
    const auto& outputs = interpreter_->outputs();
    if (outputs.size() < 2) {
        std::cerr << "Expected at least 2 output tensors (descriptors and semi), but got " 
                  << outputs.size() << std::endl;
        return false;
    }
    
    // Find descriptor and semi outputs by checking tensor shapes
    for (int i = 0; i < outputs.size(); ++i) {
        int output_idx = outputs[i];
        TfLiteTensor* tensor = interpreter_->tensor(output_idx);
        TfLiteIntArray* dims = tensor->dims;
        
        // Print output tensor details for debugging
        std::cout << "Output tensor " << i << " (index " << output_idx << "): ";
        std::cout << "Name=" << tensor->name << ", ";
        std::cout << "Shape=[";
        for (int j = 0; j < dims->size; ++j) {
            std::cout << dims->data[j];
            if (j < dims->size - 1) std::cout << ",";
        }
        std::cout << "], ";
        std::cout << "Type=" << TfLiteTypeGetName(tensor->type) << std::endl;
        
        // Check if this is the descriptor tensor (256 channels)
        if (dims->size == 4 && dims->data[1] == 256) {
            descriptor_output_index_ = output_idx;
            descriptor_height_ = dims->data[2];
            descriptor_width_ = dims->data[3];
            descriptor_channels_ = dims->data[1];
            
            // Get quantization parameters
            descriptor_quant_scale_ = tensor->params.scale;
            descriptor_quant_zero_point_ = tensor->params.zero_point;
            
            std::cout << "Found descriptor tensor: index=" << descriptor_output_index_ 
                      << ", shape=[1," << descriptor_channels_ << "," 
                      << descriptor_height_ << "," << descriptor_width_ << "]" 
                      << ", scale=" << descriptor_quant_scale_
                      << ", zero_point=" << descriptor_quant_zero_point_ << std::endl;
        }
        // Check if this is the semi tensor (keypoints, 65 channels in NHWC format)
        else if (dims->size == 4 && dims->data[3] == 65) {
            semi_output_index_ = output_idx;
            semi_height_ = dims->data[1];
            semi_width_ = dims->data[2];
            semi_channels_ = dims->data[3];
            
            // Get quantization parameters
            semi_quant_scale_ = tensor->params.scale;
            semi_quant_zero_point_ = tensor->params.zero_point;
            
            std::cout << "Found semi tensor: index=" << semi_output_index_ 
                      << ", shape=[1," << semi_height_ << "," 
                      << semi_width_ << "," << semi_channels_ << "]" 
                      << ", scale=" << semi_quant_scale_
                      << ", zero_point=" << semi_quant_zero_point_ << std::endl;
        }
    }
    
    // Verify that we found both output tensors
    if (descriptor_output_index_ == -1 || semi_output_index_ == -1) {
        std::cerr << "Failed to identify descriptor or semi output tensors." << std::endl;
        return false;
    }

    return true;
}

void TPUFeatureExtractor::initializeScaleFactors()
{
    inv_scale_factors_.resize(n_levels_);
    scale_factors_.resize(n_levels_);
    level_sigma2_.resize(n_levels_);
    inv_level_sigma2_.resize(n_levels_);

    scale_factors_[0] = 1.0f;
    inv_scale_factors_[0] = 1.0f;
    level_sigma2_[0] = 1.0f;
    inv_level_sigma2_[0] = 1.0f;

    for (int i = 1; i < n_levels_; i++) {
        scale_factors_[i] = scale_factors_[i - 1] * scale_factor_;
        inv_scale_factors_[i] = 1.0f / scale_factors_[i];
        level_sigma2_[i] = scale_factors_[i] * scale_factors_[i];
        inv_level_sigma2_[i] = 1.0f / level_sigma2_[i];
    }
}

void TPUFeatureExtractor::createImagePyramid(const cv::Mat& image)
{
    // Clear previous pyramid if any
    mvImagePyramid.clear();
    mvImagePyramid.resize(n_levels_);
    
    // Copy the original image to the first level
    image.copyTo(mvImagePyramid[0]);
    
    // Create the rest of the pyramid
    for (int level = 1; level < n_levels_; ++level)
    {
        float scale = inv_scale_factors_[level];
        cv::Size sz(cvRound((float)image.cols * scale), cvRound((float)image.rows * scale));
        cv::resize(mvImagePyramid[level-1], mvImagePyramid[level], sz, 0, 0, cv::INTER_LINEAR);
    }
}

cv::Mat TPUFeatureExtractor::preprocessImage(const cv::Mat& input_image, const cv::Size& model_input_size)
{
    cv::Mat processed_image;
    
    // 1. Convert to Grayscale if needed
    if (input_tensor_channels_ == 1 && input_image.channels() != 1) {
        cv::cvtColor(input_image, processed_image, cv::COLOR_BGR2GRAY);
    } else if (input_tensor_channels_ == 3 && input_image.channels() == 1) {
        cv::cvtColor(input_image, processed_image, cv::COLOR_GRAY2BGR);
    } else {
        input_image.copyTo(processed_image);
    }

    // 2. Resize to model's expected input dimensions
    if (processed_image.rows != model_input_size.height || processed_image.cols != model_input_size.width) {
        cv::resize(processed_image, processed_image, model_input_size, 0, 0, cv::INTER_LINEAR);
    }

    // 3. Ensure the image is in uint8 format for quantization
    if (processed_image.type() != CV_8UC1 && processed_image.type() != CV_8UC3) {
        processed_image.convertTo(processed_image, CV_8U);
    }
    
    // 4. Ensure memory is continuous for faster copying
    if (!processed_image.isContinuous()) {
        processed_image = processed_image.clone();
    }
    
    return processed_image;
}

void TPUFeatureExtractor::runInference(
    const cv::Mat& preprocessed_image,
    std::vector<float>& raw_keypoints,
    std::vector<float>& raw_descriptors,
    std::vector<float>& raw_scores)
{
    if (!interpreter_) {
        std::cerr << "Interpreter not initialized." << std::endl;
        return;
    }

    // Get pointer to input tensor data
    TfLiteTensor* input_tensor_ptr = interpreter_->tensor(interpreter_->inputs()[0]);
    
    if (input_tensor_ptr->type == kTfLiteInt8) {
        // Model expects int8 input, need to convert from uint8 to int8
        int8_t* input_data = interpreter_->typed_input_tensor<int8_t>(0);
        
        // Convert uint8 image data to int8 with zero_point shift
        // For the analyzed model, zero_point is -128, which means uint8_value - 128 = int8_value
        if (preprocessed_image.isContinuous()) {
            const uint8_t* img_data = preprocessed_image.data;
            const size_t total_elements = preprocessed_image.total() * preprocessed_image.elemSize();
            
            // Use OpenMP for parallel processing if available
            #pragma omp parallel for if(total_elements > 10000)
            for (size_t i = 0; i < total_elements; ++i) {
                input_data[i] = static_cast<int8_t>(img_data[i]) - 128; // Apply zero point shift
            }
        } else {
            // Handle non-continuous Mat (slower)
            for (int i = 0; i < preprocessed_image.rows; ++i) {
                const uint8_t* row_ptr = preprocessed_image.ptr<uint8_t>(i);
                for (int j = 0; j < preprocessed_image.cols * preprocessed_image.channels(); ++j) {
                    int idx = i * preprocessed_image.cols * preprocessed_image.channels() + j;
                    input_data[idx] = static_cast<int8_t>(row_ptr[j]) - 128; // Apply zero point shift
                }
            }
        }
    } else {
        std::cerr << "Unexpected input tensor type: " << TfLiteTypeGetName(input_tensor_ptr->type) 
                  << ". Expected INT8." << std::endl;
        return;
    }

    // Run inference
    if (interpreter_->Invoke() != kTfLiteOk) {
        std::cerr << "Failed to invoke TFLite interpreter." << std::endl;
        return;
    }

    // Extract descriptor output tensor
    if (descriptor_output_index_ != -1) {
        const TfLiteTensor* descriptor_tensor = interpreter_->tensor(descriptor_output_index_);
        const int8_t* descriptor_data = descriptor_tensor->data.int8;
        
        // Calculate total elements in descriptor tensor
        int total_elements = descriptor_channels_ * descriptor_height_ * descriptor_width_;
        
        // Resize output vector
        raw_descriptors.resize(total_elements);
        
        // Dequantize descriptor data
        // Use OpenMP for parallel processing if available
        #pragma omp parallel for if(total_elements > 10000)
        for (int i = 0; i < total_elements; ++i) {
            // Dequantize: (int8_value - zero_point) * scale
            raw_descriptors[i] = (descriptor_data[i] - descriptor_quant_zero_point_) * descriptor_quant_scale_;
        }
    } else {
        std::cerr << "Descriptor output tensor not found." << std::endl;
    }
    
    // Extract semi (keypoints) output tensor
    if (semi_output_index_ != -1) {
        const TfLiteTensor* semi_tensor = interpreter_->tensor(semi_output_index_);
        const int8_t* semi_data = semi_tensor->data.int8;
        
        // Calculate total elements in semi tensor
        int score_channels = semi_channels_ - 1; // 64 score channels
        int total_elements = semi_height_ * semi_width_ * score_channels;
        
        // Resize output vectors
        raw_scores.resize(total_elements);
        
        // Dequantize semi data
        // Use OpenMP for parallel processing if available
        #pragma omp parallel for collapse(2) if(semi_height_ * semi_width_ > 100)
        for (int h = 0; h < semi_height_; ++h) {
            for (int w = 0; w < semi_width_; ++w) {
                for (int c = 0; c < score_channels; ++c) {
                    int idx = h * semi_width_ * semi_channels_ + w * semi_channels_ + c;
                    int out_idx = h * semi_width_ * score_channels + w * score_channels + c;
                    
                    // Dequantize: (int8_value - zero_point) * scale
                    raw_scores[out_idx] = (semi_data[idx] - semi_quant_zero_point_) * semi_quant_scale_;
                }
            }
        }
    } else {
        std::cerr << "Semi output tensor not found." << std::endl;
    }
}

std::vector<cv::KeyPoint> TPUFeatureExtractor::applyNMS(
    const std::vector<std::vector<std::vector<float>>>& scores,
    float radius,
    float threshold)
{
    std::vector<cv::KeyPoint> keypoints;
    const int cell_size = 8; // SuperPoint uses 8x8 cells
    
    // First pass: find all points above threshold
    std::vector<cv::KeyPoint> candidates;
    for (int h = 0; h < semi_height_; ++h) {
        for (int w = 0; w < semi_width_; ++w) {
            for (int c = 0; c < 64; ++c) { // 64 score channels
                float score = scores[h][w][c];
                if (score > threshold) {
                    // Convert cell index to pixel coordinates
                    int cell_h = c / 8;
                    int cell_w = c % 8;
                    
                    // Calculate pixel coordinates in the feature map
                    float x = w * cell_size + cell_w;
                    float y = h * cell_size + cell_h;
                    
                    candidates.emplace_back(x, y, 8.0f, -1, score, 0);
                }
            }
        }
    }
    
    // Sort candidates by score (descending)
    std::sort(candidates.begin(), candidates.end(), 
             [](const cv::KeyPoint& a, const cv::KeyPoint& b) {
                 return a.response > b.response;
             });
    
    // Second pass: apply NMS
    std::vector<bool> is_suppressed(candidates.size(), false);
    
    for (size_t i = 0; i < candidates.size(); ++i) {
        if (is_suppressed[i]) continue;
        
        keypoints.push_back(candidates[i]);
        
        // Suppress nearby points
        for (size_t j = i + 1; j < candidates.size(); ++j) {
            if (is_suppressed[j]) continue;
            
            float dx = candidates[i].pt.x - candidates[j].pt.x;
            float dy = candidates[i].pt.y - candidates[j].pt.y;
            float dist_sq = dx * dx + dy * dy;
            
            if (dist_sq < radius * radius) {
                is_suppressed[j] = true;
            }
        }
    }
    
    return keypoints;
}

void TPUFeatureExtractor::postprocessResults(
    const cv::Mat& original_image,
    const cv::InputArray& mask_in,
    const std::vector<float>& raw_keypoints,
    const std::vector<float>& raw_descriptors,
    const std::vector<float>& raw_scores,
    std::vector<cv::KeyPoint>& keypoints,
    cv::Mat& descriptors,
    std::vector<int>& vLappingArea)
{
    keypoints.clear();
    
    // Get mask if provided
    cv::Mat mask = mask_in.getMat();
    
    // 1. Reshape the raw_scores into a 3D tensor [H, W, C]
    int score_channels = 64; // First 64 channels are keypoint scores
    
    // Create a 3D tensor for scores
    std::vector<std::vector<std::vector<float>>> score_map(
        semi_height_, 
        std::vector<std::vector<float>>(
            semi_width_, 
            std::vector<float>(score_channels, 0.0f)
        )
    );
    
    // Fill the score map
    for (int h = 0; h < semi_height_; ++h) {
        for (int w = 0; w < semi_width_; ++w) {
            for (int c = 0; c < score_channels; ++c) {
                int idx = h * semi_width_ * score_channels + w * score_channels + c;
                if (idx < raw_scores.size()) {
                    score_map[h][w][c] = raw_scores[idx];
                }
            }
        }
    }
    
    // 2. Apply non-maximum suppression to extract keypoints
    std::vector<cv::KeyPoint> nms_keypoints = applyNMS(score_map, nms_radius_, confidence_threshold_);
    
    // 3. Map keypoints to original image coordinates
    for (auto& kp : nms_keypoints) {
        // Convert feature map coordinates to original image coordinates
        float x = kp.pt.x * original_image.cols / (semi_width_ * 8);
        float y = kp.pt.y * original_image.rows / (semi_height_ * 8);
        
        // Check if the point is within the mask (if provided)
        if (!mask.empty()) {
            int img_x = static_cast<int>(x);
            int img_y = static_cast<int>(y);
            
            // Ensure coordinates are within image bounds
            if (img_x < 0 || img_x >= original_image.cols || 
                img_y < 0 || img_y >= original_image.rows) {
                continue;
            }
            
            // Check mask value
            if (mask.at<uchar>(img_y, img_x) == 0) {
                continue; // Skip masked points
            }
        }
        
        // Update keypoint coordinates
        kp.pt.x = x;
        kp.pt.y = y;
        keypoints.push_back(kp);
    }
    
    // 4. If we have a target number of features, keep the top N
    if (n_features_target_ > 0 && keypoints.size() > static_cast<size_t>(n_features_target_)) {
        // Sort by score (descending)
        std::sort(keypoints.begin(), keypoints.end(), 
                 [](const cv::KeyPoint& a, const cv::KeyPoint& b) {
                     return a.response > b.response;
                 });
        
        // Keep only the top n_features_target_ keypoints
        keypoints.resize(n_features_target_);
    }
    
    // 5. Extract descriptors for the selected keypoints
    int num_keypoints = keypoints.size();
    descriptors = cv::Mat(num_keypoints, descriptor_channels_, CV_32F);
    
    // Use OpenMP for parallel processing if available
    #pragma omp parallel for if(num_keypoints > 50)
    for (int i = 0; i < num_keypoints; ++i) {
        // Convert keypoint coordinates to descriptor map coordinates
        int desc_w = static_cast<int>(keypoints[i].pt.x * descriptor_width_ / original_image.cols);
        int desc_h = static_cast<int>(keypoints[i].pt.y * descriptor_height_ / original_image.rows);
        
        // Clamp to valid range
        desc_w = std::max(0, std::min(desc_w, descriptor_width_ - 1));
        desc_h = std::max(0, std::min(desc_h, descriptor_height_ - 1));
        
        // Extract descriptor for this keypoint
        for (int c = 0; c < descriptor_channels_; ++c) {
            int idx = c * descriptor_height_ * descriptor_width_ + 
                      desc_h * descriptor_width_ + desc_w;
            
            if (idx < raw_descriptors.size()) {
                descriptors.at<float>(i, c) = raw_descriptors[idx];
            } else {
                descriptors.at<float>(i, c) = 0.0f;
            }
        }
        
        // Normalize descriptor to unit length (L2 norm)
        cv::Mat desc_row = descriptors.row(i);
        float norm = cv::norm(desc_row);
        if (norm > 1e-6) {
            desc_row /= norm;
        }
    }
    
    // 6. Handle vLappingArea (specific to ORB-SLAM3)
    vLappingArea.clear();
}

int TPUFeatureExtractor::operator()(
    cv::InputArray image_in,
    cv::InputArray mask_in,
    std::vector<cv::KeyPoint>& keypoints,
    cv::OutputArray descriptors_out,
    std::vector<int>& vLappingArea)
{
    if (image_in.empty()) {
        return 0;
    }
    
    // Performance tracking
    auto start_time = std::chrono::high_resolution_clock::now();
    auto preprocess_start = start_time;
    
    cv::Mat image = image_in.getMat();
    
    // Create image pyramid for compatibility with ORB-SLAM3
    createImagePyramid(image);

    // 1. Preprocess image
    cv::Mat preprocessed_img = preprocessImage(image, cv::Size(input_tensor_width_, input_tensor_height_));
    
    auto inference_start = std::chrono::high_resolution_clock::now();
    auto preprocess_duration = std::chrono::duration_cast<std::chrono::milliseconds>(
        inference_start - preprocess_start).count();
    
    // 2. Run inference
    std::vector<float> raw_kpts, raw_desc, raw_scores;
    runInference(preprocessed_img, raw_kpts, raw_desc, raw_scores);
    
    auto postprocess_start = std::chrono::high_resolution_clock::now();
    auto inference_duration = std::chrono::duration_cast<std::chrono::milliseconds>(
        postprocess_start - inference_start).count();

    // 3. Postprocess results
    cv::Mat descriptors_mat;
    postprocessResults(image, mask_in, raw_kpts, raw_desc, raw_scores, keypoints, descriptors_mat, vLappingArea);

    // 4. Copy descriptors to output
    descriptors_out.create(descriptors_mat.size(), descriptors_mat.type());
    descriptors_mat.copyTo(descriptors_out.getMat());
    
    auto end_time = std::chrono::high_resolution_clock::now();
    auto postprocess_duration = std::chrono::duration_cast<std::chrono::milliseconds>(
        end_time - postprocess_start).count();
    
    // Update performance tracking
    total_preprocess_time += preprocess_duration;
    total_inference_time += inference_duration;
    total_postprocess_time += postprocess_duration;
    frame_count++;
    
    // Log performance for this frame
    if (frame_count % 10 == 0) {
        std::cout << "Frame " << frame_count << " timing: "
                  << "preprocess=" << preprocess_duration << "ms, "
                  << "inference=" << inference_duration << "ms, "
                  << "postprocess=" << postprocess_duration << "ms, "
                  << "total=" << (preprocess_duration + inference_duration + postprocess_duration) << "ms, "
                  << "keypoints=" << keypoints.size() << std::endl;
    }

    return static_cast<int>(keypoints.size());
}

// --- Implementation of ORBextractor-like public methods ---
int TPUFeatureExtractor::GetLevels() { return n_levels_; }
float TPUFeatureExtractor::GetScaleFactor() { return scale_factor_; }
std::vector<float> TPUFeatureExtractor::GetScaleFactors() { return scale_factors_; }
std::vector<float> TPUFeatureExtractor::GetInverseScaleFactors() { return inv_scale_factors_; }
std::vector<float> TPUFeatureExtractor::GetScaleSigmaSquares() { return level_sigma2_; }
std::vector<float> TPUFeatureExtractor::GetInverseScaleSigmaSquares() { return inv_level_sigma2_; }


} // namespace ORB_SLAM3

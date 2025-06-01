#ifndef TPU_FEATURE_EXTRACTOR_HPP
#define TPU_FEATURE_EXTRACTOR_HPP

#include <vector>
#include <string>
#include <memory>
#include "opencv2/core/mat.hpp"
#include "opencv2/features2d.hpp"

// Forward declarations for TensorFlow Lite and EdgeTPU
namespace tflite {
    class Interpreter;
    class FlatBufferModel;
}

namespace ORB_SLAM3
{

/**
 * @brief TPU-accelerated feature extractor using SuperPoint model
 * 
 * This class implements a feature extractor compatible with ORB-SLAM3
 * that uses the EdgeTPU to accelerate the SuperPoint neural network model.
 * It is designed to be a drop-in replacement for ORBextractor.
 */
class TPUFeatureExtractor
{
public:
    /**
     * @brief Constructor
     * 
     * @param model_path Path to the TFLite model file (EdgeTPU-compiled SuperPoint)
     * @param delegate_path Path to EdgeTPU delegate library (optional, usually auto-detected)
     * @param n_features_target Target number of features to extract
     * @param scale_factor Scale factor between pyramid levels
     * @param n_levels Number of pyramid levels
     */
    TPUFeatureExtractor(
        const std::string& model_path,
        const std::string& delegate_path,
        int n_features_target,
        float scale_factor,
        int n_levels);
    
    /**
     * @brief Destructor
     */
    ~TPUFeatureExtractor();

    /**
     * @brief Extract features from an image
     * 
     * This operator matches the interface of ORBextractor in ORB-SLAM3
     * 
     * @param image_in Input image
     * @param mask_in Optional mask (regions to ignore)
     * @param keypoints Output vector of detected keypoints
     * @param descriptors_out Output matrix of descriptors
     * @param vLappingArea Output vector for stereo/overlapping regions
     * @return Number of detected keypoints
     */
    int operator()(
        cv::InputArray image_in,
        cv::InputArray mask_in,
        std::vector<cv::KeyPoint>& keypoints,
        cv::OutputArray descriptors_out,
        std::vector<int>& vLappingArea);

    // --- ORBextractor-like accessors for compatibility ---
    int GetLevels();
    float GetScaleFactor();
    std::vector<float> GetScaleFactors();
    std::vector<float> GetInverseScaleFactors();
    std::vector<float> GetScaleSigmaSquares();
    std::vector<float> GetInverseScaleSigmaSquares();

    // --- Image pyramid for compatibility with ORB-SLAM3 ---
    std::vector<cv::Mat> mvImagePyramid;

private:
    // Model paths
    std::string model_path_;
    std::string delegate_path_;
    
    // ORB-SLAM3 compatible parameters
    int n_features_target_;
    float scale_factor_;
    int n_levels_;
    std::vector<float> scale_factors_;
    std::vector<float> inv_scale_factors_;
    std::vector<float> level_sigma2_;
    std::vector<float> inv_level_sigma2_;
    
    // TensorFlow Lite and EdgeTPU objects
    std::unique_ptr<tflite::FlatBufferModel> model_;
    std::unique_ptr<tflite::Interpreter> interpreter_;
    void* edgetpu_delegate_; // TfLiteDelegate* from EdgeTPU
    
    // Model input tensor dimensions
    int input_tensor_width_;
    int input_tensor_height_;
    int input_tensor_channels_;
    
    // Model output tensor indices and dimensions
    int descriptor_output_index_;
    int semi_output_index_;
    int descriptor_height_;
    int descriptor_width_;
    int descriptor_channels_;
    int semi_height_;
    int semi_width_;
    int semi_channels_;
    
    // Quantization parameters
    float descriptor_quant_scale_;
    int descriptor_quant_zero_point_;
    float semi_quant_scale_;
    int semi_quant_zero_point_;
    
    // Non-maximum suppression parameters
    float nms_radius_;
    float confidence_threshold_;
    
    /**
     * @brief Load the TFLite model and initialize the interpreter
     * 
     * @return true if successful, false otherwise
     */
    bool loadModel();
    
    /**
     * @brief Initialize scale factors for image pyramid
     */
    void initializeScaleFactors();
    
    /**
     * @brief Create image pyramid for compatibility with ORB-SLAM3
     * 
     * @param image Input image
     */
    void createImagePyramid(const cv::Mat& image);
    
    /**
     * @brief Preprocess image for model input
     * 
     * Converts image to grayscale if needed, resizes to model input dimensions,
     * and prepares for quantized input.
     * 
     * @param input_image Original input image
     * @param model_input_size Size expected by the model
     * @return Preprocessed image ready for inference
     */
    cv::Mat preprocessImage(const cv::Mat& input_image, const cv::Size& model_input_size);
    
    /**
     * @brief Run inference on the preprocessed image
     * 
     * Executes the SuperPoint model on the EdgeTPU and extracts the output tensors.
     * 
     * @param preprocessed_image Image prepared for model input
     * @param raw_keypoints Output vector for keypoint data (unused in current implementation)
     * @param raw_descriptors Output vector for descriptor data
     * @param raw_scores Output vector for keypoint score data
     */
    void runInference(
        const cv::Mat& preprocessed_image,
        std::vector<float>& raw_keypoints,
        std::vector<float>& raw_descriptors,
        std::vector<float>& raw_scores);
    
    /**
     * @brief Process model outputs to extract keypoints and descriptors
     * 
     * Converts the raw model outputs into OpenCV keypoints and descriptors,
     * applying non-maximum suppression and filtering based on confidence threshold.
     * 
     * @param original_image Original input image (for coordinate mapping)
     * @param mask_in Optional mask for filtering keypoints
     * @param raw_keypoints Raw keypoint data from model (unused in current implementation)
     * @param raw_descriptors Raw descriptor data from model
     * @param raw_scores Raw score data from model
     * @param keypoints Output vector of detected keypoints
     * @param descriptors Output matrix of descriptors
     * @param vLappingArea Output vector for stereo/overlapping regions
     */
    void postprocessResults(
        const cv::Mat& original_image,
        const cv::InputArray& mask_in,
        const std::vector<float>& raw_keypoints,
        const std::vector<float>& raw_descriptors,
        const std::vector<float>& raw_scores,
        std::vector<cv::KeyPoint>& keypoints,
        cv::Mat& descriptors,
        std::vector<int>& vLappingArea);
    
    /**
     * @brief Apply non-maximum suppression to keypoint scores
     * 
     * @param scores Score map from model output
     * @param radius Radius for NMS
     * @param threshold Confidence threshold
     * @return Vector of keypoints after NMS
     */
    std::vector<cv::KeyPoint> applyNMS(
        const std::vector<std::vector<std::vector<float>>>& scores,
        float radius,
        float threshold);
};

} // namespace ORB_SLAM3

#endif // TPU_FEATURE_EXTRACTOR_HPP

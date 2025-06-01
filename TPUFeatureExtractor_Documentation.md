# TPUFeatureExtractor Implementation Documentation

## Overview
This document provides a detailed explanation of the `TPUFeatureExtractor` class implementation, which integrates the EdgeTPU-accelerated SuperPoint model into ORB-SLAM3 as a replacement for the original `ORBextractor`.

## Key Components

### 1. Model Loading and Initialization
- The class loads a TFLite model file that contains the quantized SuperPoint model compiled for EdgeTPU
- It identifies input and output tensors and their properties (shapes, quantization parameters)
- It initializes the EdgeTPU delegate for hardware acceleration

### 2. Input Processing
- Converts input images to the format expected by the model (grayscale, resized)
- Handles quantization of input data (uint8 to int8 with zero-point shift)
- Creates an image pyramid for compatibility with ORB-SLAM3

### 3. Inference Execution
- Runs the SuperPoint model on the EdgeTPU
- Extracts and dequantizes output tensors:
  - Descriptor tensor: [1, 256, 15, 20], int8, scale=0.0023780472110956907, zero_point=-2
  - Semi (keypoints) tensor: [1, 15, 20, 65], int8, scale=0.2690383195877075, zero_point=82

### 4. Output Processing
- Extracts keypoints from the semi tensor using non-maximum suppression
- Maps keypoints back to original image coordinates
- Extracts and normalizes descriptors for each keypoint
- Applies optional mask filtering
- Limits to target number of features if specified

### 5. ORB-SLAM3 Compatibility
- Implements the same interface as `ORBextractor`
- Provides accessor methods for scale factors and other parameters
- Maintains an image pyramid for compatibility

## Error Handling
- Comprehensive error checking during model loading
- Validation of tensor shapes and types
- Graceful fallback to CPU if EdgeTPU is unavailable
- Boundary checking for tensor access
- Proper exception handling in constructor

## Performance Considerations
- Minimizes memory copies
- Uses continuous memory when possible
- Efficient descriptor extraction
- Optimized non-maximum suppression

## Usage Notes
- The model path should be relative to the ORB-SLAM3 executable's working directory
- The EdgeTPU delegate is automatically detected if available
- The implementation maintains compatibility with the ORB-SLAM3 codebase while leveraging the SuperPoint model's improved feature detection capabilities

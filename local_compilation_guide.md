# Local Compilation Guide for ORB-SLAM3 with TPU Feature Extractor

This guide provides instructions to compile the ORB-SLAM3 project, including the custom TPUFeatureExtractor integration, on your local Linux machine.

## 1. Prerequisites and Dependencies

Ensure you have a C++14 compliant compiler (e.g., g++ >= 5.0) and CMake (>= 3.5).

### 1.1. Core System Dependencies:

Install the following packages using your system's package manager (e.g., apt for Debian/Ubuntu):

```bash
sudo apt-get update
sudo apt-get install -y build-essential cmake git libglew-dev libpython3-dev python3-dev
```

### 1.2. Eigen3:

ORB-SLAM3 requires Eigen3. It is usually installed as part of `build-essential` or can be installed separately:

```bash
sudo apt-get install -y libeigen3-dev
```

### 1.3. Pangolin:

Pangolin is used for visualization and UI. It has its own set of dependencies.

**Pangolin Dependencies:**
```bash
sudo apt-get install -y libgl1-mesa-dev libglu1-mesa-dev libglew-dev libxkbcommon-dev libwayland-dev libx11-dev wayland-protocols libegl1-mesa-dev libepoxy-dev
```

**Build and Install Pangolin:**
```bash
cd /path/to/your/workspace # Choose a suitable directory
git clone https://github.com/stevenlovegrove/Pangolin.git
cd Pangolin
mkdir build
cd build
cmake .. -DPython_EXECUTABLE=/usr/bin/python3
make -j$(nproc)
sudo make install
```

### 1.4. OpenCV:

ORB-SLAM3 requires OpenCV (>= 3.0, < 4.0 is often recommended for older ORB-SLAM3 versions, but >=4.x should also work with the latest ORB-SLAM3). Install the development libraries:

```bash
sudo apt-get install -y libopencv-dev
```

### 1.5. Boost (for DBoW2):

The DBoW2 library used by ORB-SLAM3 requires Boost serialization libraries:

```bash
sudo apt-get install -y libboost-serialization-dev libboost-filesystem-dev
```

### 1.6. EdgeTPU Delegate Library (for TPUFeatureExtractor):

To compile and run the `TPUFeatureExtractor`, you will need the EdgeTPU runtime library and development headers. Follow the official Google Coral documentation for installing the EdgeTPU library (`libedgetpu.so`) and the development headers (`edgetpu.h`).

-   **Runtime Library:** Typically installed via a Debian package (`libedgetpu1-std` or `libedgetpu1-max`).
-   **Development Headers:** You might need to install `libedgetpu-dev` or manually place `edgetpu.h` in your include path.

Ensure the EdgeTPU library is in your `LD_LIBRARY_PATH` or a standard system library path.

## 2. Project Setup

1.  **Clone the ORB-SLAM3 Repository (if you haven't already):
    ```bash
    cd /path/to/your/workspace
    git clone https://github.com/UZ-SLAMLab/ORB_SLAM3.git ORB_SLAM3_with_TPU
    cd ORB_SLAM3_with_TPU
    ```

2.  **Copy Custom TPU Feature Extractor Files:**
    Place the provided `tpu_feature_extractor.hpp` and `tpu_feature_extractor.cpp` files into the ORB-SLAM3 source tree:
    *   Copy `tpu_feature_extractor.hpp` to `ORB_SLAM3_with_TPU/include/`
    *   Copy `tpu_feature_extractor.cpp` to `ORB_SLAM3_with_TPU/src/`

3.  **Modify ORB-SLAM3 Source for TPUFeatureExtractor Integration:**
    You will need to modify `ORB_SLAM3_with_TPU/include/Tracking.h` and `ORB_SLAM3_with_TPU/src/Tracking.cc` to use `TPUFeatureExtractor` instead of `ORBextractor`. The changes involve:
    *   In `Tracking.h`:
        *   Replace `#include "ORBextractor.h"` with `#include "tpu_feature_extractor.hpp"`.
        *   Change `ORBextractor*` member variables (e.g., `mpORBextractorLeft`, `mpIniORBextractor`) to `TPUFeatureExtractor*`.
    *   In `Tracking.cc`:
        *   Replace instantiations like `new ORBextractor(...)` with `new TPUFeatureExtractor("/path/to/your/superpoint_edgetpu.tflite", "", ...)`.
        *   **Important:** Ensure the path to your compiled SuperPoint EdgeTPU model (`.tflite` file) is correct in these instantiations.

## 3. Compilation Script (`build_orbslam3_locally.sh`)

Create a shell script named `build_orbslam3_locally.sh` in the root of your `ORB_SLAM3_with_TPU` directory with the following content:

```bash
#!/bin/bash

echo "Configuring and building Thirdparty/DBoW2 ..."
cd Thirdparty/DBoW2
mkdir -p build
cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
if [ $? -ne 0 ]; then echo "DBoW2 CMake configuration failed"; exit 1; fi
make -j$(nproc)
if [ $? -ne 0 ]; then echo "DBoW2 build failed"; exit 1; fi
cd ../../..

echo "Configuring and building Thirdparty/g2o ..."
cd Thirdparty/g2o
mkdir -p build
cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
if [ $? -ne 0 ]; then echo "g2o CMake configuration failed"; exit 1; fi
make -j$(nproc)
if [ $? -ne 0 ]; then echo "g2o build failed"; exit 1; fi
cd ../../..

echo "Configuring and building ORB_SLAM3 ..."
mkdir -p build
cd build
# Add -DEIGEN3_INCLUDE_DIR=/usr/include/eigen3 if CMake has trouble finding Eigen3
# Add -DPangolin_DIR=/usr/local/lib/cmake/Pangolin if CMake has trouble finding Pangolin
cmake .. -DCMAKE_BUILD_TYPE=Release \
         -DORB_SLAM3_USE_PANGOLIN_VIEWER=ON \
         -DORB_SLAM3_BUILD_EXAMPLES=ON
if [ $? -ne 0 ]; then echo "ORB_SLAM3 CMake configuration failed"; exit 1; fi
make -j$(nproc)
if [ $? -ne 0 ]; then echo "ORB_SLAM3 build failed"; exit 1; fi

echo "Build completed successfully!"
```

## 4. How to Compile

1.  Navigate to the root of your `ORB_SLAM3_with_TPU` directory.
2.  Make the script executable: `chmod +x build_orbslam3_locally.sh`
3.  Run the script: `./build_orbslam3_locally.sh`

## 5. Notes and Troubleshooting

*   **Eigen3/Pangolin Paths:** If CMake has trouble finding Eigen3 or Pangolin, you might need to specify their paths in the main ORB-SLAM3 `cmake` command within `build_orbslam3_locally.sh` (see comments in the script).
*   **EdgeTPU Library Path:** Ensure `libedgetpu.so` is in your `LD_LIBRARY_PATH` or a standard system path when running any ORB-SLAM3 executable that uses the `TPUFeatureExtractor`.
*   **Compiler Errors:** Pay close attention to compiler errors. They often indicate missing dependencies or incorrect paths.
*   **Clean Build:** If you encounter persistent issues, try a clean build by removing the `build` directories within `Thirdparty/DBoW2`, `Thirdparty/g2o`, and the main `ORB_SLAM3_with_TPU` directory before running the script again.

This guide should help you get ORB-SLAM3 with the TPU feature extractor compiled on your local system.


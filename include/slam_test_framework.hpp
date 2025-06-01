#ifndef SLAM_TEST_FRAMEWORK_HPP
#define SLAM_TEST_FRAMEWORK_HPP

#include <string>
#include <vector>
#include <functional>
#include <memory>
#include <chrono>
#include <opencv2/core.hpp>

namespace ORB_SLAM3
{

/**
 * @brief Test result structure
 */
struct TestResult {
    bool success;                  ///< Whether the test passed
    std::string name;              ///< Name of the test
    std::string description;       ///< Description of the test
    std::string message;           ///< Result message
    double execution_time_ms;      ///< Execution time in milliseconds
    std::vector<std::string> logs; ///< Detailed logs
};

/**
 * @brief Test case base class
 */
class TestCase {
public:
    /**
     * @brief Constructor
     * 
     * @param name Name of the test
     * @param description Description of the test
     */
    TestCase(const std::string& name, const std::string& description);
    
    /**
     * @brief Virtual destructor
     */
    virtual ~TestCase();
    
    /**
     * @brief Run the test
     * 
     * @return Test result
     */
    TestResult Run();
    
    /**
     * @brief Get the name of the test
     * 
     * @return Test name
     */
    std::string GetName() const;
    
    /**
     * @brief Get the description of the test
     * 
     * @return Test description
     */
    std::string GetDescription() const;
    
protected:
    /**
     * @brief Execute the test (to be implemented by derived classes)
     * 
     * @param result Test result to be filled
     */
    virtual void Execute(TestResult& result) = 0;
    
    /**
     * @brief Add a log message to the test result
     * 
     * @param result Test result to add log to
     * @param message Log message
     */
    void Log(TestResult& result, const std::string& message);
    
    /**
     * @brief Set test as passed
     * 
     * @param result Test result to update
     * @param message Success message
     */
    void SetSuccess(TestResult& result, const std::string& message);
    
    /**
     * @brief Set test as failed
     * 
     * @param result Test result to update
     * @param message Failure message
     */
    void SetFailure(TestResult& result, const std::string& message);
    
private:
    std::string name_;
    std::string description_;
};

/**
 * @brief Test suite class
 */
class TestSuite {
public:
    /**
     * @brief Constructor
     * 
     * @param name Name of the test suite
     */
    explicit TestSuite(const std::string& name);
    
    /**
     * @brief Destructor
     */
    ~TestSuite();
    
    /**
     * @brief Add a test case to the suite
     * 
     * @param test Test case to add
     */
    void AddTest(std::shared_ptr<TestCase> test);
    
    /**
     * @brief Run all tests in the suite
     * 
     * @return Vector of test results
     */
    std::vector<TestResult> RunAll();
    
    /**
     * @brief Run a specific test by name
     * 
     * @param name Name of the test to run
     * @return Test result, or empty result if test not found
     */
    TestResult RunTest(const std::string& name);
    
    /**
     * @brief Get the name of the test suite
     * 
     * @return Test suite name
     */
    std::string GetName() const;
    
    /**
     * @brief Get all test cases in the suite
     * 
     * @return Vector of test cases
     */
    std::vector<std::shared_ptr<TestCase>> GetTests() const;
    
private:
    std::string name_;
    std::vector<std::shared_ptr<TestCase>> tests_;
};

/**
 * @brief Test runner class
 */
class TestRunner {
public:
    /**
     * @brief Constructor
     */
    TestRunner();
    
    /**
     * @brief Destructor
     */
    ~TestRunner();
    
    /**
     * @brief Add a test suite to the runner
     * 
     * @param suite Test suite to add
     */
    void AddSuite(std::shared_ptr<TestSuite> suite);
    
    /**
     * @brief Run all test suites
     * 
     * @return Vector of test results
     */
    std::vector<TestResult> RunAll();
    
    /**
     * @brief Run a specific test suite by name
     * 
     * @param suite_name Name of the test suite to run
     * @return Vector of test results
     */
    std::vector<TestResult> RunSuite(const std::string& suite_name);
    
    /**
     * @brief Run a specific test by suite name and test name
     * 
     * @param suite_name Name of the test suite
     * @param test_name Name of the test
     * @return Test result, or empty result if test not found
     */
    TestResult RunTest(const std::string& suite_name, const std::string& test_name);
    
    /**
     * @brief Generate a report of test results
     * 
     * @param results Vector of test results
     * @param output_file Path to output file (empty for console output)
     */
    void GenerateReport(const std::vector<TestResult>& results, const std::string& output_file = "");
    
private:
    std::vector<std::shared_ptr<TestSuite>> suites_;
};

/**
 * @brief TPU Feature Extractor test case
 */
class TPUFeatureExtractorTest : public TestCase {
public:
    /**
     * @brief Constructor
     * 
     * @param model_path Path to the TPU model file
     * @param test_image_path Path to test image
     */
    TPUFeatureExtractorTest(const std::string& model_path, const std::string& test_image_path);
    
    /**
     * @brief Destructor
     */
    ~TPUFeatureExtractorTest() override;
    
protected:
    /**
     * @brief Execute the test
     * 
     * @param result Test result to be filled
     */
    void Execute(TestResult& result) override;
    
private:
    std::string model_path_;
    std::string test_image_path_;
};

/**
 * @brief Multi-Camera Rig test case
 */
class MultiCameraRigTest : public TestCase {
public:
    /**
     * @brief Constructor
     * 
     * @param calibration_path Path to calibration file
     * @param test_images_path Path to test images directory
     */
    MultiCameraRigTest(const std::string& calibration_path, const std::string& test_images_path);
    
    /**
     * @brief Destructor
     */
    ~MultiCameraRigTest() override;
    
protected:
    /**
     * @brief Execute the test
     * 
     * @param result Test result to be filled
     */
    void Execute(TestResult& result) override;
    
private:
    std::string calibration_path_;
    std::string test_images_path_;
};

/**
 * @brief VR Motion Model test case
 */
class VRMotionModelTest : public TestCase {
public:
    /**
     * @brief Constructor
     * 
     * @param trajectory_path Path to trajectory file
     */
    explicit VRMotionModelTest(const std::string& trajectory_path);
    
    /**
     * @brief Destructor
     */
    ~VRMotionModelTest() override;
    
protected:
    /**
     * @brief Execute the test
     * 
     * @param result Test result to be filled
     */
    void Execute(TestResult& result) override;
    
private:
    std::string trajectory_path_;
};

/**
 * @brief BNO085 Interface test case
 */
class BNO085InterfaceTest : public TestCase {
public:
    /**
     * @brief Constructor
     * 
     * @param imu_data_path Path to IMU data file
     */
    explicit BNO085InterfaceTest(const std::string& imu_data_path);
    
    /**
     * @brief Destructor
     */
    ~BNO085InterfaceTest() override;
    
protected:
    /**
     * @brief Execute the test
     * 
     * @param result Test result to be filled
     */
    void Execute(TestResult& result) override;
    
private:
    std::string imu_data_path_;
};

/**
 * @brief Zero Copy Frame Provider test case
 */
class ZeroCopyFrameProviderTest : public TestCase {
public:
    /**
     * @brief Constructor
     * 
     * @param video_path Path to test video file
     */
    explicit ZeroCopyFrameProviderTest(const std::string& video_path);
    
    /**
     * @brief Destructor
     */
    ~ZeroCopyFrameProviderTest() override;
    
protected:
    /**
     * @brief Execute the test
     * 
     * @param result Test result to be filled
     */
    void Execute(TestResult& result) override;
    
private:
    std::string video_path_;
};

/**
 * @brief Integration test case for the full SLAM system
 */
class SLAMIntegrationTest : public TestCase {
public:
    /**
     * @brief Constructor
     * 
     * @param config_path Path to SLAM configuration file
     * @param dataset_path Path to test dataset
     */
    SLAMIntegrationTest(const std::string& config_path, const std::string& dataset_path);
    
    /**
     * @brief Destructor
     */
    ~SLAMIntegrationTest() override;
    
protected:
    /**
     * @brief Execute the test
     * 
     * @param result Test result to be filled
     */
    void Execute(TestResult& result) override;
    
private:
    std::string config_path_;
    std::string dataset_path_;
};

/**
 * @brief Performance test case
 */
class PerformanceTest : public TestCase {
public:
    /**
     * @brief Constructor
     * 
     * @param component_name Name of the component to test
     * @param test_data_path Path to test data
     * @param iterations Number of iterations to run
     */
    PerformanceTest(const std::string& component_name, const std::string& test_data_path, int iterations);
    
    /**
     * @brief Destructor
     */
    ~PerformanceTest() override;
    
protected:
    /**
     * @brief Execute the test
     * 
     * @param result Test result to be filled
     */
    void Execute(TestResult& result) override;
    
private:
    std::string component_name_;
    std::string test_data_path_;
    int iterations_;
};

} // namespace ORB_SLAM3

#endif // SLAM_TEST_FRAMEWORK_HPP

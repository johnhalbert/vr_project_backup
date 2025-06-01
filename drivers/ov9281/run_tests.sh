#!/bin/bash

# OV9281 Camera Driver Test Runner
# This script compiles and runs all tests for the OV9281 camera driver

# Set up environment
echo "Setting up test environment..."
mkdir -p build
cd build

# Install dependencies if needed
if [ ! -f "dependencies_installed" ]; then
    echo "Installing test dependencies..."
    apt-get update
    apt-get install -y cmake g++ libgtest-dev libgmock-dev
    touch dependencies_installed
fi

# Compile mock libraries
echo "Compiling mock libraries..."
mkdir -p mocks
cat > mocks/mock_kernel.h << 'EOF'
#ifndef MOCK_KERNEL_H
#define MOCK_KERNEL_H

#include <stdint.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

/* Basic types */
typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;
typedef int8_t s8;
typedef int16_t s16;
typedef int32_t s32;
typedef int64_t s64;

/* Error codes */
#define EINVAL 22
#define EIO 5
#define ENOMEM 12
#define ENODEV 19
#define ETIMEDOUT 110

/* Bit operations */
#define BIT(n) (1UL << (n))

/* Memory allocation */
#define GFP_KERNEL 0
void* kmalloc(size_t size, int flags) { return malloc(size); }
void kfree(void* ptr) { free(ptr); }

/* Device structure */
struct device {
    void* driver_data;
};

void* dev_get_drvdata(struct device* dev) { return dev->driver_data; }
void dev_set_drvdata(struct device* dev, void* data) { dev->driver_data = data; }

/* Logging */
#define dev_err(dev, fmt, ...) printf("ERROR: " fmt "\n", ##__VA_ARGS__)
#define dev_warn(dev, fmt, ...) printf("WARNING: " fmt "\n", ##__VA_ARGS__)
#define dev_info(dev, fmt, ...) printf("INFO: " fmt "\n", ##__VA_ARGS__)
#define pr_err(fmt, ...) printf("ERROR: " fmt "\n", ##__VA_ARGS__)

/* Mutex */
struct mutex {
    int locked;
};

void mutex_init(struct mutex* mutex) { mutex->locked = 0; }
void mutex_lock(struct mutex* mutex) { mutex->locked = 1; }
void mutex_unlock(struct mutex* mutex) { mutex->locked = 0; }
void mutex_destroy(struct mutex* mutex) { /* No-op in test */ }

/* GPIO */
int gpio_set_value(unsigned gpio, int value) { return 0; }
int gpio_get_value(unsigned gpio) { return 0; }

/* Time functions */
typedef uint64_t ktime_t;
ktime_t ktime_get(void) { return 0; }
void msleep(unsigned int msecs) { /* No-op in test */ }
void udelay(unsigned int usecs) { /* No-op in test */ }

/* Clocks */
struct clk {
    int dummy;
};

int clk_prepare_enable(struct clk* clk) { return 0; }
void clk_disable_unprepare(struct clk* clk) { /* No-op in test */ }
unsigned long clk_get_rate(struct clk* clk) { return 24000000; }

/* Regulators */
struct regulator {
    int dummy;
};

int regulator_enable(struct regulator* reg) { return 0; }
int regulator_disable(struct regulator* reg) { return 0; }

/* Device tree */
struct device_node {
    int dummy;
};

int of_get_named_gpio(struct device_node* np, const char* propname, int index) { return 42; }

/* PM Runtime */
int pm_runtime_enable(struct device* dev) { return 0; }
void pm_runtime_disable(struct device* dev) { /* No-op in test */ }

/* Debugfs */
struct dentry {
    int dummy;
};

struct dentry* debugfs_create_dir(const char* name, struct dentry* parent) {
    return (struct dentry*)malloc(sizeof(struct dentry));
}

void debugfs_remove_recursive(struct dentry* dentry) {
    free(dentry);
}

#endif /* MOCK_KERNEL_H */
EOF

cat > mocks/mock_v4l2.h << 'EOF'
#ifndef MOCK_V4L2_H
#define MOCK_V4L2_H

#include <gmock/gmock.h>
#include "mock_kernel.h"

/* I2C structures */
struct i2c_client {
    struct device dev;
    int addr;
    struct i2c_adapter* adapter;
};

struct i2c_adapter {
    int dummy;
};

struct i2c_msg {
    u16 addr;
    u16 flags;
    u16 len;
    u8* buf;
};

struct i2c_device_id {
    char name[20];
    unsigned long driver_data;
};

#define I2C_M_RD 0x01

/* V4L2 structures */
struct v4l2_subdev {
    struct device* dev;
    const struct v4l2_subdev_ops* ops;
    struct v4l2_ctrl_handler* ctrl_handler;
    struct media_entity entity;
};

struct v4l2_subdev_ops {
    const struct v4l2_subdev_core_ops* core;
    const struct v4l2_subdev_video_ops* video;
    const struct v4l2_subdev_pad_ops* pad;
};

struct v4l2_subdev_core_ops {
    int (*s_power)(struct v4l2_subdev* sd, int on);
};

struct v4l2_subdev_video_ops {
    int (*g_frame_interval)(struct v4l2_subdev* sd, struct v4l2_subdev_frame_interval* fi);
    int (*s_frame_interval)(struct v4l2_subdev* sd, struct v4l2_subdev_frame_interval* fi);
    int (*s_stream)(struct v4l2_subdev* sd, int enable);
};

struct v4l2_subdev_pad_ops {
    int (*enum_mbus_code)(struct v4l2_subdev* sd, struct v4l2_subdev_pad_config* cfg, struct v4l2_subdev_mbus_code_enum* code);
    int (*enum_frame_size)(struct v4l2_subdev* sd, struct v4l2_subdev_pad_config* cfg, struct v4l2_subdev_frame_size_enum* fse);
    int (*get_fmt)(struct v4l2_subdev* sd, struct v4l2_subdev_pad_config* cfg, struct v4l2_subdev_format* format);
    int (*set_fmt)(struct v4l2_subdev* sd, struct v4l2_subdev_pad_config* cfg, struct v4l2_subdev_format* format);
};

struct v4l2_subdev_frame_interval {
    struct v4l2_fract interval;
};

struct v4l2_fract {
    u32 numerator;
    u32 denominator;
};

struct v4l2_subdev_pad_config {
    struct v4l2_mbus_framefmt* try_fmt;
};

struct v4l2_subdev_mbus_code_enum {
    u32 index;
    u32 code;
};

struct v4l2_subdev_frame_size_enum {
    u32 index;
    u32 code;
    u32 min_width;
    u32 max_width;
    u32 min_height;
    u32 max_height;
};

struct v4l2_subdev_format {
    u32 which;
    struct v4l2_mbus_framefmt format;
};

struct v4l2_mbus_framefmt {
    u32 width;
    u32 height;
    u32 code;
    u32 field;
    u32 colorspace;
};

struct media_entity {
    int dummy;
};

struct media_pad {
    u32 flags;
};

struct v4l2_ctrl_handler {
    int error;
};

struct v4l2_ctrl {
    u32 id;
    s32 val;
};

struct v4l2_ctrl_ops {
    int (*s_ctrl)(struct v4l2_ctrl* ctrl);
};

struct v4l2_ctrl_config {
    u32 id;
    const char* name;
    u32 type;
    s32 min;
    s32 max;
    u32 step;
    s32 def;
    const struct v4l2_ctrl_ops* ops;
};

/* V4L2 constants */
#define V4L2_FIELD_NONE 0
#define V4L2_COLORSPACE_RAW 0
#define V4L2_SUBDEV_FORMAT_TRY 0
#define V4L2_SUBDEV_FORMAT_ACTIVE 1
#define V4L2_CID_EXPOSURE 0x00980911
#define V4L2_CID_GAIN 0x00980913
#define V4L2_CID_HFLIP 0x00980914
#define V4L2_CID_VFLIP 0x00980915
#define V4L2_CID_TEST_PATTERN 0x00980916
#define V4L2_CID_PIXEL_RATE 0x00980917
#define V4L2_CID_LINK_FREQ 0x00980918
#define V4L2_CID_PRIVATE_BASE 0x00980919
#define V4L2_CTRL_TYPE_INTEGER 0
#define V4L2_CTRL_TYPE_BOOLEAN 1
#define V4L2_CTRL_TYPE_MENU 2
#define MEDIA_PAD_FL_SOURCE 0x00000001

/* Media bus formats */
#define MEDIA_BUS_FMT_Y10_1X10 0x200a

/* Mock I2C class */
class MockI2C {
public:
    MOCK_METHOD(int, i2c_transfer, (struct i2c_client* client, struct i2c_msg* msgs, int num));
};

/* Mock DMA class */
class MockDMA {
public:
    MOCK_METHOD(void*, dma_alloc_coherent, (struct device* dev, size_t size, dma_addr_t* dma_handle, gfp_t gfp));
    MOCK_METHOD(void, dma_free_coherent, (struct device* dev, size_t size, void* vaddr, dma_addr_t dma_handle));
};

/* Helper functions */
struct i2c_client* mock_i2c_client_create(void) {
    struct i2c_client* client = (struct i2c_client*)malloc(sizeof(struct i2c_client));
    client->addr = 0x42;
    client->adapter = (struct i2c_adapter*)malloc(sizeof(struct i2c_adapter));
    client->dev.driver_data = new MockI2C();
    return client;
}

void mock_i2c_client_destroy(struct i2c_client* client) {
    delete static_cast<MockI2C*>(client->dev.driver_data);
    free(client->adapter);
    free(client);
}

/* V4L2 functions */
void v4l2_i2c_subdev_init(struct v4l2_subdev* sd, struct i2c_client* client, const struct v4l2_subdev_ops* ops) {
    sd->dev = &client->dev;
    sd->ops = ops;
}

int v4l2_ctrl_handler_init(struct v4l2_ctrl_handler* hdl, int nr_of_controls) {
    hdl->error = 0;
    return 0;
}

void v4l2_ctrl_handler_free(struct v4l2_ctrl_handler* hdl) {
    /* No-op in test */
}

struct v4l2_ctrl* v4l2_ctrl_new_std(struct v4l2_ctrl_handler* hdl, const struct v4l2_ctrl_ops* ops,
                                  u32 id, s32 min, s32 max, u32 step, s32 def) {
    struct v4l2_ctrl* ctrl = (struct v4l2_ctrl*)malloc(sizeof(struct v4l2_ctrl));
    ctrl->id = id;
    ctrl->val = def;
    return ctrl;
}

struct v4l2_ctrl* v4l2_ctrl_new_std_menu_items(struct v4l2_ctrl_handler* hdl, const struct v4l2_ctrl_ops* ops,
                                            u32 id, s32 max, s32 mask, s32 def, const char* const* qmenu) {
    struct v4l2_ctrl* ctrl = (struct v4l2_ctrl*)malloc(sizeof(struct v4l2_ctrl));
    ctrl->id = id;
    ctrl->val = def;
    return ctrl;
}

struct v4l2_ctrl* v4l2_ctrl_new_int_menu(struct v4l2_ctrl_handler* hdl, const struct v4l2_ctrl_ops* ops,
                                      u32 id, s32 max, s32 def, const s64* menu_int) {
    struct v4l2_ctrl* ctrl = (struct v4l2_ctrl*)malloc(sizeof(struct v4l2_ctrl));
    ctrl->id = id;
    ctrl->val = def;
    return ctrl;
}

struct v4l2_ctrl* v4l2_ctrl_new_custom(struct v4l2_ctrl_handler* hdl, const struct v4l2_ctrl_config* cfg, void* priv) {
    struct v4l2_ctrl* ctrl = (struct v4l2_ctrl*)malloc(sizeof(struct v4l2_ctrl));
    ctrl->id = cfg->id;
    ctrl->val = cfg->def;
    return ctrl;
}

int media_entity_pads_init(struct media_entity* entity, u16 num_pads, struct media_pad* pads) {
    return 0;
}

void media_entity_cleanup(struct media_entity* entity) {
    /* No-op in test */
}

int v4l2_async_register_subdev(struct v4l2_subdev* sd) {
    return 0;
}

void v4l2_async_unregister_subdev(struct v4l2_subdev* sd) {
    /* No-op in test */
}

/* DMA types */
typedef u64 dma_addr_t;
typedef u32 gfp_t;

#endif /* MOCK_V4L2_H */
EOF

cat > mocks/mock_dma.h << 'EOF'
#ifndef MOCK_DMA_H
#define MOCK_DMA_H

#include "mock_kernel.h"

/* DMA types */
typedef u64 dma_addr_t;
typedef u32 gfp_t;

#endif /* MOCK_DMA_H */
EOF

# Compile tests
echo "Compiling tests..."
mkdir -p bin

# Compile unit tests
g++ -std=c++11 -I. -Imocks ../tests/unit/ov9281_unit_tests.cpp -o bin/ov9281_unit_tests -lgtest -lgmock -lpthread

# Compile simulation tests
g++ -std=c++11 -I. -Imocks ../tests/simulation/ov9281_simulation_tests.cpp -o bin/ov9281_simulation_tests -lgtest -lgmock -lpthread

# Compile performance tests
g++ -std=c++11 -I. -Imocks ../tests/performance/ov9281_performance_tests.cpp -o bin/ov9281_performance_tests -lgtest -lgmock -lpthread

# Run tests
echo "Running unit tests..."
./bin/ov9281_unit_tests

echo "Running simulation tests..."
./bin/ov9281_simulation_tests

echo "Running performance tests..."
./bin/ov9281_performance_tests

echo "All tests completed!"

#!/bin/bash

# BNO085 IMU Driver Test Runner
# This script compiles and runs all tests for the BNO085 IMU driver

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

/* Completion */
struct completion {
    int done;
};

void init_completion(struct completion* comp) { comp->done = 0; }
void reinit_completion(struct completion* comp) { comp->done = 0; }
void complete(struct completion* comp) { comp->done = 1; }
int wait_for_completion_timeout(struct completion* comp, unsigned long timeout) {
    comp->done = 1; /* Simulate completion */
    return 1;
}

/* Work queue */
struct work_struct {
    void (*func)(struct work_struct*);
};

#define INIT_WORK(work, _func) ((work)->func = (_func))
void schedule_work(struct work_struct* work) {
    if (work->func) {
        work->func(work);
    }
}

/* Time functions */
typedef uint64_t ktime_t;
ktime_t ktime_get(void) { return 0; }
void msleep(unsigned int msecs) { /* No-op in test */ }

/* Device creation for tests */
struct device* mock_device_create(void) {
    struct device* dev = (struct device*)malloc(sizeof(struct device));
    dev->driver_data = NULL;
    return dev;
}

void mock_device_destroy(struct device* dev) {
    free(dev);
}

#endif /* MOCK_KERNEL_H */
EOF

cat > mocks/mock_i2c.h << 'EOF'
#ifndef MOCK_I2C_H
#define MOCK_I2C_H

#include <gmock/gmock.h>
#include "mock_kernel.h"

/* I2C structures */
struct i2c_client {
    struct device dev;
    int irq;
};

struct i2c_device_id {
    char name[20];
    unsigned long driver_data;
};

/* Mock I2C class */
class MockI2C {
public:
    MOCK_METHOD(int, i2c_smbus_read_i2c_block_data, (u8 reg, int len, u8* data));
    MOCK_METHOD(int, i2c_smbus_write_i2c_block_data, (u8 reg, int len, const u8* data));
};

/* Helper functions */
struct i2c_client* to_i2c_client(struct device* dev) {
    return reinterpret_cast<struct i2c_client*>(
        reinterpret_cast<char*>(dev) - offsetof(struct i2c_client, dev)
    );
}

struct device* mock_i2c_device_create(void) {
    struct i2c_client* client = new struct i2c_client();
    client->irq = 42;
    client->dev.driver_data = new MockI2C();
    return &client->dev;
}

void mock_i2c_device_destroy(struct device* dev) {
    struct i2c_client* client = to_i2c_client(dev);
    delete static_cast<MockI2C*>(client->dev.driver_data);
    delete client;
}

#endif /* MOCK_I2C_H */
EOF

cat > mocks/mock_spi.h << 'EOF'
#ifndef MOCK_SPI_H
#define MOCK_SPI_H

#include <gmock/gmock.h>
#include "mock_kernel.h"

/* SPI structures */
struct spi_device {
    struct device dev;
    int irq;
    u8 mode;
    u8 bits_per_word;
};

struct spi_transfer {
    const void* tx_buf;
    void* rx_buf;
    size_t len;
    struct list_head transfer_list;
};

struct spi_message {
    struct list_head transfers;
    int status;
    unsigned frame_length;
    unsigned actual_length;
};

struct list_head {
    struct list_head* next;
    struct list_head* prev;
};

/* List operations */
#define list_first_entry(ptr, type, member) \
    ((type*)((char*)(ptr)->next - offsetof(type, member)))

#define list_next_entry(pos, member) \
    list_entry((pos)->member.next, typeof(*(pos)), member)

#define list_entry(ptr, type, member) \
    ((type*)((char*)(ptr) - offsetof(type, member)))

void spi_message_init(struct spi_message* m) {
    m->transfers.next = &m->transfers;
    m->transfers.prev = &m->transfers;
    m->status = 0;
    m->frame_length = 0;
    m->actual_length = 0;
}

void spi_message_add_tail(struct spi_transfer* t, struct spi_message* m) {
    t->transfer_list.next = &m->transfers;
    t->transfer_list.prev = m->transfers.prev;
    m->transfers.prev->next = &t->transfer_list;
    m->transfers.prev = &t->transfer_list;
    m->frame_length += t->len;
}

int spi_setup(struct spi_device* spi) {
    return 0;
}

/* Mock SPI class */
class MockSPI {
public:
    MOCK_METHOD(int, spi_sync, (struct spi_device* spi, struct spi_message* message));
};

/* Helper functions */
struct spi_device* to_spi_device(struct device* dev) {
    return reinterpret_cast<struct spi_device*>(
        reinterpret_cast<char*>(dev) - offsetof(struct spi_device, dev)
    );
}

struct device* mock_spi_device_create(void) {
    struct spi_device* spi = new struct spi_device();
    spi->irq = 43;
    spi->mode = 0;
    spi->bits_per_word = 8;
    spi->dev.driver_data = new MockSPI();
    return &spi->dev;
}

void mock_spi_device_destroy(struct device* dev) {
    struct spi_device* spi = to_spi_device(dev);
    delete static_cast<MockSPI*>(spi->dev.driver_data);
    delete spi;
}

#endif /* MOCK_SPI_H */
EOF

cat > mocks/mock_iio.h << 'EOF'
#ifndef MOCK_IIO_H
#define MOCK_IIO_H

#include "mock_kernel.h"

/* IIO structures */
struct iio_dev {
    const char* name;
    struct device* dev;
    const struct iio_info* info;
    const struct iio_chan_spec* channels;
    int num_channels;
    int modes;
    struct iio_trigger* trig;
    void* priv;
    unsigned long* active_scan_mask;
    struct debugfs_regset32* debugfs_reg32;
    struct dentry* debugfs_dentry;
};

struct iio_chan_spec {
    enum iio_chan_type type;
    int channel;
    int channel2;
    unsigned long info_mask_separate;
    unsigned long info_mask_shared_by_type;
    int scan_index;
    struct {
        char sign;
        u8 realbits;
        u8 storagebits;
        u8 shift;
        u8 repeat;
        enum iio_endian endianness;
    } scan_type;
    long address;
};

struct iio_info {
    int (*read_raw)(struct iio_dev* indio_dev,
                   struct iio_chan_spec const* chan,
                   int* val, int* val2, long mask);
    int (*write_raw)(struct iio_dev* indio_dev,
                    struct iio_chan_spec const* chan,
                    int val, int val2, long mask);
    int (*debugfs_reg_access)(struct iio_dev* indio_dev,
                             unsigned reg, unsigned writeval,
                             unsigned* readval);
    const struct attribute_group* attrs;
};

struct iio_trigger {
    const char* name;
    struct device dev;
};

struct iio_poll_func {
    struct iio_dev* indio_dev;
};

struct dentry {
    int dummy;
};

struct debugfs_regset32 {
    int dummy;
};

struct attribute_group {
    const char* name;
    struct attribute** attrs;
};

struct attribute {
    const char* name;
    unsigned short mode;
};

/* IIO constants */
enum iio_chan_type {
    IIO_VOLTAGE,
    IIO_CURRENT,
    IIO_POWER,
    IIO_ACCEL,
    IIO_ANGL_VEL,
    IIO_MAGN,
    IIO_LIGHT,
    IIO_INTENSITY,
    IIO_PROXIMITY,
    IIO_TEMP,
    IIO_INCLI,
    IIO_ROT,
    IIO_ANGL,
    IIO_TIMESTAMP,
    IIO_CAPACITANCE,
    IIO_ALTVOLTAGE,
    IIO_CCT,
    IIO_PRESSURE,
    IIO_HUMIDITYRELATIVE,
    IIO_ACTIVITY,
    IIO_STEPS,
    IIO_ENERGY,
    IIO_DISTANCE,
    IIO_VELOCITY,
    IIO_CONCENTRATION,
    IIO_RESISTANCE,
    IIO_PH,
    IIO_UVINDEX,
    IIO_ELECTRICALCONDUCTIVITY,
    IIO_COUNT,
    IIO_INDEX,
    IIO_GRAVITY,
    IIO_POSITIONRELATIVE,
    IIO_PHASE,
    IIO_MASSCONCENTRATION,
    IIO_CHAN_TYPE_MAX,
};

enum iio_modifier {
    IIO_NO_MOD,
    IIO_MOD_X,
    IIO_MOD_Y,
    IIO_MOD_Z,
    IIO_MOD_X_AND_Y,
    IIO_MOD_X_AND_Z,
    IIO_MOD_Y_AND_Z,
    IIO_MOD_X_AND_Y_AND_Z,
    IIO_MOD_X_OR_Y,
    IIO_MOD_X_OR_Z,
    IIO_MOD_Y_OR_Z,
    IIO_MOD_X_OR_Y_OR_Z,
    IIO_MOD_LIGHT_BOTH,
    IIO_MOD_LIGHT_IR,
    IIO_MOD_ROOT_SUM_SQUARED_X_Y,
    IIO_MOD_SUM_SQUARED_X_Y_Z,
    IIO_MOD_LIGHT_CLEAR,
    IIO_MOD_LIGHT_RED,
    IIO_MOD_LIGHT_GREEN,
    IIO_MOD_LIGHT_BLUE,
    IIO_MOD_QUATERNION_W,
    IIO_MOD_QUATERNION_X,
    IIO_MOD_QUATERNION_Y,
    IIO_MOD_QUATERNION_Z,
    IIO_MOD_TEMP_AMBIENT,
    IIO_MOD_TEMP_OBJECT,
    IIO_MOD_NORTH,
    IIO_MOD_EAST,
    IIO_MOD_SOUTH,
    IIO_MOD_WEST,
    IIO_MOD_RUNNING,
    IIO_MOD_JOGGING,
    IIO_MOD_WALKING,
    IIO_MOD_STILL,
    IIO_MOD_ROOT_SUM_SQUARED_X_Y_Z,
    IIO_MOD_I,
    IIO_MOD_Q,
    IIO_MOD_CO2,
    IIO_MOD_VOC,
    IIO_MOD_LIGHT_UV,
    IIO_MOD_LIGHT_DUV,
    IIO_MOD_PM1,
    IIO_MOD_PM2P5,
    IIO_MOD_PM4,
    IIO_MOD_PM10,
    IIO_MOD_ETHANOL,
    IIO_MOD_H2,
    IIO_MOD_O2,
    IIO_MOD_MAX,
};

enum iio_endian {
    IIO_CPU,
    IIO_BE,
    IIO_LE,
};

/* IIO constants */
#define IIO_VAL_INT 1
#define IIO_VAL_INT_PLUS_MICRO 2
#define IIO_VAL_INT_PLUS_NANO 3
#define IIO_VAL_INT_PLUS_MICRO_DB 4
#define IIO_VAL_FRACTIONAL 10
#define IIO_VAL_FRACTIONAL_LOG2 11

#define IIO_CHAN_INFO_RAW BIT(0)
#define IIO_CHAN_INFO_PROCESSED BIT(1)
#define IIO_CHAN_INFO_SCALE BIT(2)
#define IIO_CHAN_INFO_OFFSET BIT(3)
#define IIO_CHAN_INFO_CALIBSCALE BIT(4)
#define IIO_CHAN_INFO_CALIBBIAS BIT(5)
#define IIO_CHAN_INFO_PEAK BIT(6)
#define IIO_CHAN_INFO_PEAK_SCALE BIT(7)
#define IIO_CHAN_INFO_QUADRATURE_CORRECTION_RAW BIT(8)
#define IIO_CHAN_INFO_AVERAGE_RAW BIT(9)
#define IIO_CHAN_INFO_LOW_PASS_FILTER_3DB_FREQUENCY BIT(10)
#define IIO_CHAN_INFO_HIGH_PASS_FILTER_3DB_FREQUENCY BIT(11)
#define IIO_CHAN_INFO_SAMP_FREQ BIT(12)
#define IIO_CHAN_INFO_FREQUENCY BIT(13)
#define IIO_CHAN_INFO_PHASE BIT(14)
#define IIO_CHAN_INFO_HARDWAREGAIN BIT(15)
#define IIO_CHAN_INFO_HYSTERESIS BIT(16)
#define IIO_CHAN_INFO_HYSTERESIS_RELATIVE BIT(17)
#define IIO_CHAN_INFO_INT_TIME BIT(18)
#define IIO_CHAN_INFO_ENABLE BIT(19)
#define IIO_CHAN_INFO_CALIBHEIGHT BIT(20)
#define IIO_CHAN_INFO_CALIBWEIGHT BIT(21)
#define IIO_CHAN_INFO_DEBOUNCE_COUNT BIT(22)
#define IIO_CHAN_INFO_DEBOUNCE_TIME BIT(23)
#define IIO_CHAN_INFO_CALIBEMISSIVITY BIT(24)
#define IIO_CHAN_INFO_OVERSAMPLING_RATIO BIT(25)
#define IIO_CHAN_INFO_THERMOCOUPLE_TYPE BIT(26)

#define INDIO_DIRECT_MODE BIT(0)
#define INDIO_BUFFER_TRIGGERED BIT(1)
#define INDIO_BUFFER_SOFTWARE BIT(2)
#define INDIO_BUFFER_HARDWARE BIT(3)
#define INDIO_EVENT_TRIGGERED BIT(4)
#define INDIO_HARDWARE_TRIGGERED BIT(5)

#define IIO_CHAN_SOFT_TIMESTAMP 0

/* IIO functions */
struct iio_dev* devm_iio_device_alloc(struct device* dev, size_t size) {
    struct iio_dev* indio_dev = (struct iio_dev*)malloc(sizeof(struct iio_dev) + size);
    indio_dev->dev = dev;
    indio_dev->priv = (char*)indio_dev + sizeof(struct iio_dev);
    indio_dev->active_scan_mask = (unsigned long*)calloc(1, sizeof(unsigned long));
    indio_dev->debugfs_dentry = (struct dentry*)calloc(1, sizeof(struct dentry));
    return indio_dev;
}

void* iio_priv(struct iio_dev* indio_dev) {
    return indio_dev->priv;
}

int devm_iio_device_register(struct device* dev, struct iio_dev* indio_dev) {
    return 0;
}

int devm_iio_triggered_buffer_setup(struct device* dev,
                                  struct iio_dev* indio_dev,
                                  irqreturn_t (*h)(int irq, void* p),
                                  irqreturn_t (*thread)(int irq, void* p),
                                  const struct iio_buffer_setup_ops* setup_ops) {
    return 0;
}

struct iio_trigger* devm_iio_trigger_alloc(struct device* dev, const char* fmt, ...) {
    struct iio_trigger* trig = (struct iio_trigger*)malloc(sizeof(struct iio_trigger));
    trig->name = strdup("test-trigger");
    return trig;
}

void iio_trigger_set_drvdata(struct iio_trigger* trig, void* data) {
    trig->dev.driver_data = data;
}

int devm_iio_trigger_register(struct device* dev, struct iio_trigger* trig) {
    return 0;
}

void iio_trigger_notify_done(struct iio_trigger* trig) {
    /* No-op in test */
}

int iio_push_to_buffers_with_timestamp(struct iio_dev* indio_dev, void* data, s64 timestamp) {
    return 0;
}

s64 iio_get_time_ns(struct iio_dev* indio_dev) {
    return 0;
}

void iio_trigger_poll(struct iio_trigger* trig) {
    /* No-op in test */
}

struct dentry* debugfs_create_dir(const char* name, struct dentry* parent) {
    return (struct dentry*)calloc(1, sizeof(struct dentry));
}

typedef int irqreturn_t;
#define IRQ_HANDLED 1
#define IRQ_NONE 0
#define IRQF_TRIGGER_RISING 0x00000001

int devm_request_irq(struct device* dev, unsigned int irq,
                   irqreturn_t (*handler)(int, void*),
                   unsigned long flags, const char* name, void* dev_id) {
    return 0;
}

#endif /* MOCK_IIO_H */
EOF

# Compile tests
echo "Compiling tests..."
mkdir -p bin

# Compile unit tests
g++ -std=c++11 -I. -Imocks ../tests/unit/bno085_unit_tests.cpp -o bin/bno085_unit_tests -lgtest -lgmock -lpthread

# Compile integration tests
g++ -std=c++11 -I. -Imocks ../tests/integration/bno085_integration_tests.cpp -o bin/bno085_integration_tests -lgtest -lgmock -lpthread

# Compile simulation tests
g++ -std=c++11 -I. -Imocks ../tests/simulation/bno085_simulation_tests.cpp -o bin/bno085_simulation_tests -lgtest -lgmock -lpthread

# Compile performance tests
g++ -std=c++11 -I. -Imocks ../tests/performance/bno085_performance_tests.cpp -o bin/bno085_performance_tests -lgtest -lgmock -lpthread

# Run tests
echo "Running unit tests..."
./bin/bno085_unit_tests

echo "Running integration tests..."
./bin/bno085_integration_tests

echo "Running simulation tests..."
./bin/bno085_simulation_tests

echo "Running performance tests..."
./bin/bno085_performance_tests

echo "All tests completed!"

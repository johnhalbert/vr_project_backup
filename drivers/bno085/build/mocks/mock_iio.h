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

/* SPDX-License-Identifier: GPL-2.0 */
/*
 * BNO085 IMU driver
 *
 * Copyright (C) 2025 VR Headset Project
 */

#ifndef _BNO085_CORE_H_
#define _BNO085_CORE_H_

#include <linux/types.h>
#include <linux/iio/iio.h>
#include <linux/regmap.h>

/* BNO085 Register Map */
#define BNO085_REG_CHIP_ID          0x00
#define BNO085_REG_RESET            0x01
#define BNO085_REG_STATUS           0x02
#define BNO085_REG_COMMAND          0x03
#define BNO085_REG_RESPONSE         0x04
#define BNO085_REG_DATA_BUFFER      0x05
#define BNO085_REG_FEAT_STATUS      0x06
#define BNO085_REG_FEAT_CTRL        0x07
#define BNO085_REG_CALIB_STATUS     0x08
#define BNO085_REG_INT_STATUS       0x09
#define BNO085_REG_INT_ENABLE       0x0A
#define BNO085_REG_TEMP             0x0B
#define BNO085_REG_ACCEL_X          0x0C
#define BNO085_REG_ACCEL_Y          0x0E
#define BNO085_REG_ACCEL_Z          0x10
#define BNO085_REG_GYRO_X           0x12
#define BNO085_REG_GYRO_Y           0x14
#define BNO085_REG_GYRO_Z           0x16
#define BNO085_REG_MAG_X            0x18
#define BNO085_REG_MAG_Y            0x1A
#define BNO085_REG_MAG_Z            0x1C
#define BNO085_REG_QUAT_W           0x1E
#define BNO085_REG_QUAT_X           0x20
#define BNO085_REG_QUAT_Y           0x22
#define BNO085_REG_QUAT_Z           0x24
#define BNO085_REG_TIMESTAMP        0x26

/* BNO085 Constants */
#define BNO085_CHIP_ID              0x83
#define BNO085_RESET_COMMAND        0x01
#define BNO085_MAX_TRANSFER_SIZE    32
#define BNO085_FIFO_SIZE            1024

/* BNO085 Status Register Bits */
#define BNO085_STATUS_IDLE          0x00
#define BNO085_STATUS_DATA_READY    0x01
#define BNO085_STATUS_CALIB_CHANGE  0x02
#define BNO085_STATUS_ERROR         0x04
#define BNO085_STATUS_RESET_DONE    0x08
#define BNO085_STATUS_OVERFLOW      0x10

/* BNO085 Interrupt Enable/Status Bits */
#define BNO085_INT_ACCEL            0x01
#define BNO085_INT_GYRO             0x02
#define BNO085_INT_MAG              0x04
#define BNO085_INT_QUAT             0x08
#define BNO085_INT_TEMP             0x10
#define BNO085_INT_ERROR            0x20
#define BNO085_INT_CALIB            0x40
#define BNO085_INT_FIFO             0x80

/* BNO085 Operation Modes */
enum bno085_operation_mode {
    BNO085_MODE_CONFIG              = 0x00,
    BNO085_MODE_IMU                 = 0x01,
    BNO085_MODE_NDOF                = 0x02,
    BNO085_MODE_NDOF_FMC_OFF        = 0x03,
    BNO085_MODE_GYRO_ONLY           = 0x04,
    BNO085_MODE_ACCEL_ONLY          = 0x05,
    BNO085_MODE_MAG_ONLY            = 0x06,
    BNO085_MODE_AR_VR_STABILIZED    = 0x07,
    BNO085_MODE_AR_VR_PREDICTIVE    = 0x08,
};

/* BNO085 Device State */
enum bno085_state {
    BNO085_STATE_DISABLED           = 0,
    BNO085_STATE_INITIALIZING       = 1,
    BNO085_STATE_INITIALIZED        = 2,
    BNO085_STATE_RUNNING            = 3,
    BNO085_STATE_ERROR              = 4,
};

/* BNO085 Sensor Features */
enum bno085_sensor_feature {
    BNO085_FEATURE_ACCELEROMETER    = BIT(0),
    BNO085_FEATURE_GYROSCOPE        = BIT(1),
    BNO085_FEATURE_MAGNETOMETER     = BIT(2),
    BNO085_FEATURE_ORIENTATION      = BIT(3),
    BNO085_FEATURE_ROTATION_VECTOR  = BIT(4),
    BNO085_FEATURE_GAME_ROTATION    = BIT(5),
    BNO085_FEATURE_LINEAR_ACCEL     = BIT(6),
    BNO085_FEATURE_GRAVITY          = BIT(7),
    BNO085_FEATURE_TEMPERATURE      = BIT(8),
};

/* BNO085 Transport Interface */
struct bno085_transport {
    int (*read)(struct device *dev, u8 reg, u8 *data, int len);
    int (*write)(struct device *dev, u8 reg, const u8 *data, int len);
    int (*read_fifo)(struct device *dev, u8 *data, int len);
};

/* BNO085 Device Structure */
struct bno085_device {
    struct device *dev;
    struct iio_dev *indio_dev;
    struct regmap *regmap;
    struct mutex lock;
    struct bno085_transport transport;
    
    /* Device state */
    enum bno085_state state;
    enum bno085_operation_mode mode;
    u32 enabled_features;
    u32 sampling_frequency;
    
    /* Calibration data */
    bool calibrated;
    u8 accel_calib[6];
    u8 gyro_calib[6];
    u8 mag_calib[6];
    
    /* Interrupt handling */
    int irq;
    bool irq_enabled;
    struct work_struct irq_work;
    
    /* IIO buffer and trigger */
    struct iio_trigger *trig;
    bool buffer_enabled;
    
    /* Data buffers */
    s16 accel_data[3];
    s16 gyro_data[3];
    s16 mag_data[3];
    s16 quaternion_data[4];
    s16 temperature_data;
    
    /* Timestamps */
    s64 timestamp;
    ktime_t last_sample_time;
    
    /* Debug */
    struct dentry *debugfs_root;
};

/* Core driver functions */
int bno085_core_probe(struct device *dev, struct bno085_transport *transport, int irq);
int bno085_core_remove(struct device *dev);
int bno085_core_init(struct bno085_device *dev);
int bno085_set_mode(struct bno085_device *dev, enum bno085_operation_mode mode);
int bno085_set_feature(struct bno085_device *dev, enum bno085_sensor_feature feature, bool enable);
int bno085_set_sampling_frequency(struct bno085_device *dev, u32 frequency);
int bno085_read_data(struct bno085_device *dev);
int bno085_reset(struct bno085_device *dev);
int bno085_suspend(struct device *dev);
int bno085_resume(struct device *dev);
int bno085_update_calibration(struct bno085_device *dev);

/* Transport layer registration */
int bno085_i2c_probe(struct i2c_client *client, const struct i2c_device_id *id);
int bno085_i2c_remove(struct i2c_client *client);
int bno085_spi_probe(struct spi_device *spi);
int bno085_spi_remove(struct spi_device *spi);

/* Buffer and trigger functions */
int bno085_buffer_postenable(struct iio_dev *indio_dev);
int bno085_buffer_predisable(struct iio_dev *indio_dev);
irqreturn_t bno085_trigger_handler(int irq, void *p);

/* Debugfs functions */
int bno085_debugfs_reg_access(struct iio_dev *indio_dev,
                            unsigned reg, unsigned writeval,
                            unsigned *readval);

/* External declarations */
extern const struct regmap_config bno085_regmap_config;
extern const struct iio_info bno085_info;
extern const struct attribute_group bno085_attribute_group;
extern const struct dev_pm_ops bno085_pm_ops;
extern const struct debugfs_reg_access_ops bno085_debugfs_reg_fops;

#endif /* _BNO085_CORE_H_ */

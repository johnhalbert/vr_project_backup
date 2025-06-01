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

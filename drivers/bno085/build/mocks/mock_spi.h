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

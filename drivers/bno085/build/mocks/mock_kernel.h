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

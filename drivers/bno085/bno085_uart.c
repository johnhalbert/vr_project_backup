// SPDX-License-Identifier: GPL-2.0
/*
 * BNO085 IMU UART driver
 *
 * Copyright (C) 2025 VR Headset Project
 */

#include <linux/module.h>
#include <linux/tty.h>
#include <linux/serial.h>
#include <linux/serial_core.h>
#include <linux/of.h>
#include <linux/of_device.h>
#include <linux/platform_device.h>
#include <linux/regmap.h>

#include "bno085_core.h"

struct bno085_uart {
    struct device *dev;
    struct tty_struct *tty;
    struct mutex lock;
    u8 rx_buffer[BNO085_FIFO_SIZE];
    int rx_count;
    struct completion rx_done;
    struct work_struct rx_work;
};

/* UART read function */
static int bno085_uart_read(struct device *dev, u8 reg, u8 *data, int len)
{
    struct bno085_uart *uart = dev_get_drvdata(dev);
    u8 cmd[2] = {0xAA, reg}; /* Start byte + register */
    int ret;
    
    mutex_lock(&uart->lock);
    
    /* Initialize completion */
    reinit_completion(&uart->rx_done);
    uart->rx_count = 0;
    
    /* Send read command */
    ret = uart->tty->ops->write(uart->tty, cmd, 2);
    if (ret != 2) {
        dev_err(dev, "Failed to write UART command\n");
        mutex_unlock(&uart->lock);
        return -EIO;
    }
    
    /* Wait for response */
    ret = wait_for_completion_timeout(&uart->rx_done, msecs_to_jiffies(100));
    if (ret == 0) {
        dev_err(dev, "UART read timeout\n");
        mutex_unlock(&uart->lock);
        return -ETIMEDOUT;
    }
    
    /* Check if we received enough data */
    if (uart->rx_count < len) {
        dev_err(dev, "UART read incomplete: %d/%d bytes\n", uart->rx_count, len);
        mutex_unlock(&uart->lock);
        return -EIO;
    }
    
    /* Copy data */
    memcpy(data, uart->rx_buffer, len);
    
    mutex_unlock(&uart->lock);
    return 0;
}

/* UART write function */
static int bno085_uart_write(struct device *dev, u8 reg, const u8 *data, int len)
{
    struct bno085_uart *uart = dev_get_drvdata(dev);
    u8 *cmd;
    int ret;
    
    /* Allocate command buffer */
    cmd = kmalloc(len + 2, GFP_KERNEL);
    if (!cmd)
        return -ENOMEM;
    
    /* Prepare command */
    cmd[0] = 0xAA; /* Start byte */
    cmd[1] = reg;
    memcpy(cmd + 2, data, len);
    
    mutex_lock(&uart->lock);
    
    /* Send command */
    ret = uart->tty->ops->write(uart->tty, cmd, len + 2);
    if (ret != len + 2) {
        dev_err(dev, "Failed to write UART command\n");
        kfree(cmd);
        mutex_unlock(&uart->lock);
        return -EIO;
    }
    
    kfree(cmd);
    mutex_unlock(&uart->lock);
    return 0;
}

/* UART FIFO read function */
static int bno085_uart_read_fifo(struct device *dev, u8 *data, int len)
{
    struct bno085_uart *uart = dev_get_drvdata(dev);
    u8 cmd[2] = {0xAA, BNO085_REG_DATA_BUFFER}; /* Start byte + FIFO register */
    int ret;
    
    mutex_lock(&uart->lock);
    
    /* Initialize completion */
    reinit_completion(&uart->rx_done);
    uart->rx_count = 0;
    
    /* Send read command */
    ret = uart->tty->ops->write(uart->tty, cmd, 2);
    if (ret != 2) {
        dev_err(dev, "Failed to write UART command\n");
        mutex_unlock(&uart->lock);
        return -EIO;
    }
    
    /* Wait for response */
    ret = wait_for_completion_timeout(&uart->rx_done, msecs_to_jiffies(100));
    if (ret == 0) {
        dev_err(dev, "UART read timeout\n");
        mutex_unlock(&uart->lock);
        return -ETIMEDOUT;
    }
    
    /* Check if we received enough data */
    if (uart->rx_count < len) {
        dev_err(dev, "UART read incomplete: %d/%d bytes\n", uart->rx_count, len);
        mutex_unlock(&uart->lock);
        return -EIO;
    }
    
    /* Copy data */
    memcpy(data, uart->rx_buffer, len);
    
    mutex_unlock(&uart->lock);
    return 0;
}

/* UART receive callback */
static void bno085_uart_rx_work(struct work_struct *work)
{
    struct bno085_uart *uart = container_of(work, struct bno085_uart, rx_work);
    
    /* Signal completion */
    complete(&uart->rx_done);
}

/* UART receive callback */
static int bno085_uart_rx_callback(struct tty_struct *tty, const u8 *data, char *flags, int count)
{
    struct bno085_uart *uart = tty->disc_data;
    
    /* Check if we have enough space */
    if (uart->rx_count + count > BNO085_FIFO_SIZE) {
        dev_warn(uart->dev, "UART RX buffer overflow\n");
        count = BNO085_FIFO_SIZE - uart->rx_count;
        if (count <= 0)
            return 0;
    }
    
    /* Copy data to buffer */
    memcpy(uart->rx_buffer + uart->rx_count, data, count);
    uart->rx_count += count;
    
    /* Schedule work to process data */
    schedule_work(&uart->rx_work);
    
    return count;
}

/* UART line discipline operations */
static struct tty_ldisc_ops bno085_uart_ldisc_ops = {
    .owner = THIS_MODULE,
    .name = "bno085",
    .receive_buf = bno085_uart_rx_callback,
};

/* UART probe function */
static int bno085_uart_probe(struct platform_device *pdev)
{
    struct device *dev = &pdev->dev;
    struct device_node *np = dev->of_node;
    struct bno085_uart *uart;
    struct bno085_transport transport;
    struct tty_struct *tty;
    int tty_idx, ret;
    
    /* Allocate driver data */
    uart = devm_kzalloc(dev, sizeof(*uart), GFP_KERNEL);
    if (!uart)
        return -ENOMEM;
    
    uart->dev = dev;
    mutex_init(&uart->lock);
    init_completion(&uart->rx_done);
    INIT_WORK(&uart->rx_work, bno085_uart_rx_work);
    
    /* Get TTY index from device tree */
    ret = of_property_read_u32(np, "tty-idx", &tty_idx);
    if (ret) {
        dev_err(dev, "Failed to get TTY index from device tree\n");
        return ret;
    }
    
    /* Get TTY device */
    tty = tty_kref_get(tty_idx);
    if (!tty) {
        dev_err(dev, "Failed to get TTY device\n");
        return -ENODEV;
    }
    
    uart->tty = tty;
    tty->disc_data = uart;
    
    /* Set up transport interface */
    transport.read = bno085_uart_read;
    transport.write = bno085_uart_write;
    transport.read_fifo = bno085_uart_read_fifo;
    
    /* Store driver data */
    platform_set_drvdata(pdev, uart);
    
    /* Call core probe function */
    return bno085_core_probe(dev, &transport, 0); /* No IRQ for UART */
}

/* UART remove function */
static int bno085_uart_remove(struct platform_device *pdev)
{
    struct bno085_uart *uart = platform_get_drvdata(pdev);
    
    /* Release TTY */
    if (uart->tty) {
        uart->tty->disc_data = NULL;
        tty_kref_put(uart->tty);
    }
    
    return bno085_core_remove(&pdev->dev);
}

/* Device tree match table */
static const struct of_device_id bno085_uart_of_match[] = {
    { .compatible = "bosch,bno085-uart" },
    { }
};
MODULE_DEVICE_TABLE(of, bno085_uart_of_match);

/* Platform driver structure */
static struct platform_driver bno085_uart_driver = {
    .driver = {
        .name = "bno085-uart",
        .of_match_table = bno085_uart_of_match,
#ifdef CONFIG_PM
        .pm = &bno085_pm_ops,
#endif
    },
    .probe = bno085_uart_probe,
    .remove = bno085_uart_remove,
};

/* Module initialization */
static int __init bno085_uart_init(void)
{
    int ret;
    
    /* Register line discipline */
    ret = tty_register_ldisc(N_BNO085, &bno085_uart_ldisc_ops);
    if (ret) {
        pr_err("Failed to register BNO085 line discipline: %d\n", ret);
        return ret;
    }
    
    /* Register platform driver */
    ret = platform_driver_register(&bno085_uart_driver);
    if (ret) {
        tty_unregister_ldisc(N_BNO085);
        return ret;
    }
    
    return 0;
}

/* Module cleanup */
static void __exit bno085_uart_exit(void)
{
    platform_driver_unregister(&bno085_uart_driver);
    tty_unregister_ldisc(N_BNO085);
}

module_init(bno085_uart_init);
module_exit(bno085_uart_exit);

MODULE_AUTHOR("VR Headset Project");
MODULE_DESCRIPTION("BNO085 IMU UART driver");
MODULE_LICENSE("GPL v2");

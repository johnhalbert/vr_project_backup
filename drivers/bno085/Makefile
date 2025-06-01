# SPDX-License-Identifier: GPL-2.0
#
# Makefile for BNO085 IMU driver
#

obj-$(CONFIG_IIO_BNO085) += bno085.o
bno085-y := bno085_core.o
bno085-$(CONFIG_IIO_BNO085_I2C) += bno085_i2c.o
bno085-$(CONFIG_IIO_BNO085_SPI) += bno085_spi.o

# Orange Pi CM5 Camera Driver Adaptation

This file contains the necessary adaptations to the OV9281 camera driver for the Orange Pi CM5 platform.

```c
// drivers/media/i2c/ov9281_orangepi.c

#include <linux/module.h>
#include <linux/of.h>
#include <linux/of_device.h>
#include <linux/of_graph.h>
#include <linux/clk.h>
#include <linux/delay.h>
#include <linux/gpio/consumer.h>
#include <linux/i2c.h>
#include <linux/regulator/consumer.h>
#include <linux/videodev2.h>
#include <media/v4l2-async.h>
#include <media/v4l2-ctrls.h>
#include <media/v4l2-device.h>
#include <media/v4l2-event.h>
#include <media/v4l2-fwnode.h>
#include <media/v4l2-subdev.h>

// Include the original OV9281 driver header
#include "ov9281.h"

// Orange Pi CM5 specific configuration
static int ov9281_configure_orangepi_cm5(struct ov9281_device *ov9281)
{
    struct device *dev = &ov9281->client->dev;
    int ret;

    dev_info(dev, "Configuring OV9281 for Orange Pi CM5\n");

    // Orange Pi CM5 specific MIPI configuration
    ret = ov9281_write_reg(ov9281->client, 0x3034, 0x0a);
    if (ret)
        return ret;

    // Orange Pi CM5 specific clock configuration
    ret = ov9281_write_reg(ov9281->client, 0x3035, 0x21);
    if (ret)
        return ret;

    // Orange Pi CM5 specific lane configuration
    ret = ov9281_write_reg(ov9281->client, 0x3036, 0x60);
    if (ret)
        return ret;

    // Orange Pi CM5 specific timing configuration
    ret = ov9281_write_reg(ov9281->client, 0x303c, 0x11);
    if (ret)
        return ret;

    // Orange Pi CM5 specific power optimization
    ret = ov9281_write_reg(ov9281->client, 0x3106, 0x11);
    if (ret)
        return ret;

    dev_info(dev, "OV9281 configured for Orange Pi CM5\n");
    return 0;
}

// Update probe function to detect Orange Pi CM5
static int ov9281_probe_orangepi(struct i2c_client *client,
                               const struct i2c_device_id *id)
{
    struct device *dev = &client->dev;
    struct ov9281_device *ov9281;
    int ret;

    // Call original probe function
    ret = ov9281_probe(client, id);
    if (ret)
        return ret;

    // Get the ov9281 device pointer
    ov9281 = i2c_get_clientdata(client);
    if (!ov9281)
        return -ENODEV;

    // Check if this is an Orange Pi CM5 device
    if (of_device_is_compatible(dev->of_node, "orangepi,ov9281-vr")) {
        dev_info(dev, "Detected Orange Pi CM5 VR camera\n");
        
        // Apply Orange Pi CM5 specific configuration
        ret = ov9281_configure_orangepi_cm5(ov9281);
        if (ret) {
            dev_err(dev, "Failed to configure for Orange Pi CM5: %d\n", ret);
            return ret;
        }
        
        // Set Orange Pi CM5 specific flags
        ov9281->is_orangepi_cm5 = true;
    }

    return 0;
}

// Update the compatible strings to include Orange Pi variant
static const struct of_device_id ov9281_of_match_orangepi[] = {
    { .compatible = "ovti,ov9281" },
    { .compatible = "orangepi,ov9281-vr" },
    { /* sentinel */ }
};
MODULE_DEVICE_TABLE(of, ov9281_of_match_orangepi);

// Update the i2c_driver structure
static struct i2c_driver ov9281_i2c_driver_orangepi = {
    .driver = {
        .name = "ov9281_orangepi",
        .of_match_table = ov9281_of_match_orangepi,
    },
    .probe = ov9281_probe_orangepi,
    .remove = ov9281_remove,
    .id_table = ov9281_id,
};

module_i2c_driver(ov9281_i2c_driver_orangepi);

MODULE_DESCRIPTION("OV9281 Camera Driver for Orange Pi CM5");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");
```

## Integration with Existing Driver

To integrate this adaptation with the existing OV9281 driver, we need to:

1. Add the Orange Pi CM5 specific configuration to the existing driver
2. Update the device tree bindings to include the Orange Pi CM5 compatible string
3. Add the Orange Pi CM5 specific flags to the driver structure

## Device Tree Binding Updates

```
Required properties for Orange Pi CM5:
- compatible: Must include "orangepi,ov9281-vr" for Orange Pi CM5 VR cameras
- reg: I2C address of the sensor
- clocks: Reference to the sensor input clock
- clock-names: Should be "xvclk"
- reset-gpios: Reference to the GPIO connected to the reset pin
- pwdn-gpios: Reference to the GPIO connected to the power down pin

Example:
vr_camera_0: vr-camera@0 {
    compatible = "orangepi,ov9281-vr";
    reg = <0x60>;
    clocks = <&cru CLK_MIPI_CAMARAOUT_M1>;
    clock-names = "xvclk";
    power-domains = <&power RK3588_PD_VI>;
    pinctrl-names = "default";
    pinctrl-0 = <&mipim1_camera1_clk>;
    reset-gpios = <&gpio1 RK_PB2 GPIO_ACTIVE_LOW>;
    pwdn-gpios = <&gpio1 RK_PB3 GPIO_ACTIVE_HIGH>;
    rockchip,camera-module-index = <0>;
    rockchip,camera-module-facing = "front";
    rockchip,camera-module-name = "VR-Camera-Left";
    rockchip,camera-module-lens-name = "VR-Lens";
    port {
        vr_camera_0_out: endpoint {
            remote-endpoint = <&csi2_0_out>;
            data-lanes = <1 2>;
        };
    };
};
```

## Build System Integration

To integrate this into the build system, add the following to the Makefile:

```makefile
# drivers/media/i2c/Makefile
obj-$(CONFIG_VIDEO_OV9281) += ov9281.o
obj-$(CONFIG_VIDEO_OV9281_ORANGEPI) += ov9281_orangepi.o
```

And add the following to the Kconfig:

```kconfig
config VIDEO_OV9281_ORANGEPI
    tristate "OV9281 camera sensor support for Orange Pi CM5"
    depends on VIDEO_OV9281 && I2C && VIDEO_V4L2
    help
      This is a Video4Linux2 sensor driver for the OmniBision
      OV9281 camera sensor, specifically adapted for the
      Orange Pi CM5 platform.

      To compile this driver as a module, choose M here: the
      module will be called ov9281_orangepi.
```

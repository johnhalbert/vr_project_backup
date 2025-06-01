/*
 * Orange Pi CM5 VR Headset ALSA Machine Driver
 *
 * Copyright (c) 2025 VR Headset Project
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License version 2 as
 * published by the Free Software Foundation.
 */

#include <linux/module.h>
#include <linux/platform_device.h>
#include <linux/of.h>
#include <linux/of_device.h>
#include <sound/soc.h>
#include <sound/pcm.h>
#include <sound/pcm_params.h>

#include "orangepi_vr_i2s.h"
#include "orangepi_vr_headphone.h"
#include "orangepi_vr_mic_array.h"
#include "orangepi_vr_beamforming.h"
#include "orangepi_vr_spatial_audio.h"
#include "orangepi_vr_machine.h"

struct orangepi_vr_card_data {
    struct snd_soc_card card;
    struct orangepi_vr_i2s_dev *i2s;
    
    bool vr_low_latency_mode;
    bool vr_beamforming_enabled;
    bool vr_spatial_audio_enabled;
    
    int playback_channels;
    int capture_channels;
};

static int orangepi_vr_card_hw_params(struct snd_pcm_substream *substream,
                                    struct snd_pcm_hw_params *params)
{
    struct snd_soc_pcm_runtime *rtd = substream->private_data;
    struct snd_soc_card *card = rtd->card;
    struct orangepi_vr_card_data *priv = snd_soc_card_get_drvdata(card);
    int ret = 0;

    /* Configure I2S format */
    ret = snd_soc_dai_set_fmt(asoc_rtd_to_cpu(rtd, 0),
                             SND_SOC_DAIFMT_I2S |
                             SND_SOC_DAIFMT_NB_NF |
                             SND_SOC_DAIFMT_CBS_CFS);
    if (ret < 0) {
        dev_err(card->dev, "Failed to set CPU DAI format: %d\n", ret);
        return ret;
    }

    /* Configure codec format */
    ret = snd_soc_dai_set_fmt(asoc_rtd_to_codec(rtd, 0),
                             SND_SOC_DAIFMT_I2S |
                             SND_SOC_DAIFMT_NB_NF |
                             SND_SOC_DAIFMT_CBS_CFS);
    if (ret < 0) {
        dev_err(card->dev, "Failed to set CODEC DAI format: %d\n", ret);
        return ret;
    }

    /* Configure system clock */
    ret = snd_soc_dai_set_sysclk(asoc_rtd_to_cpu(rtd, 0), 0,
                                params_rate(params) * 256, SND_SOC_CLOCK_OUT);
    if (ret < 0) {
        dev_err(card->dev, "Failed to set CPU DAI system clock: %d\n", ret);
        return ret;
    }

    return 0;
}

static int orangepi_vr_card_startup(struct snd_pcm_substream *substream)
{
    struct snd_soc_pcm_runtime *rtd = substream->private_data;
    struct snd_soc_card *card = rtd->card;
    struct orangepi_vr_card_data *priv = snd_soc_card_get_drvdata(card);

    /* Configure constraints based on VR mode */
    if (priv->vr_low_latency_mode) {
        /* Limit buffer size for low latency */
        snd_pcm_hw_constraint_minmax(substream->runtime,
                                    SNDRV_PCM_HW_PARAM_BUFFER_SIZE,
                                    1024, 4096);
        
        /* Limit period size for low latency */
        snd_pcm_hw_constraint_minmax(substream->runtime,
                                    SNDRV_PCM_HW_PARAM_PERIOD_SIZE,
                                    256, 1024);
    }

    return 0;
}

static struct snd_soc_ops orangepi_vr_card_ops = {
    .startup = orangepi_vr_card_startup,
    .hw_params = orangepi_vr_card_hw_params,
};

static int orangepi_vr_card_init(struct snd_soc_pcm_runtime *rtd)
{
    struct snd_soc_card *card = rtd->card;
    struct orangepi_vr_card_data *priv = snd_soc_card_get_drvdata(card);
    int ret;

    /* Configure CODEC for VR-specific settings */
    if (priv->vr_low_latency_mode) {
        /* Apply low-latency settings */
        dev_info(card->dev, "Configuring for VR low-latency mode\n");
    }

    if (priv->vr_beamforming_enabled) {
        /* Initialize beamforming */
        ret = orangepi_vr_beamforming_init(card->dev);
        if (ret < 0) {
            dev_err(card->dev, "Failed to initialize beamforming: %d\n", ret);
            return ret;
        }
        dev_info(card->dev, "Beamforming initialized\n");
    }

    if (priv->vr_spatial_audio_enabled) {
        /* Initialize spatial audio */
        ret = orangepi_vr_spatial_audio_init(card->dev);
        if (ret < 0) {
            dev_err(card->dev, "Failed to initialize spatial audio: %d\n", ret);
            return ret;
        }
        dev_info(card->dev, "Spatial audio initialized\n");
    }

    return 0;
}

static struct snd_soc_dai_link orangepi_vr_dai_links[] = {
    {
        .name = "Orange Pi CM5 VR",
        .stream_name = "Orange Pi CM5 VR Audio",
        .cpu_dai_name = "orangepi-vr-i2s",
        .codec_dai_name = "orangepi-vr-codec",
        .platform_name = "orangepi-vr-i2s",
        .codec_name = "orangepi-vr-codec",
        .init = orangepi_vr_card_init,
        .ops = &orangepi_vr_card_ops,
        .dai_fmt = SND_SOC_DAIFMT_I2S | SND_SOC_DAIFMT_NB_NF | SND_SOC_DAIFMT_CBS_CFS,
    },
};

static const struct snd_soc_dapm_widget orangepi_vr_dapm_widgets[] = {
    SND_SOC_DAPM_HP("Headphone", NULL),
    SND_SOC_DAPM_MIC("Microphone Array", NULL),
};

static const struct snd_soc_dapm_route orangepi_vr_dapm_routes[] = {
    {"Headphone", NULL, "HPOL"},
    {"Headphone", NULL, "HPOR"},
    {"MIC1", NULL, "Microphone Array"},
    {"MIC2", NULL, "Microphone Array"},
    {"MIC3", NULL, "Microphone Array"},
    {"MIC4", NULL, "Microphone Array"},
};

static int orangepi_vr_machine_probe(struct platform_device *pdev)
{
    struct device_node *np = pdev->dev.of_node;
    struct snd_soc_card *card;
    struct orangepi_vr_card_data *priv;
    struct orangepi_vr_i2s_dev *i2s;
    int ret;

    priv = devm_kzalloc(&pdev->dev, sizeof(*priv), GFP_KERNEL);
    if (!priv)
        return -ENOMEM;

    card = &priv->card;
    card->dev = &pdev->dev;
    card->owner = THIS_MODULE;
    card->name = "Orange Pi CM5 VR Audio";
    card->dai_link = orangepi_vr_dai_links;
    card->num_links = ARRAY_SIZE(orangepi_vr_dai_links);
    card->dapm_widgets = orangepi_vr_dapm_widgets;
    card->num_dapm_widgets = ARRAY_SIZE(orangepi_vr_dapm_widgets);
    card->dapm_routes = orangepi_vr_dapm_routes;
    card->num_dapm_routes = ARRAY_SIZE(orangepi_vr_dapm_routes);

    /* Parse VR-specific properties */
    priv->vr_low_latency_mode = of_property_read_bool(np, "vr,low-latency-mode");
    priv->vr_beamforming_enabled = of_property_read_bool(np, "vr,beamforming-enabled");
    priv->vr_spatial_audio_enabled = of_property_read_bool(np, "vr,spatial-audio-enabled");

    /* Get channel configuration */
    of_property_read_u32(np, "orangepi,playback-channels", &priv->playback_channels);
    of_property_read_u32(np, "orangepi,capture-channels", &priv->capture_channels);

    /* Set default values if not specified */
    if (!priv->playback_channels)
        priv->playback_channels = 2;
    if (!priv->capture_channels)
        priv->capture_channels = 4;

    /* Initialize I2S controller */
    i2s = devm_kzalloc(&pdev->dev, sizeof(*i2s), GFP_KERNEL);
    if (!i2s)
        return -ENOMEM;

    i2s->vr_low_latency_mode = priv->vr_low_latency_mode;
    i2s->vr_beamforming_enabled = priv->vr_beamforming_enabled;
    i2s->vr_spatial_audio_enabled = priv->vr_spatial_audio_enabled;
    i2s->playback_channels = priv->playback_channels;
    i2s->capture_channels = priv->capture_channels;

    priv->i2s = i2s;

    /* Initialize headphone driver */
    ret = orangepi_vr_headphone_init(&pdev->dev, i2s);
    if (ret < 0) {
        dev_err(&pdev->dev, "Failed to initialize headphone driver: %d\n", ret);
        return ret;
    }

    /* Initialize microphone array driver */
    ret = orangepi_vr_mic_array_init(&pdev->dev, i2s);
    if (ret < 0) {
        dev_err(&pdev->dev, "Failed to initialize microphone array driver: %d\n", ret);
        return ret;
    }

    snd_soc_card_set_drvdata(card, priv);

    ret = devm_snd_soc_register_card(&pdev->dev, card);
    if (ret) {
        dev_err(&pdev->dev, "Failed to register sound card: %d\n", ret);
        return ret;
    }

    dev_info(&pdev->dev, "Orange Pi CM5 VR Audio Card registered\n");
    if (priv->vr_low_latency_mode)
        dev_info(&pdev->dev, "VR low-latency mode enabled\n");
    if (priv->vr_beamforming_enabled)
        dev_info(&pdev->dev, "VR beamforming enabled\n");
    if (priv->vr_spatial_audio_enabled)
        dev_info(&pdev->dev, "VR spatial audio enabled\n");

    return 0;
}

static const struct of_device_id orangepi_vr_machine_of_match[] = {
    { .compatible = "orangepi,vr-sound", },
    {},
};
MODULE_DEVICE_TABLE(of, orangepi_vr_machine_of_match);

static struct platform_driver orangepi_vr_machine_driver = {
    .driver = {
        .name = "orangepi-vr-sound",
        .of_match_table = orangepi_vr_machine_of_match,
    },
    .probe = orangepi_vr_machine_probe,
};
module_platform_driver(orangepi_vr_machine_driver);

MODULE_DESCRIPTION("Orange Pi CM5 VR Headset ALSA Machine Driver");
MODULE_AUTHOR("VR Headset Project");
MODULE_LICENSE("GPL v2");

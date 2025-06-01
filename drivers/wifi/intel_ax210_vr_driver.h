/* SPDX-License-Identifier: GPL-2.0 */
/*
 * Intel AX210 WiFi driver optimizations for VR applications
 *
 * Copyright (C) 2025 VR SLAM Project Team
 */

#ifndef __INTEL_AX210_VR_DRIVER_H__
#define __INTEL_AX210_VR_DRIVER_H__

#include <linux/types.h>
#include <linux/netdevice.h>
#include <linux/skbuff.h>
#include <linux/if_ether.h>
#include <linux/ieee80211.h>
#include <net/cfg80211.h>

/* VR mode configuration */
enum intel_ax210_vr_mode {
    INTEL_AX210_VR_MODE_DISABLED = 0,
    INTEL_AX210_VR_MODE_ENABLED = 1,
};

/* Power profiles */
enum intel_ax210_power_profile {
    INTEL_AX210_POWER_MAX_PERFORMANCE = 0,  /* Maximum performance, highest power */
    INTEL_AX210_POWER_VR_ACTIVE = 1,        /* Balanced for active VR use */
    INTEL_AX210_POWER_VR_IDLE = 2,          /* Optimized for idle VR */
    INTEL_AX210_POWER_STANDARD = 3,         /* Standard power management */
    INTEL_AX210_POWER_MAX_SAVING = 4,       /* Maximum power saving */
};

/* Traffic classes */
enum intel_ax210_traffic_class {
    INTEL_AX210_TC_TRACKING = 0,    /* VR tracking data (highest priority) */
    INTEL_AX210_TC_CONTROL = 1,     /* VR control data */
    INTEL_AX210_TC_VIDEO = 2,       /* VR video streams */
    INTEL_AX210_TC_AUDIO = 3,       /* VR audio streams */
    INTEL_AX210_TC_BACKGROUND = 4,  /* Background data (lowest priority) */
};

/* Latency configuration */
struct intel_ax210_latency_config {
    bool latency_mode_enabled;          /* Enable/disable latency mode */
    u8 aggregation_limit;               /* Maximum A-MPDU size (0-64) */
    u8 queue_size_limit;                /* TX queue size limit */
    u8 retry_limit;                     /* Maximum retry count */
    u16 rts_threshold;                  /* RTS threshold */
    u16 beacon_interval;                /* Beacon interval in TUs */
    u8 power_save_mode;                 /* Power save mode (0-3) */
    u8 spatial_streams;                 /* Number of spatial streams */
    u8 bandwidth;                       /* Channel bandwidth */
    u8 guard_interval;                  /* Guard interval */
};

/* QoS configuration */
struct intel_ax210_qos_config {
    bool auto_classification;           /* Enable automatic classification */
    u8 tracking_dscp;                   /* DSCP value for tracking data */
    u8 control_dscp;                    /* DSCP value for control data */
    u8 video_dscp;                      /* DSCP value for video data */
    u8 audio_dscp;                      /* DSCP value for audio data */
    u8 background_dscp;                 /* DSCP value for background data */
    u8 tracking_queue_weight;           /* Weight for tracking queue */
    u8 control_queue_weight;            /* Weight for control queue */
    u8 video_queue_weight;              /* Weight for video queue */
    u8 audio_queue_weight;              /* Weight for audio queue */
    u8 background_queue_weight;         /* Weight for background queue */
};

/* Channel metrics */
struct intel_ax210_channel_metrics {
    u8 channel;                         /* Current channel */
    u8 utilization;                     /* Channel utilization (0-100%) */
    u8 interference;                    /* Interference level (0-100%) */
    s8 noise_floor;                     /* Noise floor (dBm) */
    s8 signal_strength;                 /* Signal strength (dBm) */
    u32 tx_packets;                     /* Transmitted packets */
    u32 rx_packets;                     /* Received packets */
    u32 tx_errors;                      /* Transmission errors */
    u32 rx_errors;                      /* Reception errors */
    u32 retries;                        /* Retry count */
    u64 timestamp;                      /* Timestamp (microseconds) */
};

/* Channel configuration */
struct intel_ax210_channel_config {
    bool auto_channel_selection;        /* Enable automatic channel selection */
    u16 scan_interval;                  /* Channel scan interval (seconds) */
    u8 interference_threshold;          /* Interference threshold (0-100%) */
    u8 utilization_threshold;           /* Utilization threshold (0-100%) */
    u8 hysteresis;                      /* Hysteresis for channel switching */
    bool prefer_5ghz;                   /* Prefer 5GHz band */
    bool prefer_160mhz;                 /* Prefer 160MHz channels */
    bool allow_dfs;                     /* Allow DFS channels */
};

/* Power configuration */
struct intel_ax210_power_config {
    enum intel_ax210_power_profile profile;  /* Current power profile */
    bool dynamic_adjustment;            /* Enable dynamic adjustment */
    u16 active_timeout;                 /* Timeout for active state (ms) */
    u16 idle_timeout;                   /* Timeout for idle state (ms) */
    s8 tx_power;                        /* Transmit power level (dBm) */
    bool disable_spatial_streams;       /* Disable unused spatial streams */
    bool disable_unused_chains;         /* Disable unused antenna chains */
    bool enable_ps_poll;                /* Enable PS-Poll */
    bool enable_uapsd;                  /* Enable U-APSD */
};

/* Performance metrics */
struct intel_ax210_performance_metrics {
    /* Latency metrics */
    u32 avg_latency_us;                 /* Average latency (microseconds) */
    u32 min_latency_us;                 /* Minimum latency (microseconds) */
    u32 max_latency_us;                 /* Maximum latency (microseconds) */
    u32 jitter_us;                      /* Jitter (microseconds) */
    
    /* Throughput metrics */
    u32 tx_throughput_kbps;             /* TX throughput (kbps) */
    u32 rx_throughput_kbps;             /* RX throughput (kbps) */
    
    /* Reliability metrics */
    u32 packet_loss_percent;            /* Packet loss percentage */
    u32 retry_count;                    /* Retry count */
    u32 crc_error_count;                /* CRC error count */
    
    /* Channel metrics */
    u8 channel_utilization;             /* Channel utilization (0-100%) */
    u8 interference_level;              /* Interference level (0-100%) */
    s8 signal_strength;                 /* Signal strength (dBm) */
    s8 noise_level;                     /* Noise level (dBm) */
    
    /* Power metrics */
    u8 tx_power;                        /* TX power (dBm) */
    u8 power_save_level;                /* Power save level (0-5) */
    u32 power_consumption_mw;           /* Estimated power consumption (mW) */
    
    /* QoS metrics */
    u32 tracking_queue_depth;           /* Tracking queue depth */
    u32 control_queue_depth;            /* Control queue depth */
    u32 video_queue_depth;              /* Video queue depth */
    u32 audio_queue_depth;              /* Audio queue depth */
    u32 background_queue_depth;         /* Background queue depth */
    
    /* Timestamp */
    u64 timestamp;                      /* Timestamp (microseconds) */
};

/* VR application registration */
struct intel_ax210_vr_app_info {
    char app_name[32];                  /* Application name */
    u16 tracking_port;                  /* Port used for tracking data */
    u16 control_port;                   /* Port used for control data */
    u16 video_port;                     /* Port used for video data */
    u16 audio_port;                     /* Port used for audio data */
    u32 app_id;                         /* Application ID (returned) */
};

/* Netlink commands */
enum intel_ax210_vr_nl_commands {
    INTEL_AX210_VR_CMD_UNSPEC,
    INTEL_AX210_VR_CMD_SET_MODE,        /* Set VR mode */
    INTEL_AX210_VR_CMD_GET_MODE,        /* Get VR mode */
    INTEL_AX210_VR_CMD_SET_LATENCY,     /* Set latency configuration */
    INTEL_AX210_VR_CMD_GET_LATENCY,     /* Get latency configuration */
    INTEL_AX210_VR_CMD_SET_QOS,         /* Set QoS configuration */
    INTEL_AX210_VR_CMD_GET_QOS,         /* Get QoS configuration */
    INTEL_AX210_VR_CMD_SET_CHANNEL,     /* Set channel configuration */
    INTEL_AX210_VR_CMD_GET_CHANNEL,     /* Get channel configuration */
    INTEL_AX210_VR_CMD_SET_POWER,       /* Set power configuration */
    INTEL_AX210_VR_CMD_GET_POWER,       /* Get power configuration */
    INTEL_AX210_VR_CMD_GET_METRICS,     /* Get performance metrics */
    INTEL_AX210_VR_CMD_REGISTER_APP,    /* Register VR application */
    INTEL_AX210_VR_CMD_UNREGISTER_APP,  /* Unregister VR application */
    INTEL_AX210_VR_CMD_MAX,
};

/* Netlink attributes */
enum intel_ax210_vr_nl_attrs {
    INTEL_AX210_VR_ATTR_UNSPEC,
    INTEL_AX210_VR_ATTR_MODE,           /* VR mode (u8) */
    INTEL_AX210_VR_ATTR_LATENCY_CONFIG, /* Latency configuration (nested) */
    INTEL_AX210_VR_ATTR_QOS_CONFIG,     /* QoS configuration (nested) */
    INTEL_AX210_VR_ATTR_CHANNEL_CONFIG, /* Channel configuration (nested) */
    INTEL_AX210_VR_ATTR_POWER_CONFIG,   /* Power configuration (nested) */
    INTEL_AX210_VR_ATTR_METRICS,        /* Performance metrics (nested) */
    INTEL_AX210_VR_ATTR_APP_INFO,       /* Application info (nested) */
    INTEL_AX210_VR_ATTR_APP_ID,         /* Application ID (u32) */
    INTEL_AX210_VR_ATTR_MAX,
};

/* Latency configuration attributes */
enum intel_ax210_vr_latency_attrs {
    INTEL_AX210_VR_LATENCY_ATTR_UNSPEC,
    INTEL_AX210_VR_LATENCY_ATTR_ENABLED,        /* Enabled flag (u8) */
    INTEL_AX210_VR_LATENCY_ATTR_AGG_LIMIT,      /* Aggregation limit (u8) */
    INTEL_AX210_VR_LATENCY_ATTR_QUEUE_LIMIT,    /* Queue size limit (u8) */
    INTEL_AX210_VR_LATENCY_ATTR_RETRY_LIMIT,    /* Retry limit (u8) */
    INTEL_AX210_VR_LATENCY_ATTR_RTS_THRESHOLD,  /* RTS threshold (u16) */
    INTEL_AX210_VR_LATENCY_ATTR_BEACON_INTERVAL,/* Beacon interval (u16) */
    INTEL_AX210_VR_LATENCY_ATTR_PS_MODE,        /* Power save mode (u8) */
    INTEL_AX210_VR_LATENCY_ATTR_SPATIAL_STREAMS,/* Spatial streams (u8) */
    INTEL_AX210_VR_LATENCY_ATTR_BANDWIDTH,      /* Bandwidth (u8) */
    INTEL_AX210_VR_LATENCY_ATTR_GUARD_INTERVAL, /* Guard interval (u8) */
    INTEL_AX210_VR_LATENCY_ATTR_MAX,
};

/* QoS configuration attributes */
enum intel_ax210_vr_qos_attrs {
    INTEL_AX210_VR_QOS_ATTR_UNSPEC,
    INTEL_AX210_VR_QOS_ATTR_AUTO_CLASS,         /* Auto classification (u8) */
    INTEL_AX210_VR_QOS_ATTR_TRACKING_DSCP,      /* Tracking DSCP (u8) */
    INTEL_AX210_VR_QOS_ATTR_CONTROL_DSCP,       /* Control DSCP (u8) */
    INTEL_AX210_VR_QOS_ATTR_VIDEO_DSCP,         /* Video DSCP (u8) */
    INTEL_AX210_VR_QOS_ATTR_AUDIO_DSCP,         /* Audio DSCP (u8) */
    INTEL_AX210_VR_QOS_ATTR_BACKGROUND_DSCP,    /* Background DSCP (u8) */
    INTEL_AX210_VR_QOS_ATTR_TRACKING_WEIGHT,    /* Tracking weight (u8) */
    INTEL_AX210_VR_QOS_ATTR_CONTROL_WEIGHT,     /* Control weight (u8) */
    INTEL_AX210_VR_QOS_ATTR_VIDEO_WEIGHT,       /* Video weight (u8) */
    INTEL_AX210_VR_QOS_ATTR_AUDIO_WEIGHT,       /* Audio weight (u8) */
    INTEL_AX210_VR_QOS_ATTR_BACKGROUND_WEIGHT,  /* Background weight (u8) */
    INTEL_AX210_VR_QOS_ATTR_MAX,
};

/* Channel configuration attributes */
enum intel_ax210_vr_channel_attrs {
    INTEL_AX210_VR_CHANNEL_ATTR_UNSPEC,
    INTEL_AX210_VR_CHANNEL_ATTR_AUTO_SELECT,    /* Auto selection (u8) */
    INTEL_AX210_VR_CHANNEL_ATTR_SCAN_INTERVAL,  /* Scan interval (u16) */
    INTEL_AX210_VR_CHANNEL_ATTR_INTF_THRESHOLD, /* Interference threshold (u8) */
    INTEL_AX210_VR_CHANNEL_ATTR_UTIL_THRESHOLD, /* Utilization threshold (u8) */
    INTEL_AX210_VR_CHANNEL_ATTR_HYSTERESIS,     /* Hysteresis (u8) */
    INTEL_AX210_VR_CHANNEL_ATTR_PREFER_5GHZ,    /* Prefer 5GHz (u8) */
    INTEL_AX210_VR_CHANNEL_ATTR_PREFER_160MHZ,  /* Prefer 160MHz (u8) */
    INTEL_AX210_VR_CHANNEL_ATTR_ALLOW_DFS,      /* Allow DFS (u8) */
    INTEL_AX210_VR_CHANNEL_ATTR_MAX,
};

/* Power configuration attributes */
enum intel_ax210_vr_power_attrs {
    INTEL_AX210_VR_POWER_ATTR_UNSPEC,
    INTEL_AX210_VR_POWER_ATTR_PROFILE,          /* Power profile (u8) */
    INTEL_AX210_VR_POWER_ATTR_DYNAMIC_ADJ,      /* Dynamic adjustment (u8) */
    INTEL_AX210_VR_POWER_ATTR_ACTIVE_TIMEOUT,   /* Active timeout (u16) */
    INTEL_AX210_VR_POWER_ATTR_IDLE_TIMEOUT,     /* Idle timeout (u16) */
    INTEL_AX210_VR_POWER_ATTR_TX_POWER,         /* TX power (s8) */
    INTEL_AX210_VR_POWER_ATTR_DISABLE_STREAMS,  /* Disable streams (u8) */
    INTEL_AX210_VR_POWER_ATTR_DISABLE_CHAINS,   /* Disable chains (u8) */
    INTEL_AX210_VR_POWER_ATTR_ENABLE_PS_POLL,   /* Enable PS-Poll (u8) */
    INTEL_AX210_VR_POWER_ATTR_ENABLE_UAPSD,     /* Enable U-APSD (u8) */
    INTEL_AX210_VR_POWER_ATTR_MAX,
};

/* Application info attributes */
enum intel_ax210_vr_app_attrs {
    INTEL_AX210_VR_APP_ATTR_UNSPEC,
    INTEL_AX210_VR_APP_ATTR_NAME,               /* App name (string) */
    INTEL_AX210_VR_APP_ATTR_TRACKING_PORT,      /* Tracking port (u16) */
    INTEL_AX210_VR_APP_ATTR_CONTROL_PORT,       /* Control port (u16) */
    INTEL_AX210_VR_APP_ATTR_VIDEO_PORT,         /* Video port (u16) */
    INTEL_AX210_VR_APP_ATTR_AUDIO_PORT,         /* Audio port (u16) */
    INTEL_AX210_VR_APP_ATTR_ID,                 /* App ID (u32) */
    INTEL_AX210_VR_APP_ATTR_MAX,
};

/* Performance metrics attributes */
enum intel_ax210_vr_metrics_attrs {
    INTEL_AX210_VR_METRICS_ATTR_UNSPEC,
    INTEL_AX210_VR_METRICS_ATTR_AVG_LATENCY,    /* Average latency (u32) */
    INTEL_AX210_VR_METRICS_ATTR_MIN_LATENCY,    /* Minimum latency (u32) */
    INTEL_AX210_VR_METRICS_ATTR_MAX_LATENCY,    /* Maximum latency (u32) */
    INTEL_AX210_VR_METRICS_ATTR_JITTER,         /* Jitter (u32) */
    INTEL_AX210_VR_METRICS_ATTR_TX_THROUGHPUT,  /* TX throughput (u32) */
    INTEL_AX210_VR_METRICS_ATTR_RX_THROUGHPUT,  /* RX throughput (u32) */
    INTEL_AX210_VR_METRICS_ATTR_PACKET_LOSS,    /* Packet loss (u32) */
    INTEL_AX210_VR_METRICS_ATTR_RETRY_COUNT,    /* Retry count (u32) */
    INTEL_AX210_VR_METRICS_ATTR_CRC_ERRORS,     /* CRC errors (u32) */
    INTEL_AX210_VR_METRICS_ATTR_CHANNEL_UTIL,   /* Channel utilization (u8) */
    INTEL_AX210_VR_METRICS_ATTR_INTERFERENCE,   /* Interference level (u8) */
    INTEL_AX210_VR_METRICS_ATTR_SIGNAL,         /* Signal strength (s8) */
    INTEL_AX210_VR_METRICS_ATTR_NOISE,          /* Noise level (s8) */
    INTEL_AX210_VR_METRICS_ATTR_TX_POWER,       /* TX power (u8) */
    INTEL_AX210_VR_METRICS_ATTR_PS_LEVEL,       /* Power save level (u8) */
    INTEL_AX210_VR_METRICS_ATTR_POWER_CONSUMP,  /* Power consumption (u32) */
    INTEL_AX210_VR_METRICS_ATTR_TRACKING_DEPTH, /* Tracking queue depth (u32) */
    INTEL_AX210_VR_METRICS_ATTR_CONTROL_DEPTH,  /* Control queue depth (u32) */
    INTEL_AX210_VR_METRICS_ATTR_VIDEO_DEPTH,    /* Video queue depth (u32) */
    INTEL_AX210_VR_METRICS_ATTR_AUDIO_DEPTH,    /* Audio queue depth (u32) */
    INTEL_AX210_VR_METRICS_ATTR_BG_DEPTH,       /* Background queue depth (u32) */
    INTEL_AX210_VR_METRICS_ATTR_TIMESTAMP,      /* Timestamp (u64) */
    INTEL_AX210_VR_METRICS_ATTR_MAX,
};

/* Driver private data */
struct intel_ax210_vr_priv {
    struct net_device *dev;                     /* Network device */
    struct wireless_dev *wdev;                  /* Wireless device */
    
    /* VR mode configuration */
    enum intel_ax210_vr_mode vr_mode;           /* VR mode */
    
    /* Configuration structures */
    struct intel_ax210_latency_config latency_config;
    struct intel_ax210_qos_config qos_config;
    struct intel_ax210_channel_config channel_config;
    struct intel_ax210_power_config power_config;
    
    /* Metrics */
    struct intel_ax210_performance_metrics metrics;
    
    /* Registered applications */
    struct list_head app_list;
    spinlock_t app_list_lock;
    u32 next_app_id;
    
    /* Netlink */
    struct genl_family *nl_family;
    
    /* Work queues */
    struct workqueue_struct *wq;
    struct delayed_work metrics_work;
    struct delayed_work channel_scan_work;
    struct delayed_work power_adjust_work;
    
    /* Locks */
    spinlock_t lock;
    
    /* Statistics */
    atomic_t tracking_packets;
    atomic_t control_packets;
    atomic_t video_packets;
    atomic_t audio_packets;
    atomic_t background_packets;
    
    /* Timestamps for latency calculation */
    ktime_t last_tx_timestamp[5];  /* One per traffic class */
    ktime_t last_rx_timestamp[5];  /* One per traffic class */
    
    /* Original driver callbacks */
    struct {
        netdev_tx_t (*ndo_start_xmit)(struct sk_buff *skb, struct net_device *dev);
        int (*ndo_select_queue)(struct net_device *dev, struct sk_buff *skb,
                               struct net_device *sb_dev);
    } orig_ops;
};

/* Function prototypes */

/* Initialization and cleanup */
int intel_ax210_vr_init(struct net_device *dev);
void intel_ax210_vr_cleanup(struct net_device *dev);

/* Configuration */
int intel_ax210_vr_set_mode(struct intel_ax210_vr_priv *priv, enum intel_ax210_vr_mode mode);
int intel_ax210_vr_set_latency_config(struct intel_ax210_vr_priv *priv, 
                                     const struct intel_ax210_latency_config *config);
int intel_ax210_vr_set_qos_config(struct intel_ax210_vr_priv *priv,
                                 const struct intel_ax210_qos_config *config);
int intel_ax210_vr_set_channel_config(struct intel_ax210_vr_priv *priv,
                                     const struct intel_ax210_channel_config *config);
int intel_ax210_vr_set_power_config(struct intel_ax210_vr_priv *priv,
                                   const struct intel_ax210_power_config *config);

/* Traffic classification */
enum intel_ax210_traffic_class intel_ax210_vr_classify_packet(struct intel_ax210_vr_priv *priv,
                                                            struct sk_buff *skb);

/* Packet scheduling */
void intel_ax210_vr_schedule_packet(struct intel_ax210_vr_priv *priv,
                                  struct sk_buff *skb,
                                  enum intel_ax210_traffic_class tc);

/* Metrics */
void intel_ax210_vr_update_metrics(struct intel_ax210_vr_priv *priv);
void intel_ax210_vr_get_metrics(struct intel_ax210_vr_priv *priv,
                              struct intel_ax210_performance_metrics *metrics);

/* Application registration */
int intel_ax210_vr_register_app(struct intel_ax210_vr_priv *priv,
                              const struct intel_ax210_vr_app_info *app_info,
                              u32 *app_id);
int intel_ax210_vr_unregister_app(struct intel_ax210_vr_priv *priv, u32 app_id);

/* Netlink interface */
int intel_ax210_vr_init_netlink(struct intel_ax210_vr_priv *priv);
void intel_ax210_vr_cleanup_netlink(struct intel_ax210_vr_priv *priv);

/* Sysfs interface */
int intel_ax210_vr_init_sysfs(struct intel_ax210_vr_priv *priv);
void intel_ax210_vr_cleanup_sysfs(struct intel_ax210_vr_priv *priv);

/* Packet handling */
netdev_tx_t intel_ax210_vr_start_xmit(struct sk_buff *skb, struct net_device *dev);
int intel_ax210_vr_select_queue(struct net_device *dev, struct sk_buff *skb,
                              struct net_device *sb_dev);

/* Channel management */
void intel_ax210_vr_scan_channels(struct work_struct *work);
int intel_ax210_vr_select_channel(struct intel_ax210_vr_priv *priv);

/* Power management */
void intel_ax210_vr_adjust_power(struct work_struct *work);
int intel_ax210_vr_set_power_profile(struct intel_ax210_vr_priv *priv,
                                   enum intel_ax210_power_profile profile);

#endif /* __INTEL_AX210_VR_DRIVER_H__ */

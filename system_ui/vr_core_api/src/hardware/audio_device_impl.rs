//! Audio device implementation for the Hardware Access API.
//!
//! This module provides concrete implementations of audio devices for the VR headset,
//! including headphones, microphones, and spatial audio processing.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::device::{
    Device, DeviceCapability, DeviceError, DeviceEvent, DeviceEventHandler, DeviceEventType,
    DeviceInfo, DeviceResult, DeviceState, DeviceType, DeviceBus,
};
use super::audio::{
    AudioCapability, AudioChannelConfig, AudioConfig, AudioDevice, AudioFormat,
    AudioInputDevice, AudioOutputDevice, AudioSampleRate, SpatialAudioConfig,
};

/// VR Headphone device implementation.
#[derive(Debug)]
pub struct VRHeadphone {
    /// Device information
    info: DeviceInfo,
    
    /// Audio configuration
    config: AudioConfig,
    
    /// Available sample rates
    available_sample_rates: Vec<AudioSampleRate>,
    
    /// Available channel configurations
    available_channel_configs: Vec<AudioChannelConfig>,
    
    /// Available audio formats
    available_formats: Vec<AudioFormat>,
    
    /// Spatial audio configuration
    spatial_audio_config: SpatialAudioConfig,
    
    /// Volume level (0.0 - 1.0)
    volume: f32,
    
    /// Muted state
    muted: bool,
    
    /// Power consumption in watts
    power_consumption: f32,
    
    /// Event handlers
    event_handlers: Vec<DeviceEventHandler>,
    
    /// Last update time
    last_update: Instant,
}

impl VRHeadphone {
    /// Create a new VRHeadphone.
    pub fn new(
        id: String,
        name: String,
        manufacturer: String,
        model: String,
    ) -> Self {
        let now = chrono::Utc::now();
        
        // Create device info
        let mut info = DeviceInfo {
            id,
            name,
            device_type: DeviceType::Audio,
            manufacturer,
            model,
            serial_number: None,
            firmware_version: None,
            driver_version: None,
            bus_type: DeviceBus::USB,
            bus_address: None,
            capabilities: vec![
                DeviceCapability::PowerControl,
                DeviceCapability::Configuration,
                DeviceCapability::Statistics,
                DeviceCapability::PowerManagement,
                DeviceCapability::AudioOutput,
                DeviceCapability::SpatialAudio,
            ],
            state: DeviceState::Connected,
            description: Some("VR Headphone".to_string()),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        // Add audio-specific properties
        info.properties.insert("audio_type".to_string(), "headphone".to_string());
        info.properties.insert("audio_channels".to_string(), "stereo".to_string());
        
        // Create available sample rates
        let available_sample_rates = vec![
            AudioSampleRate::new(44100),
            AudioSampleRate::new(48000),
            AudioSampleRate::new(96000),
        ];
        
        // Create available channel configurations
        let available_channel_configs = vec![
            AudioChannelConfig::Stereo,
            AudioChannelConfig::Surround51,
            AudioChannelConfig::Surround71,
        ];
        
        // Create available audio formats
        let available_formats = vec![
            AudioFormat::PCM16,
            AudioFormat::PCM24,
            AudioFormat::PCM32Float,
        ];
        
        // Create audio configuration
        let config = AudioConfig::new(
            available_sample_rates[1], // 48000 Hz
            available_channel_configs[0], // Stereo
            available_formats[0], // PCM16
        );
        
        // Create spatial audio configuration
        let spatial_audio_config = SpatialAudioConfig::vr_optimized();
        
        Self {
            info,
            config,
            available_sample_rates,
            available_channel_configs,
            available_formats,
            spatial_audio_config,
            volume: 0.8,
            muted: false,
            power_consumption: 0.5,
            event_handlers: Vec::new(),
            last_update: Instant::now(),
        }
    }
    
    /// Update the device status.
    fn update_status(&mut self) {
        // Simulate power consumption changes based on volume and time
        let elapsed = self.last_update.elapsed().as_secs_f32();
        let volume_factor = if self.muted { 0.1 } else { self.volume };
        
        // Power consumption varies with volume
        self.power_consumption = 0.2 + (volume_factor * 0.5);
        
        self.last_update = Instant::now();
    }
    
    /// Dispatch an event to all registered handlers.
    fn dispatch_event(&self, event_type: DeviceEventType) {
        let event = DeviceEvent::new(self.info.id.clone(), event_type);
        
        for handler in &self.event_handlers {
            handler(&event);
        }
    }
}

impl Device for VRHeadphone {
    fn info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.clone())
    }
    
    fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing VR Headphone: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::Initializing;
        
        // Simulate initialization delay
        std::thread::sleep(Duration::from_millis(100));
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Initialized);
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> DeviceResult<()> {
        info!("Shutting down VR Headphone: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::ShuttingDown;
        
        // Simulate shutdown delay
        std::thread::sleep(Duration::from_millis(50));
        
        // Update state
        self.info.state = DeviceState::Disconnected;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Shutdown);
        
        Ok(())
    }
    
    fn reset(&mut self) -> DeviceResult<()> {
        info!("Resetting VR Headphone: {}", self.info.id);
        
        // Update state
        let previous_state = self.info.state;
        self.info.state = DeviceState::Initializing;
        
        // Simulate reset delay
        std::thread::sleep(Duration::from_millis(200));
        
        // Reset configuration to defaults
        self.config = AudioConfig::new(
            self.available_sample_rates[1], // 48000 Hz
            self.available_channel_configs[0], // Stereo
            self.available_formats[0], // PCM16
        );
        
        self.volume = 0.8;
        self.muted = false;
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Reset);
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: DeviceState::Ready,
        });
        
        Ok(())
    }
    
    fn is_connected(&self) -> DeviceResult<bool> {
        Ok(self.info.state != DeviceState::Disconnected)
    }
    
    fn state(&self) -> DeviceResult<DeviceState> {
        Ok(self.info.state)
    }
    
    fn set_state(&mut self, state: DeviceState) -> DeviceResult<()> {
        let previous_state = self.info.state;
        self.info.state = state;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: state,
        });
        
        Ok(())
    }
    
    fn has_capability(&self, capability: DeviceCapability) -> DeviceResult<bool> {
        Ok(self.info.capabilities.contains(&capability))
    }
    
    fn properties(&self) -> DeviceResult<HashMap<String, String>> {
        Ok(self.info.properties.clone())
    }
    
    fn get_property(&self, key: &str) -> DeviceResult<Option<String>> {
        Ok(self.info.properties.get(key).cloned())
    }
    
    fn set_property(&mut self, key: &str, value: &str) -> DeviceResult<()> {
        let previous = self.info.properties.get(key).cloned();
        self.info.properties.insert(key.to_string(), value.to_string());
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::PropertyChanged {
            key: key.to_string(),
            previous,
            current: Some(value.to_string()),
        });
        
        Ok(())
    }
    
    fn register_event_handler(&mut self, handler: DeviceEventHandler) -> DeviceResult<()> {
        self.event_handlers.push(handler);
        Ok(())
    }
    
    fn unregister_event_handlers(&mut self) -> DeviceResult<()> {
        self.event_handlers.clear();
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn Device> {
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            available_sample_rates: self.available_sample_rates.clone(),
            available_channel_configs: self.available_channel_configs.clone(),
            available_formats: self.available_formats.clone(),
            spatial_audio_config: self.spatial_audio_config.clone(),
            volume: self.volume,
            muted: self.muted,
            power_consumption: self.power_consumption,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

impl AudioDevice for VRHeadphone {
    fn get_config(&self) -> DeviceResult<AudioConfig> {
        Ok(self.config.clone())
    }
    
    fn set_config(&mut self, config: &AudioConfig) -> DeviceResult<()> {
        // Validate sample rate
        if !self.available_sample_rates.contains(&config.sample_rate) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported sample rate: {} Hz",
                config.sample_rate.rate
            )));
        }
        
        // Validate channel configuration
        if !self.available_channel_configs.contains(&config.channel_config) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported channel configuration: {:?}",
                config.channel_config
            )));
        }
        
        // Validate audio format
        if !self.available_formats.contains(&config.format) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported audio format: {:?}",
                config.format
            )));
        }
        
        // Apply configuration
        self.config = config.clone();
        
        // Update properties
        self.info.properties.insert(
            "sample_rate".to_string(),
            format!("{} Hz", config.sample_rate.rate),
        );
        self.info.properties.insert(
            "channel_config".to_string(),
            format!("{:?}", config.channel_config),
        );
        self.info.properties.insert(
            "format".to_string(),
            format!("{:?}", config.format),
        );
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ConfigChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("sample_rate".to_string(), format!("{} Hz", config.sample_rate.rate));
                data.insert("channel_config".to_string(), format!("{:?}", config.channel_config));
                data.insert("format".to_string(), format!("{:?}", config.format));
                data
            },
        });
        
        Ok(())
    }
    
    fn get_available_sample_rates(&self) -> DeviceResult<Vec<AudioSampleRate>> {
        Ok(self.available_sample_rates.clone())
    }
    
    fn get_available_channel_configs(&self) -> DeviceResult<Vec<AudioChannelConfig>> {
        Ok(self.available_channel_configs.clone())
    }
    
    fn get_available_formats(&self) -> DeviceResult<Vec<AudioFormat>> {
        Ok(self.available_formats.clone())
    }
    
    fn set_sample_rate(&mut self, sample_rate: AudioSampleRate) -> DeviceResult<()> {
        // Validate sample rate
        if !self.available_sample_rates.contains(&sample_rate) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported sample rate: {} Hz",
                sample_rate.rate
            )));
        }
        
        // Update configuration
        self.config.sample_rate = sample_rate;
        
        // Update properties
        self.info.properties.insert(
            "sample_rate".to_string(),
            format!("{} Hz", sample_rate.rate),
        );
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "SampleRateChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("rate".to_string(), sample_rate.rate.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_channel_config(&mut self, channel_config: AudioChannelConfig) -> DeviceResult<()> {
        // Validate channel configuration
        if !self.available_channel_configs.contains(&channel_config) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported channel configuration: {:?}",
                channel_config
            )));
        }
        
        // Update configuration
        self.config.channel_config = channel_config;
        
        // Update properties
        self.info.properties.insert(
            "channel_config".to_string(),
            format!("{:?}", channel_config),
        );
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ChannelConfigChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("channel_config".to_string(), format!("{:?}", channel_config));
                data
            },
        });
        
        Ok(())
    }
    
    fn set_format(&mut self, format: AudioFormat) -> DeviceResult<()> {
        // Validate audio format
        if !self.available_formats.contains(&format) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported audio format: {:?}",
                format
            )));
        }
        
        // Update configuration
        self.config.format = format;
        
        // Update properties
        self.info.properties.insert(
            "format".to_string(),
            format!("{:?}", format),
        );
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "FormatChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("format".to_string(), format!("{:?}", format));
                data
            },
        });
        
        Ok(())
    }
    
    fn get_power_consumption(&self) -> DeviceResult<f32> {
        Ok(self.power_consumption)
    }
    
    fn has_audio_capability(&self, capability: AudioCapability) -> DeviceResult<bool> {
        match capability {
            AudioCapability::Output => Ok(true),
            AudioCapability::Input => Ok(false),
            AudioCapability::SpatialAudio => Ok(true),
            AudioCapability::NoiseReduction => Ok(false),
            AudioCapability::EchoCancellation => Ok(false),
            AudioCapability::VoiceRecognition => Ok(false),
            AudioCapability::BeamForming => Ok(false),
        }
    }
    
    fn clone_audio_box(&self) -> Box<dyn AudioDevice> {
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            available_sample_rates: self.available_sample_rates.clone(),
            available_channel_configs: self.available_channel_configs.clone(),
            available_formats: self.available_formats.clone(),
            spatial_audio_config: self.spatial_audio_config.clone(),
            volume: self.volume,
            muted: self.muted,
            power_consumption: self.power_consumption,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

impl AudioOutputDevice for VRHeadphone {
    fn get_volume(&self) -> DeviceResult<f32> {
        Ok(self.volume)
    }
    
    fn set_volume(&mut self, volume: f32) -> DeviceResult<()> {
        // Validate volume
        if !(0.0..=1.0).contains(&volume) {
            return Err(DeviceError::InvalidParameter(format!(
                "Volume must be between 0.0 and 1.0: {}",
                volume
            )));
        }
        
        // Update volume
        self.volume = volume;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "VolumeChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("volume".to_string(), volume.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn is_muted(&self) -> DeviceResult<bool> {
        Ok(self.muted)
    }
    
    fn set_muted(&mut self, muted: bool) -> DeviceResult<()> {
        // Update muted state
        self.muted = muted;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "MuteChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("muted".to_string(), muted.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn get_spatial_audio_config(&self) -> DeviceResult<SpatialAudioConfig> {
        Ok(self.spatial_audio_config.clone())
    }
    
    fn set_spatial_audio_config(&mut self, config: &SpatialAudioConfig) -> DeviceResult<()> {
        // Update configuration
        self.spatial_audio_config = config.clone();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "SpatialAudioConfigChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("enabled".to_string(), config.enabled.to_string());
                data.insert("quality".to_string(), format!("{:?}", config.quality));
                data
            },
        });
        
        Ok(())
    }
    
    fn enable_spatial_audio(&mut self, enabled: bool) -> DeviceResult<()> {
        // Update configuration
        self.spatial_audio_config.enabled = enabled;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "SpatialAudioEnabledChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("enabled".to_string(), enabled.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn play_test_sound(&mut self, sound_type: &str) -> DeviceResult<()> {
        info!("Playing test sound '{}' on VR Headphone: {}", sound_type, self.info.id);
        
        // Validate sound type
        match sound_type {
            "sine_wave" | "white_noise" | "pink_noise" | "voice" | "spatial" => {
                // Dispatch event
                self.dispatch_event(DeviceEventType::Custom {
                    name: "TestSoundStarted".to_string(),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("sound_type".to_string(), sound_type.to_string());
                        data
                    },
                });
                
                // Simulate test sound delay
                std::thread::sleep(Duration::from_millis(500));
                
                // Dispatch event
                self.dispatch_event(DeviceEventType::Custom {
                    name: "TestSoundCompleted".to_string(),
                    data: {
                        let mut data = HashMap::new();
                        data.insert("sound_type".to_string(), sound_type.to_string());
                        data
                    },
                });
                
                Ok(())
            },
            _ => Err(DeviceError::InvalidParameter(format!(
                "Unsupported test sound: {}",
                sound_type
            ))),
        }
    }
    
    fn clone_output_box(&self) -> Box<dyn AudioOutputDevice> {
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            available_sample_rates: self.available_sample_rates.clone(),
            available_channel_configs: self.available_channel_configs.clone(),
            available_formats: self.available_formats.clone(),
            spatial_audio_config: self.spatial_audio_config.clone(),
            volume: self.volume,
            muted: self.muted,
            power_consumption: self.power_consumption,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

/// VR Microphone device implementation.
#[derive(Debug)]
pub struct VRMicrophone {
    /// Device information
    info: DeviceInfo,
    
    /// Audio configuration
    config: AudioConfig,
    
    /// Available sample rates
    available_sample_rates: Vec<AudioSampleRate>,
    
    /// Available channel configurations
    available_channel_configs: Vec<AudioChannelConfig>,
    
    /// Available audio formats
    available_formats: Vec<AudioFormat>,
    
    /// Gain level (0.0 - 1.0)
    gain: f32,
    
    /// Muted state
    muted: bool,
    
    /// Noise reduction enabled
    noise_reduction: bool,
    
    /// Echo cancellation enabled
    echo_cancellation: bool,
    
    /// Beam forming enabled
    beam_forming: bool,
    
    /// Power consumption in watts
    power_consumption: f32,
    
    /// Event handlers
    event_handlers: Vec<DeviceEventHandler>,
    
    /// Last update time
    last_update: Instant,
}

impl VRMicrophone {
    /// Create a new VRMicrophone.
    pub fn new(
        id: String,
        name: String,
        manufacturer: String,
        model: String,
    ) -> Self {
        let now = chrono::Utc::now();
        
        // Create device info
        let mut info = DeviceInfo {
            id,
            name,
            device_type: DeviceType::Audio,
            manufacturer,
            model,
            serial_number: None,
            firmware_version: None,
            driver_version: None,
            bus_type: DeviceBus::USB,
            bus_address: None,
            capabilities: vec![
                DeviceCapability::PowerControl,
                DeviceCapability::Configuration,
                DeviceCapability::Statistics,
                DeviceCapability::PowerManagement,
                DeviceCapability::AudioInput,
            ],
            state: DeviceState::Connected,
            description: Some("VR Microphone".to_string()),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        // Add audio-specific properties
        info.properties.insert("audio_type".to_string(), "microphone".to_string());
        info.properties.insert("audio_channels".to_string(), "mono".to_string());
        
        // Create available sample rates
        let available_sample_rates = vec![
            AudioSampleRate::new(16000),
            AudioSampleRate::new(44100),
            AudioSampleRate::new(48000),
        ];
        
        // Create available channel configurations
        let available_channel_configs = vec![
            AudioChannelConfig::Mono,
            AudioChannelConfig::Stereo,
        ];
        
        // Create available audio formats
        let available_formats = vec![
            AudioFormat::PCM16,
            AudioFormat::PCM24,
            AudioFormat::PCM32Float,
        ];
        
        // Create audio configuration
        let config = AudioConfig::new(
            available_sample_rates[2], // 48000 Hz
            available_channel_configs[0], // Mono
            available_formats[0], // PCM16
        );
        
        Self {
            info,
            config,
            available_sample_rates,
            available_channel_configs,
            available_formats,
            gain: 0.7,
            muted: false,
            noise_reduction: true,
            echo_cancellation: true,
            beam_forming: true,
            power_consumption: 0.3,
            event_handlers: Vec::new(),
            last_update: Instant::now(),
        }
    }
    
    /// Update the device status.
    fn update_status(&mut self) {
        // Simulate power consumption changes based on gain and time
        let elapsed = self.last_update.elapsed().as_secs_f32();
        let gain_factor = if self.muted { 0.1 } else { self.gain };
        
        // Power consumption varies with gain and features
        let feature_factor = 0.1 * (
            (if self.noise_reduction { 1.0 } else { 0.0 }) +
            (if self.echo_cancellation { 1.0 } else { 0.0 }) +
            (if self.beam_forming { 1.0 } else { 0.0 })
        );
        
        self.power_consumption = 0.1 + (gain_factor * 0.3) + feature_factor;
        
        self.last_update = Instant::now();
    }
    
    /// Dispatch an event to all registered handlers.
    fn dispatch_event(&self, event_type: DeviceEventType) {
        let event = DeviceEvent::new(self.info.id.clone(), event_type);
        
        for handler in &self.event_handlers {
            handler(&event);
        }
    }
}

impl Device for VRMicrophone {
    fn info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.clone())
    }
    
    fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing VR Microphone: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::Initializing;
        
        // Simulate initialization delay
        std::thread::sleep(Duration::from_millis(100));
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Initialized);
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> DeviceResult<()> {
        info!("Shutting down VR Microphone: {}", self.info.id);
        
        // Update state
        self.info.state = DeviceState::ShuttingDown;
        
        // Simulate shutdown delay
        std::thread::sleep(Duration::from_millis(50));
        
        // Update state
        self.info.state = DeviceState::Disconnected;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Shutdown);
        
        Ok(())
    }
    
    fn reset(&mut self) -> DeviceResult<()> {
        info!("Resetting VR Microphone: {}", self.info.id);
        
        // Update state
        let previous_state = self.info.state;
        self.info.state = DeviceState::Initializing;
        
        // Simulate reset delay
        std::thread::sleep(Duration::from_millis(200));
        
        // Reset configuration to defaults
        self.config = AudioConfig::new(
            self.available_sample_rates[2], // 48000 Hz
            self.available_channel_configs[0], // Mono
            self.available_formats[0], // PCM16
        );
        
        self.gain = 0.7;
        self.muted = false;
        self.noise_reduction = true;
        self.echo_cancellation = true;
        self.beam_forming = true;
        
        // Update state
        self.info.state = DeviceState::Ready;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Reset);
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: DeviceState::Ready,
        });
        
        Ok(())
    }
    
    fn is_connected(&self) -> DeviceResult<bool> {
        Ok(self.info.state != DeviceState::Disconnected)
    }
    
    fn state(&self) -> DeviceResult<DeviceState> {
        Ok(self.info.state)
    }
    
    fn set_state(&mut self, state: DeviceState) -> DeviceResult<()> {
        let previous_state = self.info.state;
        self.info.state = state;
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: state,
        });
        
        Ok(())
    }
    
    fn has_capability(&self, capability: DeviceCapability) -> DeviceResult<bool> {
        Ok(self.info.capabilities.contains(&capability))
    }
    
    fn properties(&self) -> DeviceResult<HashMap<String, String>> {
        Ok(self.info.properties.clone())
    }
    
    fn get_property(&self, key: &str) -> DeviceResult<Option<String>> {
        Ok(self.info.properties.get(key).cloned())
    }
    
    fn set_property(&mut self, key: &str, value: &str) -> DeviceResult<()> {
        let previous = self.info.properties.get(key).cloned();
        self.info.properties.insert(key.to_string(), value.to_string());
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::PropertyChanged {
            key: key.to_string(),
            previous,
            current: Some(value.to_string()),
        });
        
        Ok(())
    }
    
    fn register_event_handler(&mut self, handler: DeviceEventHandler) -> DeviceResult<()> {
        self.event_handlers.push(handler);
        Ok(())
    }
    
    fn unregister_event_handlers(&mut self) -> DeviceResult<()> {
        self.event_handlers.clear();
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn Device> {
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            available_sample_rates: self.available_sample_rates.clone(),
            available_channel_configs: self.available_channel_configs.clone(),
            available_formats: self.available_formats.clone(),
            gain: self.gain,
            muted: self.muted,
            noise_reduction: self.noise_reduction,
            echo_cancellation: self.echo_cancellation,
            beam_forming: self.beam_forming,
            power_consumption: self.power_consumption,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

impl AudioDevice for VRMicrophone {
    fn get_config(&self) -> DeviceResult<AudioConfig> {
        Ok(self.config.clone())
    }
    
    fn set_config(&mut self, config: &AudioConfig) -> DeviceResult<()> {
        // Validate sample rate
        if !self.available_sample_rates.contains(&config.sample_rate) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported sample rate: {} Hz",
                config.sample_rate.rate
            )));
        }
        
        // Validate channel configuration
        if !self.available_channel_configs.contains(&config.channel_config) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported channel configuration: {:?}",
                config.channel_config
            )));
        }
        
        // Validate audio format
        if !self.available_formats.contains(&config.format) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported audio format: {:?}",
                config.format
            )));
        }
        
        // Apply configuration
        self.config = config.clone();
        
        // Update properties
        self.info.properties.insert(
            "sample_rate".to_string(),
            format!("{} Hz", config.sample_rate.rate),
        );
        self.info.properties.insert(
            "channel_config".to_string(),
            format!("{:?}", config.channel_config),
        );
        self.info.properties.insert(
            "format".to_string(),
            format!("{:?}", config.format),
        );
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ConfigChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("sample_rate".to_string(), format!("{} Hz", config.sample_rate.rate));
                data.insert("channel_config".to_string(), format!("{:?}", config.channel_config));
                data.insert("format".to_string(), format!("{:?}", config.format));
                data
            },
        });
        
        Ok(())
    }
    
    fn get_available_sample_rates(&self) -> DeviceResult<Vec<AudioSampleRate>> {
        Ok(self.available_sample_rates.clone())
    }
    
    fn get_available_channel_configs(&self) -> DeviceResult<Vec<AudioChannelConfig>> {
        Ok(self.available_channel_configs.clone())
    }
    
    fn get_available_formats(&self) -> DeviceResult<Vec<AudioFormat>> {
        Ok(self.available_formats.clone())
    }
    
    fn set_sample_rate(&mut self, sample_rate: AudioSampleRate) -> DeviceResult<()> {
        // Validate sample rate
        if !self.available_sample_rates.contains(&sample_rate) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported sample rate: {} Hz",
                sample_rate.rate
            )));
        }
        
        // Update configuration
        self.config.sample_rate = sample_rate;
        
        // Update properties
        self.info.properties.insert(
            "sample_rate".to_string(),
            format!("{} Hz", sample_rate.rate),
        );
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "SampleRateChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("rate".to_string(), sample_rate.rate.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn set_channel_config(&mut self, channel_config: AudioChannelConfig) -> DeviceResult<()> {
        // Validate channel configuration
        if !self.available_channel_configs.contains(&channel_config) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported channel configuration: {:?}",
                channel_config
            )));
        }
        
        // Update configuration
        self.config.channel_config = channel_config;
        
        // Update properties
        self.info.properties.insert(
            "channel_config".to_string(),
            format!("{:?}", channel_config),
        );
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "ChannelConfigChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("channel_config".to_string(), format!("{:?}", channel_config));
                data
            },
        });
        
        Ok(())
    }
    
    fn set_format(&mut self, format: AudioFormat) -> DeviceResult<()> {
        // Validate audio format
        if !self.available_formats.contains(&format) {
            return Err(DeviceError::InvalidParameter(format!(
                "Unsupported audio format: {:?}",
                format
            )));
        }
        
        // Update configuration
        self.config.format = format;
        
        // Update properties
        self.info.properties.insert(
            "format".to_string(),
            format!("{:?}", format),
        );
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "FormatChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("format".to_string(), format!("{:?}", format));
                data
            },
        });
        
        Ok(())
    }
    
    fn get_power_consumption(&self) -> DeviceResult<f32> {
        Ok(self.power_consumption)
    }
    
    fn has_audio_capability(&self, capability: AudioCapability) -> DeviceResult<bool> {
        match capability {
            AudioCapability::Output => Ok(false),
            AudioCapability::Input => Ok(true),
            AudioCapability::SpatialAudio => Ok(false),
            AudioCapability::NoiseReduction => Ok(true),
            AudioCapability::EchoCancellation => Ok(true),
            AudioCapability::VoiceRecognition => Ok(true),
            AudioCapability::BeamForming => Ok(true),
        }
    }
    
    fn clone_audio_box(&self) -> Box<dyn AudioDevice> {
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            available_sample_rates: self.available_sample_rates.clone(),
            available_channel_configs: self.available_channel_configs.clone(),
            available_formats: self.available_formats.clone(),
            gain: self.gain,
            muted: self.muted,
            noise_reduction: self.noise_reduction,
            echo_cancellation: self.echo_cancellation,
            beam_forming: self.beam_forming,
            power_consumption: self.power_consumption,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

impl AudioInputDevice for VRMicrophone {
    fn get_gain(&self) -> DeviceResult<f32> {
        Ok(self.gain)
    }
    
    fn set_gain(&mut self, gain: f32) -> DeviceResult<()> {
        // Validate gain
        if !(0.0..=1.0).contains(&gain) {
            return Err(DeviceError::InvalidParameter(format!(
                "Gain must be between 0.0 and 1.0: {}",
                gain
            )));
        }
        
        // Update gain
        self.gain = gain;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "GainChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("gain".to_string(), gain.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn is_muted(&self) -> DeviceResult<bool> {
        Ok(self.muted)
    }
    
    fn set_muted(&mut self, muted: bool) -> DeviceResult<()> {
        // Update muted state
        self.muted = muted;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "MuteChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("muted".to_string(), muted.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn is_noise_reduction_enabled(&self) -> DeviceResult<bool> {
        Ok(self.noise_reduction)
    }
    
    fn set_noise_reduction_enabled(&mut self, enabled: bool) -> DeviceResult<()> {
        // Update noise reduction state
        self.noise_reduction = enabled;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "NoiseReductionChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("enabled".to_string(), enabled.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn is_echo_cancellation_enabled(&self) -> DeviceResult<bool> {
        Ok(self.echo_cancellation)
    }
    
    fn set_echo_cancellation_enabled(&mut self, enabled: bool) -> DeviceResult<()> {
        // Update echo cancellation state
        self.echo_cancellation = enabled;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "EchoCancellationChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("enabled".to_string(), enabled.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn is_beam_forming_enabled(&self) -> DeviceResult<bool> {
        Ok(self.beam_forming)
    }
    
    fn set_beam_forming_enabled(&mut self, enabled: bool) -> DeviceResult<()> {
        // Update beam forming state
        self.beam_forming = enabled;
        
        // Update status
        self.update_status();
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::Custom {
            name: "BeamFormingChanged".to_string(),
            data: {
                let mut data = HashMap::new();
                data.insert("enabled".to_string(), enabled.to_string());
                data
            },
        });
        
        Ok(())
    }
    
    fn get_input_level(&self) -> DeviceResult<f32> {
        // Simulate input level based on gain and muted state
        if self.muted {
            Ok(0.0)
        } else {
            // Generate a random input level between 0.0 and gain
            let random_factor = rand::random::<f32>();
            Ok(self.gain * random_factor)
        }
    }
    
    fn start_recording(&mut self) -> DeviceResult<()> {
        info!("Starting recording on VR Microphone: {}", self.info.id);
        
        // Check if device is ready
        if self.info.state != DeviceState::Ready {
            return Err(DeviceError::InvalidState(format!(
                "Device is not ready: {:?}",
                self.info.state
            )));
        }
        
        // Update state
        let previous_state = self.info.state;
        self.info.state = DeviceState::Active;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: DeviceState::Active,
        });
        self.dispatch_event(DeviceEventType::Custom {
            name: "RecordingStarted".to_string(),
            data: HashMap::new(),
        });
        
        Ok(())
    }
    
    fn stop_recording(&mut self) -> DeviceResult<()> {
        info!("Stopping recording on VR Microphone: {}", self.info.id);
        
        // Check if device is active
        if self.info.state != DeviceState::Active {
            return Err(DeviceError::InvalidState(format!(
                "Device is not active: {:?}",
                self.info.state
            )));
        }
        
        // Update state
        let previous_state = self.info.state;
        self.info.state = DeviceState::Ready;
        
        // Update timestamp
        self.info.updated_at = chrono::Utc::now();
        
        // Dispatch event
        self.dispatch_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: DeviceState::Ready,
        });
        self.dispatch_event(DeviceEventType::Custom {
            name: "RecordingStopped".to_string(),
            data: HashMap::new(),
        });
        
        Ok(())
    }
    
    fn is_recording(&self) -> DeviceResult<bool> {
        Ok(self.info.state == DeviceState::Active)
    }
    
    fn clone_input_box(&self) -> Box<dyn AudioInputDevice> {
        Box::new(Self {
            info: self.info.clone(),
            config: self.config.clone(),
            available_sample_rates: self.available_sample_rates.clone(),
            available_channel_configs: self.available_channel_configs.clone(),
            available_formats: self.available_formats.clone(),
            gain: self.gain,
            muted: self.muted,
            noise_reduction: self.noise_reduction,
            echo_cancellation: self.echo_cancellation,
            beam_forming: self.beam_forming,
            power_consumption: self.power_consumption,
            event_handlers: Vec::new(), // Event handlers are not cloned
            last_update: self.last_update,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    
    #[test]
    fn test_headphone_creation() {
        let headphone = VRHeadphone::new(
            "headphone1".to_string(),
            "VR Headphone".to_string(),
            "Test Manufacturer".to_string(),
            "HP-VR-2000".to_string(),
        );
        
        let info = headphone.info().unwrap();
        assert_eq!(info.id, "headphone1");
        assert_eq!(info.name, "VR Headphone");
        assert_eq!(info.device_type, DeviceType::Audio);
        assert_eq!(info.manufacturer, "Test Manufacturer");
        assert_eq!(info.model, "HP-VR-2000");
        assert_eq!(info.bus_type, DeviceBus::USB);
        assert_eq!(info.state, DeviceState::Connected);
        
        let config = headphone.get_config().unwrap();
        assert_eq!(config.sample_rate.rate, 48000);
        assert_eq!(config.channel_config, AudioChannelConfig::Stereo);
        assert_eq!(config.format, AudioFormat::PCM16);
    }
    
    #[test]
    fn test_microphone_creation() {
        let microphone = VRMicrophone::new(
            "mic1".to_string(),
            "VR Microphone".to_string(),
            "Test Manufacturer".to_string(),
            "MIC-VR-3000".to_string(),
        );
        
        let info = microphone.info().unwrap();
        assert_eq!(info.id, "mic1");
        assert_eq!(info.name, "VR Microphone");
        assert_eq!(info.device_type, DeviceType::Audio);
        assert_eq!(info.manufacturer, "Test Manufacturer");
        assert_eq!(info.model, "MIC-VR-3000");
        assert_eq!(info.bus_type, DeviceBus::USB);
        assert_eq!(info.state, DeviceState::Connected);
        
        let config = microphone.get_config().unwrap();
        assert_eq!(config.sample_rate.rate, 48000);
        assert_eq!(config.channel_config, AudioChannelConfig::Mono);
        assert_eq!(config.format, AudioFormat::PCM16);
    }
    
    #[test]
    fn test_headphone_volume() {
        let mut headphone = VRHeadphone::new(
            "headphone1".to_string(),
            "VR Headphone".to_string(),
            "Test Manufacturer".to_string(),
            "HP-VR-2000".to_string(),
        );
        
        // Test getting volume
        assert_eq!(headphone.get_volume().unwrap(), 0.8);
        
        // Test setting volume
        headphone.set_volume(0.5).unwrap();
        assert_eq!(headphone.get_volume().unwrap(), 0.5);
        
        // Test setting invalid volume
        assert!(headphone.set_volume(1.5).is_err());
        
        // Test muting
        assert_eq!(headphone.is_muted().unwrap(), false);
        headphone.set_muted(true).unwrap();
        assert_eq!(headphone.is_muted().unwrap(), true);
    }
    
    #[test]
    fn test_microphone_gain() {
        let mut microphone = VRMicrophone::new(
            "mic1".to_string(),
            "VR Microphone".to_string(),
            "Test Manufacturer".to_string(),
            "MIC-VR-3000".to_string(),
        );
        
        // Test getting gain
        assert_eq!(microphone.get_gain().unwrap(), 0.7);
        
        // Test setting gain
        microphone.set_gain(0.5).unwrap();
        assert_eq!(microphone.get_gain().unwrap(), 0.5);
        
        // Test setting invalid gain
        assert!(microphone.set_gain(1.5).is_err());
        
        // Test muting
        assert_eq!(microphone.is_muted().unwrap(), false);
        microphone.set_muted(true).unwrap();
        assert_eq!(microphone.is_muted().unwrap(), true);
    }
    
    #[test]
    fn test_microphone_features() {
        let mut microphone = VRMicrophone::new(
            "mic1".to_string(),
            "VR Microphone".to_string(),
            "Test Manufacturer".to_string(),
            "MIC-VR-3000".to_string(),
        );
        
        // Test noise reduction
        assert_eq!(microphone.is_noise_reduction_enabled().unwrap(), true);
        microphone.set_noise_reduction_enabled(false).unwrap();
        assert_eq!(microphone.is_noise_reduction_enabled().unwrap(), false);
        
        // Test echo cancellation
        assert_eq!(microphone.is_echo_cancellation_enabled().unwrap(), true);
        microphone.set_echo_cancellation_enabled(false).unwrap();
        assert_eq!(microphone.is_echo_cancellation_enabled().unwrap(), false);
        
        // Test beam forming
        assert_eq!(microphone.is_beam_forming_enabled().unwrap(), true);
        microphone.set_beam_forming_enabled(false).unwrap();
        assert_eq!(microphone.is_beam_forming_enabled().unwrap(), false);
    }
    
    #[test]
    fn test_microphone_recording() {
        let mut microphone = VRMicrophone::new(
            "mic1".to_string(),
            "VR Microphone".to_string(),
            "Test Manufacturer".to_string(),
            "MIC-VR-3000".to_string(),
        );
        
        // Initialize the microphone
        microphone.initialize().unwrap();
        
        // Test recording state
        assert_eq!(microphone.is_recording().unwrap(), false);
        
        // Start recording
        microphone.start_recording().unwrap();
        assert_eq!(microphone.is_recording().unwrap(), true);
        assert_eq!(microphone.state().unwrap(), DeviceState::Active);
        
        // Stop recording
        microphone.stop_recording().unwrap();
        assert_eq!(microphone.is_recording().unwrap(), false);
        assert_eq!(microphone.state().unwrap(), DeviceState::Ready);
    }
}

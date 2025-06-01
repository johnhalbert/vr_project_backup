//! Audio device interface for the VR headset.
//!
//! This module provides the implementation of audio devices for the VR headset,
//! including management of headphones, microphone array, and spatial audio processing.

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::device::{
    Device, DeviceCapability, DeviceError, DeviceEventHandler, DeviceEventType,
    DeviceInfo, DeviceResult, DeviceState, DeviceType, DeviceBus,
};

/// Audio device capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioCapability {
    /// Spatial audio processing
    SpatialAudio,
    
    /// Beamforming for microphone array
    Beamforming,
    
    /// Noise cancellation
    NoiseCancellation,
    
    /// Voice activity detection
    VoiceActivityDetection,
    
    /// Echo cancellation
    EchoCancellation,
    
    /// Automatic gain control
    AutomaticGainControl,
    
    /// Equalizer
    Equalizer,
    
    /// Virtual surround sound
    VirtualSurround,
    
    /// Bass boost
    BassBoost,
    
    /// Treble boost
    TrebleBoost,
    
    /// Voice enhancement
    VoiceEnhancement,
    
    /// Custom capability
    Custom(u32),
}

/// Audio device type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioDeviceType {
    /// Headphones
    Headphones,
    
    /// Microphone
    Microphone,
    
    /// Microphone array
    MicrophoneArray,
    
    /// Speaker
    Speaker,
    
    /// Combined headset (headphones + microphone)
    Headset,
    
    /// Virtual device
    Virtual,
    
    /// Other audio device type
    Other(String),
}

/// Audio format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AudioFormat {
    /// Sample rate in Hz
    pub sample_rate: u32,
    
    /// Bit depth
    pub bit_depth: u8,
    
    /// Number of channels
    pub channels: u8,
    
    /// Whether the format is floating point
    pub is_float: bool,
}

impl AudioFormat {
    /// Create a new AudioFormat.
    pub fn new(sample_rate: u32, bit_depth: u8, channels: u8, is_float: bool) -> Self {
        Self {
            sample_rate,
            bit_depth,
            channels,
            is_float,
        }
    }
    
    /// Standard CD quality format (44.1kHz, 16-bit, 2 channels).
    pub fn cd_quality() -> Self {
        Self {
            sample_rate: 44100,
            bit_depth: 16,
            channels: 2,
            is_float: false,
        }
    }
    
    /// High-resolution audio format (96kHz, 24-bit, 2 channels).
    pub fn high_res() -> Self {
        Self {
            sample_rate: 96000,
            bit_depth: 24,
            channels: 2,
            is_float: false,
        }
    }
    
    /// Studio quality audio format (192kHz, 32-bit float, 2 channels).
    pub fn studio_quality() -> Self {
        Self {
            sample_rate: 192000,
            bit_depth: 32,
            channels: 2,
            is_float: true,
        }
    }
    
    /// VR optimized audio format (48kHz, 16-bit, 2 channels).
    pub fn vr_optimized() -> Self {
        Self {
            sample_rate: 48000,
            bit_depth: 16,
            channels: 2,
            is_float: false,
        }
    }
    
    /// Microphone array format (48kHz, 16-bit, 4 channels).
    pub fn microphone_array() -> Self {
        Self {
            sample_rate: 48000,
            bit_depth: 16,
            channels: 4,
            is_float: false,
        }
    }
    
    /// Calculate the byte rate (bytes per second).
    pub fn byte_rate(&self) -> u32 {
        self.sample_rate * (self.bit_depth as u32 / 8) * (self.channels as u32)
    }
    
    /// Calculate the frame size (bytes per frame).
    pub fn frame_size(&self) -> u32 {
        (self.bit_depth as u32 / 8) * (self.channels as u32)
    }
}

/// Spatial audio mode.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpatialAudioMode {
    /// Stereo (no spatial processing)
    Stereo,
    
    /// Binaural rendering
    Binaural,
    
    /// Ambisonics (first order)
    AmbisonicsFirstOrder,
    
    /// Ambisonics (higher order)
    AmbisonicsHigherOrder,
    
    /// Virtual surround
    VirtualSurround,
    
    /// Custom spatial audio mode
    Custom(String),
}

/// Beamforming mode for microphone array.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BeamformingMode {
    /// Fixed beamforming
    Fixed,
    
    /// Adaptive beamforming
    Adaptive,
    
    /// Steered response power
    SteeredResponsePower,
    
    /// Minimum variance distortionless response
    MVDR,
    
    /// Linearly constrained minimum variance
    LCMV,
    
    /// Custom beamforming mode
    Custom(String),
}

/// Equalizer band.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EqualizerBand {
    /// Center frequency in Hz
    pub frequency: f32,
    
    /// Gain in dB
    pub gain: f32,
    
    /// Q factor
    pub q: f32,
}

impl EqualizerBand {
    /// Create a new EqualizerBand.
    pub fn new(frequency: f32, gain: f32, q: f32) -> Self {
        Self { frequency, gain, q }
    }
}

/// Audio configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Audio format
    pub format: AudioFormat,
    
    /// Volume level (0.0 - 1.0)
    pub volume: f32,
    
    /// Mute state
    pub muted: bool,
    
    /// Spatial audio mode
    pub spatial_audio_mode: Option<SpatialAudioMode>,
    
    /// Beamforming mode for microphone array
    pub beamforming_mode: Option<BeamformingMode>,
    
    /// Noise cancellation level (0.0 - 1.0)
    pub noise_cancellation_level: Option<f32>,
    
    /// Voice activity detection threshold (0.0 - 1.0)
    pub vad_threshold: Option<f32>,
    
    /// Echo cancellation enabled
    pub echo_cancellation_enabled: Option<bool>,
    
    /// Automatic gain control enabled
    pub agc_enabled: Option<bool>,
    
    /// Equalizer bands
    pub equalizer_bands: Option<Vec<EqualizerBand>>,
    
    /// Bass boost level (0.0 - 1.0)
    pub bass_boost: Option<f32>,
    
    /// Treble boost level (0.0 - 1.0)
    pub treble_boost: Option<f32>,
    
    /// Voice enhancement level (0.0 - 1.0)
    pub voice_enhancement: Option<f32>,
    
    /// Custom settings
    pub custom_settings: HashMap<String, String>,
}

impl AudioConfig {
    /// Create a new AudioConfig with default values.
    pub fn new(format: AudioFormat) -> Self {
        Self {
            format,
            volume: 0.8,
            muted: false,
            spatial_audio_mode: None,
            beamforming_mode: None,
            noise_cancellation_level: None,
            vad_threshold: None,
            echo_cancellation_enabled: None,
            agc_enabled: None,
            equalizer_bands: None,
            bass_boost: None,
            treble_boost: None,
            voice_enhancement: None,
            custom_settings: HashMap::new(),
        }
    }
    
    /// Create a new AudioConfig optimized for VR headphones.
    pub fn vr_headphones() -> Self {
        let mut config = Self::new(AudioFormat::vr_optimized());
        config.spatial_audio_mode = Some(SpatialAudioMode::Binaural);
        config.bass_boost = Some(0.3);
        config.treble_boost = Some(0.2);
        
        // Add a basic equalizer preset for VR
        let mut bands = Vec::new();
        bands.push(EqualizerBand::new(60.0, 3.0, 1.0));    // Bass
        bands.push(EqualizerBand::new(250.0, 1.5, 1.0));   // Low-mid
        bands.push(EqualizerBand::new(1000.0, 0.0, 1.0));  // Mid
        bands.push(EqualizerBand::new(4000.0, 2.0, 1.0));  // High-mid
        bands.push(EqualizerBand::new(12000.0, 1.0, 1.0)); // Treble
        config.equalizer_bands = Some(bands);
        
        config
    }
    
    /// Create a new AudioConfig optimized for VR microphone array.
    pub fn vr_microphone_array() -> Self {
        let mut config = Self::new(AudioFormat::microphone_array());
        config.beamforming_mode = Some(BeamformingMode::Adaptive);
        config.noise_cancellation_level = Some(0.7);
        config.vad_threshold = Some(0.2);
        config.echo_cancellation_enabled = Some(true);
        config.agc_enabled = Some(true);
        config.voice_enhancement = Some(0.5);
        
        config
    }
}

/// Audio device trait.
pub trait AudioDevice: Device {
    /// Get the audio device type.
    fn get_audio_device_type(&self) -> DeviceResult<AudioDeviceType>;
    
    /// Get the audio configuration.
    fn get_config(&self) -> DeviceResult<AudioConfig>;
    
    /// Set the audio configuration.
    fn set_config(&mut self, config: &AudioConfig) -> DeviceResult<()>;
    
    /// Get the available audio formats.
    fn get_available_formats(&self) -> DeviceResult<Vec<AudioFormat>>;
    
    /// Set the audio format.
    fn set_format(&mut self, format: AudioFormat) -> DeviceResult<()>;
    
    /// Set the volume level.
    fn set_volume(&mut self, volume: f32) -> DeviceResult<()>;
    
    /// Get the volume level.
    fn get_volume(&self) -> DeviceResult<f32>;
    
    /// Set the mute state.
    fn set_muted(&mut self, muted: bool) -> DeviceResult<()>;
    
    /// Get the mute state.
    fn is_muted(&self) -> DeviceResult<bool>;
    
    /// Set the spatial audio mode.
    fn set_spatial_audio_mode(&mut self, mode: SpatialAudioMode) -> DeviceResult<()>;
    
    /// Get the spatial audio mode.
    fn get_spatial_audio_mode(&self) -> DeviceResult<Option<SpatialAudioMode>>;
    
    /// Set the beamforming mode.
    fn set_beamforming_mode(&mut self, mode: BeamformingMode) -> DeviceResult<()>;
    
    /// Get the beamforming mode.
    fn get_beamforming_mode(&self) -> DeviceResult<Option<BeamformingMode>>;
    
    /// Set the noise cancellation level.
    fn set_noise_cancellation_level(&mut self, level: f32) -> DeviceResult<()>;
    
    /// Get the noise cancellation level.
    fn get_noise_cancellation_level(&self) -> DeviceResult<Option<f32>>;
    
    /// Set the voice activity detection threshold.
    fn set_vad_threshold(&mut self, threshold: f32) -> DeviceResult<()>;
    
    /// Get the voice activity detection threshold.
    fn get_vad_threshold(&self) -> DeviceResult<Option<f32>>;
    
    /// Enable or disable echo cancellation.
    fn set_echo_cancellation_enabled(&mut self, enabled: bool) -> DeviceResult<()>;
    
    /// Check if echo cancellation is enabled.
    fn is_echo_cancellation_enabled(&self) -> DeviceResult<Option<bool>>;
    
    /// Enable or disable automatic gain control.
    fn set_agc_enabled(&mut self, enabled: bool) -> DeviceResult<()>;
    
    /// Check if automatic gain control is enabled.
    fn is_agc_enabled(&self) -> DeviceResult<Option<bool>>;
    
    /// Set the equalizer bands.
    fn set_equalizer_bands(&mut self, bands: Vec<EqualizerBand>) -> DeviceResult<()>;
    
    /// Get the equalizer bands.
    fn get_equalizer_bands(&self) -> DeviceResult<Option<Vec<EqualizerBand>>>;
    
    /// Set the bass boost level.
    fn set_bass_boost(&mut self, level: f32) -> DeviceResult<()>;
    
    /// Get the bass boost level.
    fn get_bass_boost(&self) -> DeviceResult<Option<f32>>;
    
    /// Set the treble boost level.
    fn set_treble_boost(&mut self, level: f32) -> DeviceResult<()>;
    
    /// Get the treble boost level.
    fn get_treble_boost(&self) -> DeviceResult<Option<f32>>;
    
    /// Set the voice enhancement level.
    fn set_voice_enhancement(&mut self, level: f32) -> DeviceResult<()>;
    
    /// Get the voice enhancement level.
    fn get_voice_enhancement(&self) -> DeviceResult<Option<f32>>;
    
    /// Get the audio latency in milliseconds.
    fn get_latency(&self) -> DeviceResult<f32>;
    
    /// Get the signal-to-noise ratio in dB.
    fn get_snr(&self) -> DeviceResult<Option<f32>>;
    
    /// Get the total harmonic distortion in percentage.
    fn get_thd(&self) -> DeviceResult<Option<f32>>;
    
    /// Run an audio test.
    fn run_audio_test(&mut self, test_type: &str) -> DeviceResult<HashMap<String, String>>;
}

/// Audio manager for managing multiple audio devices.
#[derive(Debug)]
pub struct AudioManager {
    /// Audio devices by ID
    devices: HashMap<String, Arc<Mutex<Box<dyn AudioDevice>>>>,
    
    /// Default output device ID
    default_output_id: Option<String>,
    
    /// Default input device ID
    default_input_id: Option<String>,
    
    /// Global volume level (0.0 - 1.0)
    global_volume: f32,
    
    /// Global mute state
    global_muted: bool,
}

impl AudioManager {
    /// Create a new AudioManager.
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            default_output_id: None,
            default_input_id: None,
            global_volume: 0.8,
            global_muted: false,
        }
    }
    
    /// Initialize the audio manager.
    pub fn initialize(&mut self) -> DeviceResult<()> {
        info!("Initializing AudioManager");
        Ok(())
    }
    
    /// Shutdown the audio manager.
    pub fn shutdown(&mut self) -> DeviceResult<()> {
        info!("Shutting down AudioManager");
        
        // Shutdown all audio devices
        for (id, device) in &self.devices {
            info!("Shutting down audio device {}", id);
            let mut device = device.lock().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire lock on audio device".to_string())
            })?;
            
            if let Err(e) = device.shutdown() {
                warn!("Failed to shutdown audio device {}: {}", id, e);
            }
        }
        
        self.devices.clear();
        self.default_output_id = None;
        self.default_input_id = None;
        
        Ok(())
    }
    
    /// Register an audio device.
    pub fn register_device(
        &mut self,
        id: &str,
        device: Box<dyn AudioDevice>,
    ) -> DeviceResult<()> {
        info!("Registering audio device {}", id);
        
        // Get the audio device type to determine if it's an input or output device
        let device_type = {
            let audio_device_type = device.get_audio_device_type()?;
            match audio_device_type {
                AudioDeviceType::Headphones | AudioDeviceType::Speaker => "output",
                AudioDeviceType::Microphone | AudioDeviceType::MicrophoneArray => "input",
                AudioDeviceType::Headset => "both",
                _ => "unknown",
            }
        };
        
        let device = Arc::new(Mutex::new(device));
        self.devices.insert(id.to_string(), device);
        
        // Set as default if no default is set
        if device_type == "output" || device_type == "both" {
            if self.default_output_id.is_none() {
                self.set_default_output(id)?;
            }
        }
        
        if device_type == "input" || device_type == "both" {
            if self.default_input_id.is_none() {
                self.set_default_input(id)?;
            }
        }
        
        Ok(())
    }
    
    /// Unregister an audio device.
    pub fn unregister_device(&mut self, id: &str) -> DeviceResult<()> {
        info!("Unregistering audio device {}", id);
        
        if self.devices.remove(id).is_none() {
            return Err(DeviceError::NotFound(format!("Audio device {} not found", id)));
        }
        
        // Update default device IDs if necessary
        if Some(id.to_string()) == self.default_output_id {
            self.default_output_id = None;
            
            // Find a new default output device
            for (device_id, device) in &self.devices {
                let device = device.lock().map_err(|_| {
                    DeviceError::CommunicationError("Failed to acquire lock on audio device".to_string())
                })?;
                
                let audio_device_type = device.get_audio_device_type()?;
                if audio_device_type == AudioDeviceType::Headphones
                    || audio_device_type == AudioDeviceType::Speaker
                    || audio_device_type == AudioDeviceType::Headset
                {
                    self.default_output_id = Some(device_id.clone());
                    break;
                }
            }
        }
        
        if Some(id.to_string()) == self.default_input_id {
            self.default_input_id = None;
            
            // Find a new default input device
            for (device_id, device) in &self.devices {
                let device = device.lock().map_err(|_| {
                    DeviceError::CommunicationError("Failed to acquire lock on audio device".to_string())
                })?;
                
                let audio_device_type = device.get_audio_device_type()?;
                if audio_device_type == AudioDeviceType::Microphone
                    || audio_device_type == AudioDeviceType::MicrophoneArray
                    || audio_device_type == AudioDeviceType::Headset
                {
                    self.default_input_id = Some(device_id.clone());
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Get an audio device.
    pub fn get_device(&self, id: &str) -> DeviceResult<Arc<Mutex<Box<dyn AudioDevice>>>> {
        self.devices
            .get(id)
            .cloned()
            .ok_or_else(|| DeviceError::NotFound(format!("Audio device {} not found", id)))
    }
    
    /// Get all audio devices.
    pub fn get_all_devices(&self) -> HashMap<String, Arc<Mutex<Box<dyn AudioDevice>>>> {
        self.devices.clone()
    }
    
    /// Get all output devices.
    pub fn get_output_devices(&self) -> DeviceResult<HashMap<String, Arc<Mutex<Box<dyn AudioDevice>>>>> {
        let mut output_devices = HashMap::new();
        
        for (id, device) in &self.devices {
            let device_guard = device.lock().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire lock on audio device".to_string())
            })?;
            
            let audio_device_type = device_guard.get_audio_device_type()?;
            if audio_device_type == AudioDeviceType::Headphones
                || audio_device_type == AudioDeviceType::Speaker
                || audio_device_type == AudioDeviceType::Headset
            {
                output_devices.insert(id.clone(), device.clone());
            }
        }
        
        Ok(output_devices)
    }
    
    /// Get all input devices.
    pub fn get_input_devices(&self) -> DeviceResult<HashMap<String, Arc<Mutex<Box<dyn AudioDevice>>>>> {
        let mut input_devices = HashMap::new();
        
        for (id, device) in &self.devices {
            let device_guard = device.lock().map_err(|_| {
                DeviceError::CommunicationError("Failed to acquire lock on audio device".to_string())
            })?;
            
            let audio_device_type = device_guard.get_audio_device_type()?;
            if audio_device_type == AudioDeviceType::Microphone
                || audio_device_type == AudioDeviceType::MicrophoneArray
                || audio_device_type == AudioDeviceType::Headset
            {
                input_devices.insert(id.clone(), device.clone());
            }
        }
        
        Ok(input_devices)
    }
    
    /// Get the default output device.
    pub fn get_default_output(&self) -> DeviceResult<Arc<Mutex<Box<dyn AudioDevice>>>> {
        if let Some(id) = &self.default_output_id {
            self.get_device(id)
        } else {
            Err(DeviceError::NotFound("No default output device set".to_string()))
        }
    }
    
    /// Get the default input device.
    pub fn get_default_input(&self) -> DeviceResult<Arc<Mutex<Box<dyn AudioDevice>>>> {
        if let Some(id) = &self.default_input_id {
            self.get_device(id)
        } else {
            Err(DeviceError::NotFound("No default input device set".to_string()))
        }
    }
    
    /// Set the default output device.
    pub fn set_default_output(&mut self, id: &str) -> DeviceResult<()> {
        if !self.devices.contains_key(id) {
            return Err(DeviceError::NotFound(format!("Audio device {} not found", id)));
        }
        
        // Verify that the device is an output device
        let device = self.get_device(id)?;
        let device_guard = device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on audio device".to_string())
        })?;
        
        let audio_device_type = device_guard.get_audio_device_type()?;
        if audio_device_type != AudioDeviceType::Headphones
            && audio_device_type != AudioDeviceType::Speaker
            && audio_device_type != AudioDeviceType::Headset
        {
            return Err(DeviceError::InvalidParameter(format!(
                "Device {} is not an output device",
                id
            )));
        }
        
        info!("Setting {} as default output device", id);
        self.default_output_id = Some(id.to_string());
        Ok(())
    }
    
    /// Set the default input device.
    pub fn set_default_input(&mut self, id: &str) -> DeviceResult<()> {
        if !self.devices.contains_key(id) {
            return Err(DeviceError::NotFound(format!("Audio device {} not found", id)));
        }
        
        // Verify that the device is an input device
        let device = self.get_device(id)?;
        let device_guard = device.lock().map_err(|_| {
            DeviceError::CommunicationError("Failed to acquire lock on audio device".to_string())
        })?;
        
        let audio_device_type = device_guard.get_audio_device_type()?;
        if audio_device_type != AudioDeviceType::Microphone
            && audio_device_type != AudioDeviceType::MicrophoneArray
            && audio_device_type != AudioDeviceType::Headset
        {
            return Err(DeviceError::InvalidParameter(format!(
                "Device {} is not an input device",
                id
            )));
        }
        
        info!("Setting {} as default input device", id);
        self.default_input_id = Some(id.to_string());
        Ok(())
    }
    
    /// Set the global volume level.
    pub fn set_global_volume(&mut self, volume: f32) -> DeviceResult<()> {
        if volume < 0.0 || volume > 1.0 {
            return Err(DeviceError::InvalidParameter(format!(
                "Volume {} out of range (0.0 - 1.0)",
                volume
            )));
        }
        
        info!("Setting global volume to {}", volume);
        self.global_volume = volume;
        
        // Apply to all output devices
        let output_devices = self.get_output_devices()?;
        for (id, device) in output_devices {
            let mut device = device.lock().map_err(|_| {
                DeviceError::CommunicationError(format!("Failed to acquire lock on audio device {}", id))
            })?;
            
            if let Err(e) = device.set_volume(volume) {
                warn!("Failed to set volume for device {}: {}", id, e);
            }
        }
        
        Ok(())
    }
    
    /// Get the global volume level.
    pub fn get_global_volume(&self) -> f32 {
        self.global_volume
    }
    
    /// Set the global mute state.
    pub fn set_global_muted(&mut self, muted: bool) -> DeviceResult<()> {
        info!("Setting global mute to {}", muted);
        self.global_muted = muted;
        
        // Apply to all output devices
        let output_devices = self.get_output_devices()?;
        for (id, device) in output_devices {
            let mut device = device.lock().map_err(|_| {
                DeviceError::CommunicationError(format!("Failed to acquire lock on audio device {}", id))
            })?;
            
            if let Err(e) = device.set_muted(muted) {
                warn!("Failed to set mute for device {}: {}", id, e);
            }
        }
        
        Ok(())
    }
    
    /// Get the global mute state.
    pub fn is_global_muted(&self) -> bool {
        self.global_muted
    }
    
    /// Set the spatial audio mode for all output devices.
    pub fn set_spatial_audio_mode_all(&self, mode: SpatialAudioMode) -> DeviceResult<()> {
        info!("Setting spatial audio mode to {:?} for all output devices", mode);
        
        let output_devices = self.get_output_devices()?;
        for (id, device) in output_devices {
            let mut device = device.lock().map_err(|_| {
                DeviceError::CommunicationError(format!("Failed to acquire lock on audio device {}", id))
            })?;
            
            if let Err(e) = device.set_spatial_audio_mode(mode) {
                warn!("Failed to set spatial audio mode for device {}: {}", id, e);
            }
        }
        
        Ok(())
    }
    
    /// Set the beamforming mode for all input devices.
    pub fn set_beamforming_mode_all(&self, mode: BeamformingMode) -> DeviceResult<()> {
        info!("Setting beamforming mode to {:?} for all input devices", mode);
        
        let input_devices = self.get_input_devices()?;
        for (id, device) in input_devices {
            let mut device = device.lock().map_err(|_| {
                DeviceError::CommunicationError(format!("Failed to acquire lock on audio device {}", id))
            })?;
            
            if let Err(e) = device.set_beamforming_mode(mode) {
                warn!("Failed to set beamforming mode for device {}: {}", id, e);
            }
        }
        
        Ok(())
    }
    
    /// Set the noise cancellation level for all input devices.
    pub fn set_noise_cancellation_level_all(&self, level: f32) -> DeviceResult<()> {
        if level < 0.0 || level > 1.0 {
            return Err(DeviceError::InvalidParameter(format!(
                "Noise cancellation level {} out of range (0.0 - 1.0)",
                level
            )));
        }
        
        info!("Setting noise cancellation level to {} for all input devices", level);
        
        let input_devices = self.get_input_devices()?;
        for (id, device) in input_devices {
            let mut device = device.lock().map_err(|_| {
                DeviceError::CommunicationError(format!("Failed to acquire lock on audio device {}", id))
            })?;
            
            if let Err(e) = device.set_noise_cancellation_level(level) {
                warn!("Failed to set noise cancellation level for device {}: {}", id, e);
            }
        }
        
        Ok(())
    }
    
    /// Run an audio test on all devices.
    pub fn run_audio_test_all(&self, test_type: &str) -> DeviceResult<HashMap<String, HashMap<String, String>>> {
        info!("Running audio test {} on all devices", test_type);
        
        let mut results = HashMap::new();
        
        for (id, device) in &self.devices {
            let mut device = device.lock().map_err(|_| {
                DeviceError::CommunicationError(format!("Failed to acquire lock on audio device {}", id))
            })?;
            
            match device.run_audio_test(test_type) {
                Ok(test_results) => {
                    results.insert(id.clone(), test_results);
                }
                Err(e) => {
                    warn!("Failed to run audio test on device {}: {}", id, e);
                    let mut error_result = HashMap::new();
                    error_result.insert("error".to_string(), e.to_string());
                    results.insert(id.clone(), error_result);
                }
            }
        }
        
        Ok(results)
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock audio device for testing.
#[derive(Debug)]
pub struct MockAudioDevice {
    /// Device information
    info: DeviceInfo,
    
    /// Audio device type
    audio_device_type: AudioDeviceType,
    
    /// Audio configuration
    config: AudioConfig,
    
    /// Available audio formats
    available_formats: Vec<AudioFormat>,
    
    /// Audio latency in milliseconds
    latency: f32,
    
    /// Signal-to-noise ratio in dB
    snr: Option<f32>,
    
    /// Total harmonic distortion in percentage
    thd: Option<f32>,
    
    /// Event handlers
    event_handlers: Vec<DeviceEventHandler>,
    
    /// Device state
    state: DeviceState,
}

impl MockAudioDevice {
    /// Create a new MockAudioDevice.
    pub fn new(id: &str, name: &str, audio_device_type: AudioDeviceType) -> Self {
        let mut info = DeviceInfo::new(
            id.to_string(),
            name.to_string(),
            DeviceType::AudioInput,
            "Mock Manufacturer".to_string(),
            "Mock Audio Model".to_string(),
            DeviceBus::Internal,
        );
        
        info.add_capability(DeviceCapability::PowerControl);
        info.add_capability(DeviceCapability::Calibration);
        info.add_capability(DeviceCapability::Configuration);
        
        let available_formats = vec![
            AudioFormat::cd_quality(),
            AudioFormat::high_res(),
            AudioFormat::vr_optimized(),
        ];
        
        let config = match audio_device_type {
            AudioDeviceType::Headphones | AudioDeviceType::Speaker | AudioDeviceType::Headset => {
                AudioConfig::vr_headphones()
            }
            AudioDeviceType::Microphone | AudioDeviceType::MicrophoneArray => {
                AudioConfig::vr_microphone_array()
            }
            _ => AudioConfig::new(AudioFormat::vr_optimized()),
        };
        
        Self {
            info,
            audio_device_type,
            config,
            available_formats,
            latency: 10.0, // 10ms
            snr: Some(90.0), // 90dB
            thd: Some(0.01), // 0.01%
            event_handlers: Vec::new(),
            state: DeviceState::Connected,
        }
    }
    
    /// Emit an event to all registered handlers.
    fn emit_event(&self, event_type: DeviceEventType) {
        let event = super::device::DeviceEvent::new(self.info.id.clone(), event_type);
        for handler in &self.event_handlers {
            handler(&event);
        }
    }
    
    /// Update the device state.
    fn update_state(&mut self, new_state: DeviceState) {
        let previous_state = self.state;
        self.state = new_state;
        self.emit_event(DeviceEventType::StateChanged {
            previous: previous_state,
            current: new_state,
        });
    }
}

impl Device for MockAudioDevice {
    fn info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.clone())
    }
    
    fn initialize(&mut self) -> DeviceResult<()> {
        self.update_state(DeviceState::Initializing);
        
        // Simulate initialization
        self.update_state(DeviceState::Ready);
        self.emit_event(DeviceEventType::Initialized);
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> DeviceResult<()> {
        self.update_state(DeviceState::ShuttingDown);
        
        // Simulate shutdown
        self.update_state(DeviceState::Disconnected);
        self.emit_event(DeviceEventType::Shutdown);
        
        Ok(())
    }
    
    fn reset(&mut self) -> DeviceResult<()> {
        self.update_state(DeviceState::Initializing);
        self.emit_event(DeviceEventType::Reset);
        
        // Simulate reset
        self.update_state(DeviceState::Ready);
        
        Ok(())
    }
    
    fn is_connected(&self) -> DeviceResult<bool> {
        Ok(self.state != DeviceState::Disconnected)
    }
    
    fn clone_box(&self) -> Box<dyn Device> {
        Box::new(MockAudioDevice {
            id: self.id.clone(),
            info: self.info.clone(),
            state: self.state,
            config: self.config.clone(),
            event_handlers: Vec::new(), // Event handlers are not cloned
            capabilities: self.capabilities.clone(),
        })
    }
    
    fn state(&self) -> DeviceResult<DeviceState> {
        Ok(self.state)
    }
    
    fn set_state(&mut self, state: DeviceState) -> DeviceResult<()> {
        self.update_state(state);
        Ok(())
    }
    
    fn has_capability(&self, capability: DeviceCapability) -> DeviceResult<bool> {
        Ok(self.info.has_capability(capability))
    }
    
    fn properties(&self) -> DeviceResult<HashMap<String, String>> {
        Ok(self.info.properties.clone())
    }
    
    fn get_property(&self, key: &str) -> DeviceResult<Option<String>> {
        Ok(self.info.get_property(key).cloned())
    }
    
    fn set_property(&mut self, key: &str, value: &str) -> DeviceResult<()> {
        let previous = self.info.get_property(key).cloned();
        self.info.set_property(key.to_string(), value.to_string());
        self.emit_event(DeviceEventType::PropertyChanged {
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
    
    fn self_test(&mut self) -> DeviceResult<bool> {
        self.emit_event(DeviceEventType::SelfTestStarted);
        
        // Simulate self-test
        self.emit_event(DeviceEventType::SelfTestCompleted {
            success: true,
            status: "Self-test completed successfully".to_string(),
        });
        
        Ok(true)
    }
    
    fn calibrate(&mut self) -> DeviceResult<bool> {
        self.emit_event(DeviceEventType::CalibrationStarted);
        
        // Simulate calibration progress
        self.emit_event(DeviceEventType::CalibrationProgress {
            progress: 50,
            status: "Calibrating...".to_string(),
        });
        
        // Simulate calibration completion
        self.emit_event(DeviceEventType::CalibrationCompleted {
            success: true,
            status: "Calibration completed successfully".to_string(),
        });
        
        Ok(true)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl AudioDevice for MockAudioDevice {
    fn get_audio_device_type(&self) -> DeviceResult<AudioDeviceType> {
        Ok(self.audio_device_type)
    }
    
    fn get_config(&self) -> DeviceResult<AudioConfig> {
        Ok(self.config.clone())
    }
    
    fn set_config(&mut self, config: &AudioConfig) -> DeviceResult<()> {
        self.config = config.clone();
        Ok(())
    }
    
    fn get_available_formats(&self) -> DeviceResult<Vec<AudioFormat>> {
        Ok(self.available_formats.clone())
    }
    
    fn set_format(&mut self, format: AudioFormat) -> DeviceResult<()> {
        // Check if the format is supported
        if !self.available_formats.contains(&format) {
            return Err(DeviceError::InvalidParameter(format!(
                "Format {:?} not supported",
                format
            )));
        }
        
        self.config.format = format;
        Ok(())
    }
    
    fn set_volume(&mut self, volume: f32) -> DeviceResult<()> {
        if volume < 0.0 || volume > 1.0 {
            return Err(DeviceError::InvalidParameter(format!(
                "Volume {} out of range (0.0 - 1.0)",
                volume
            )));
        }
        
        self.config.volume = volume;
        Ok(())
    }
    
    fn get_volume(&self) -> DeviceResult<f32> {
        Ok(self.config.volume)
    }
    
    fn set_muted(&mut self, muted: bool) -> DeviceResult<()> {
        self.config.muted = muted;
        Ok(())
    }
    
    fn is_muted(&self) -> DeviceResult<bool> {
        Ok(self.config.muted)
    }
    
    fn set_spatial_audio_mode(&mut self, mode: SpatialAudioMode) -> DeviceResult<()> {
        // Check if the device supports spatial audio
        match self.audio_device_type {
            AudioDeviceType::Headphones | AudioDeviceType::Speaker | AudioDeviceType::Headset => {
                self.config.spatial_audio_mode = Some(mode);
                Ok(())
            }
            _ => Err(DeviceError::NotSupported(
                "Spatial audio not supported on this device".to_string(),
            )),
        }
    }
    
    fn get_spatial_audio_mode(&self) -> DeviceResult<Option<SpatialAudioMode>> {
        Ok(self.config.spatial_audio_mode)
    }
    
    fn set_beamforming_mode(&mut self, mode: BeamformingMode) -> DeviceResult<()> {
        // Check if the device supports beamforming
        match self.audio_device_type {
            AudioDeviceType::MicrophoneArray | AudioDeviceType::Headset => {
                self.config.beamforming_mode = Some(mode);
                Ok(())
            }
            _ => Err(DeviceError::NotSupported(
                "Beamforming not supported on this device".to_string(),
            )),
        }
    }
    
    fn get_beamforming_mode(&self) -> DeviceResult<Option<BeamformingMode>> {
        Ok(self.config.beamforming_mode)
    }
    
    fn set_noise_cancellation_level(&mut self, level: f32) -> DeviceResult<()> {
        if level < 0.0 || level > 1.0 {
            return Err(DeviceError::InvalidParameter(format!(
                "Noise cancellation level {} out of range (0.0 - 1.0)",
                level
            )));
        }
        
        // Check if the device supports noise cancellation
        match self.audio_device_type {
            AudioDeviceType::Microphone | AudioDeviceType::MicrophoneArray | AudioDeviceType::Headset => {
                self.config.noise_cancellation_level = Some(level);
                Ok(())
            }
            _ => Err(DeviceError::NotSupported(
                "Noise cancellation not supported on this device".to_string(),
            )),
        }
    }
    
    fn get_noise_cancellation_level(&self) -> DeviceResult<Option<f32>> {
        Ok(self.config.noise_cancellation_level)
    }
    
    fn set_vad_threshold(&mut self, threshold: f32) -> DeviceResult<()> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(DeviceError::InvalidParameter(format!(
                "VAD threshold {} out of range (0.0 - 1.0)",
                threshold
            )));
        }
        
        // Check if the device supports VAD
        match self.audio_device_type {
            AudioDeviceType::Microphone | AudioDeviceType::MicrophoneArray | AudioDeviceType::Headset => {
                self.config.vad_threshold = Some(threshold);
                Ok(())
            }
            _ => Err(DeviceError::NotSupported(
                "Voice activity detection not supported on this device".to_string(),
            )),
        }
    }
    
    fn get_vad_threshold(&self) -> DeviceResult<Option<f32>> {
        Ok(self.config.vad_threshold)
    }
    
    fn set_echo_cancellation_enabled(&mut self, enabled: bool) -> DeviceResult<()> {
        // Check if the device supports echo cancellation
        match self.audio_device_type {
            AudioDeviceType::Microphone | AudioDeviceType::MicrophoneArray | AudioDeviceType::Headset => {
                self.config.echo_cancellation_enabled = Some(enabled);
                Ok(())
            }
            _ => Err(DeviceError::NotSupported(
                "Echo cancellation not supported on this device".to_string(),
            )),
        }
    }
    
    fn is_echo_cancellation_enabled(&self) -> DeviceResult<Option<bool>> {
        Ok(self.config.echo_cancellation_enabled)
    }
    
    fn set_agc_enabled(&mut self, enabled: bool) -> DeviceResult<()> {
        // Check if the device supports AGC
        match self.audio_device_type {
            AudioDeviceType::Microphone | AudioDeviceType::MicrophoneArray | AudioDeviceType::Headset => {
                self.config.agc_enabled = Some(enabled);
                Ok(())
            }
            _ => Err(DeviceError::NotSupported(
                "Automatic gain control not supported on this device".to_string(),
            )),
        }
    }
    
    fn is_agc_enabled(&self) -> DeviceResult<Option<bool>> {
        Ok(self.config.agc_enabled)
    }
    
    fn set_equalizer_bands(&mut self, bands: Vec<EqualizerBand>) -> DeviceResult<()> {
        self.config.equalizer_bands = Some(bands);
        Ok(())
    }
    
    fn get_equalizer_bands(&self) -> DeviceResult<Option<Vec<EqualizerBand>>> {
        Ok(self.config.equalizer_bands.clone())
    }
    
    fn set_bass_boost(&mut self, level: f32) -> DeviceResult<()> {
        if level < 0.0 || level > 1.0 {
            return Err(DeviceError::InvalidParameter(format!(
                "Bass boost level {} out of range (0.0 - 1.0)",
                level
            )));
        }
        
        // Check if the device supports bass boost
        match self.audio_device_type {
            AudioDeviceType::Headphones | AudioDeviceType::Speaker | AudioDeviceType::Headset => {
                self.config.bass_boost = Some(level);
                Ok(())
            }
            _ => Err(DeviceError::NotSupported(
                "Bass boost not supported on this device".to_string(),
            )),
        }
    }
    
    fn get_bass_boost(&self) -> DeviceResult<Option<f32>> {
        Ok(self.config.bass_boost)
    }
    
    fn set_treble_boost(&mut self, level: f32) -> DeviceResult<()> {
        if level < 0.0 || level > 1.0 {
            return Err(DeviceError::InvalidParameter(format!(
                "Treble boost level {} out of range (0.0 - 1.0)",
                level
            )));
        }
        
        // Check if the device supports treble boost
        match self.audio_device_type {
            AudioDeviceType::Headphones | AudioDeviceType::Speaker | AudioDeviceType::Headset => {
                self.config.treble_boost = Some(level);
                Ok(())
            }
            _ => Err(DeviceError::NotSupported(
                "Treble boost not supported on this device".to_string(),
            )),
        }
    }
    
    fn get_treble_boost(&self) -> DeviceResult<Option<f32>> {
        Ok(self.config.treble_boost)
    }
    
    fn set_voice_enhancement(&mut self, level: f32) -> DeviceResult<()> {
        if level < 0.0 || level > 1.0 {
            return Err(DeviceError::InvalidParameter(format!(
                "Voice enhancement level {} out of range (0.0 - 1.0)",
                level
            )));
        }
        
        // Check if the device supports voice enhancement
        match self.audio_device_type {
            AudioDeviceType::Microphone | AudioDeviceType::MicrophoneArray | AudioDeviceType::Headset => {
                self.config.voice_enhancement = Some(level);
                Ok(())
            }
            _ => Err(DeviceError::NotSupported(
                "Voice enhancement not supported on this device".to_string(),
            )),
        }
    }
    
    fn get_voice_enhancement(&self) -> DeviceResult<Option<f32>> {
        Ok(self.config.voice_enhancement)
    }
    
    fn get_latency(&self) -> DeviceResult<f32> {
        Ok(self.latency)
    }
    
    fn get_snr(&self) -> DeviceResult<Option<f32>> {
        Ok(self.snr)
    }
    
    fn get_thd(&self) -> DeviceResult<Option<f32>> {
        Ok(self.thd)
    }
    
    fn run_audio_test(&mut self, test_type: &str) -> DeviceResult<HashMap<String, String>> {
        info!("Running audio test {} on device {}", test_type, self.info.id);
        
        let mut results = HashMap::new();
        results.insert("test_type".to_string(), test_type.to_string());
        results.insert("device_id".to_string(), self.info.id.clone());
        results.insert("status".to_string(), "success".to_string());
        
        match test_type {
            "latency" => {
                results.insert("latency_ms".to_string(), self.latency.to_string());
            }
            "snr" => {
                if let Some(snr) = self.snr {
                    results.insert("snr_db".to_string(), snr.to_string());
                } else {
                    results.insert("error".to_string(), "SNR measurement not available".to_string());
                }
            }
            "thd" => {
                if let Some(thd) = self.thd {
                    results.insert("thd_percent".to_string(), thd.to_string());
                } else {
                    results.insert("error".to_string(), "THD measurement not available".to_string());
                }
            }
            "frequency_response" => {
                results.insert("low_frequency".to_string(), "20Hz: -1.2dB".to_string());
                results.insert("mid_frequency".to_string(), "1kHz: 0.0dB".to_string());
                results.insert("high_frequency".to_string(), "20kHz: -2.5dB".to_string());
            }
            _ => {
                results.insert("error".to_string(), format!("Unknown test type: {}", test_type));
            }
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audio_format() {
        let format = AudioFormat::cd_quality();
        assert_eq!(format.sample_rate, 44100);
        assert_eq!(format.bit_depth, 16);
        assert_eq!(format.channels, 2);
        assert_eq!(format.is_float, false);
        
        let byte_rate = format.byte_rate();
        assert_eq!(byte_rate, 44100 * 2 * 2); // sample_rate * bytes_per_sample * channels
        
        let frame_size = format.frame_size();
        assert_eq!(frame_size, 2 * 2); // bytes_per_sample * channels
    }
    
    #[test]
    fn test_audio_config() {
        let config = AudioConfig::vr_headphones();
        assert_eq!(config.format.sample_rate, 48000);
        assert_eq!(config.volume, 0.8);
        assert_eq!(config.muted, false);
        assert_eq!(config.spatial_audio_mode, Some(SpatialAudioMode::Binaural));
        assert!(config.equalizer_bands.is_some());
        
        let mic_config = AudioConfig::vr_microphone_array();
        assert_eq!(mic_config.format.channels, 4);
        assert_eq!(mic_config.beamforming_mode, Some(BeamformingMode::Adaptive));
        assert_eq!(mic_config.noise_cancellation_level, Some(0.7));
    }
    
    #[test]
    fn test_mock_audio_device() {
        let mut headphones = MockAudioDevice::new("headphones-1", "Test Headphones", AudioDeviceType::Headphones);
        let mut microphone = MockAudioDevice::new("microphone-1", "Test Microphone", AudioDeviceType::Microphone);
        
        // Test initialization
        headphones.initialize().unwrap();
        microphone.initialize().unwrap();
        
        assert_eq!(headphones.state(), Ok(DeviceState::Ready));
        assert_eq!(microphone.state(), Ok(DeviceState::Ready));
        
        // Test audio device type
        assert_eq!(headphones.get_audio_device_type().unwrap(), AudioDeviceType::Headphones);
        assert_eq!(microphone.get_audio_device_type().unwrap(), AudioDeviceType::Microphone);
        
        // Test volume control
        headphones.set_volume(0.5).unwrap();
        assert_eq!(headphones.get_volume().unwrap(), 0.5);
        
        // Test mute control
        headphones.set_muted(true).unwrap();
        assert_eq!(headphones.is_muted().unwrap(), true);
        
        // Test spatial audio (should work on headphones)
        headphones.set_spatial_audio_mode(SpatialAudioMode::Binaural).unwrap();
        assert_eq!(headphones.get_spatial_audio_mode().unwrap(), Some(SpatialAudioMode::Binaural));
        
        // Test spatial audio (should fail on microphone)
        assert!(microphone.set_spatial_audio_mode(SpatialAudioMode::Binaural).is_err());
        
        // Test noise cancellation (should work on microphone)
        microphone.set_noise_cancellation_level(0.8).unwrap();
        assert_eq!(microphone.get_noise_cancellation_level().unwrap(), Some(0.8));
        
        // Test noise cancellation (should fail on headphones)
        assert!(headphones.set_noise_cancellation_level(0.8).is_err());
        
        // Test audio tests
        let latency_test = headphones.run_audio_test("latency").unwrap();
        assert_eq!(latency_test.get("status").unwrap(), "success");
        assert!(latency_test.contains_key("latency_ms"));
        
        // Test shutdown
        headphones.shutdown().unwrap();
        microphone.shutdown().unwrap();
        
        assert_eq!(headphones.state(), Ok(DeviceState::Disconnected));
        assert_eq!(microphone.state(), Ok(DeviceState::Disconnected));
    }
    
    #[test]
    fn test_audio_manager() {
        let mut manager = AudioManager::new();
        
        // Initialize manager
        manager.initialize().unwrap();
        
        // Register devices
        let headphones = Box::new(MockAudioDevice::new(
            "headphones-1",
            "Test Headphones",
            AudioDeviceType::Headphones,
        ));
        let microphone = Box::new(MockAudioDevice::new(
            "microphone-1",
            "Test Microphone",
            AudioDeviceType::Microphone,
        ));
        
        manager.register_device("headphones-1", headphones).unwrap();
        manager.register_device("microphone-1", microphone).unwrap();
        
        // Check default devices
        assert_eq!(manager.default_output_id, Some("headphones-1".to_string()));
        assert_eq!(manager.default_input_id, Some("microphone-1".to_string()));
        
        // Get devices
        let output = manager.get_default_output().unwrap();
        let output_guard = output.lock().unwrap();
        assert_eq!(output_guard.info().unwrap().id, "headphones-1");
        
        let input = manager.get_default_input().unwrap();
        let input_guard = input.lock().unwrap();
        assert_eq!(input_guard.info().unwrap().id, "microphone-1");
        
        // Test global volume
        drop(output_guard);
        drop(input_guard);
        
        manager.set_global_volume(0.7).unwrap();
        assert_eq!(manager.get_global_volume(), 0.7);
        
        let output = manager.get_default_output().unwrap();
        let output_guard = output.lock().unwrap();
        assert_eq!(output_guard.get_volume().unwrap(), 0.7);
        
        // Test global mute
        drop(output_guard);
        
        manager.set_global_muted(true).unwrap();
        assert_eq!(manager.is_global_muted(), true);
        
        let output = manager.get_default_output().unwrap();
        let output_guard = output.lock().unwrap();
        assert_eq!(output_guard.is_muted().unwrap(), true);
        
        // Shutdown manager
        drop(output_guard);
        manager.shutdown().unwrap();
    }
}

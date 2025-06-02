import axios from 'axios';

// Define the base URL for the API
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api';

// Create an axios instance with default config
const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Configuration API endpoints
export const configApi = {
  // Get all configuration values
  listConfig: async (category?: string) => {
    const params = category ? { category } : {};
    const response = await api.get('/config', { params });
    return response.data;
  },
  
  // Get a specific configuration value
  getConfig: async (category: string, key: string) => {
    const response = await api.get(`/config/${category}/${key}`);
    return response.data;
  },
  
  // Set a configuration value
  setConfig: async (category: string, key: string, value: string, valueType: string) => {
    const response = await api.post('/config', {
      category,
      key,
      value,
      value_type: valueType,
    });
    return response.data;
  },
  
  // Reset configuration to defaults
  resetConfig: async (category?: string) => {
    const response = await api.post('/config/reset', {
      category,
    });
    return response.data;
  },
};

// Hardware API endpoints
export const hardwareApi = {
  // Get all hardware devices
  listHardware: async (deviceType?: string) => {
    const params = deviceType ? { device_type: deviceType } : {};
    const response = await api.get('/hardware', { params });
    return response.data;
  },
  
  // Get details for a specific device
  getHardwareInfo: async (name: string) => {
    const response = await api.get(`/hardware/${name}`);
    return response.data;
  },
  
  // Initialize hardware devices
  initHardware: async (device?: string) => {
    const response = await api.post('/hardware/init', {
      device,
    });
    return response.data;
  },
  
  // Shutdown hardware devices
  shutdownHardware: async (device?: string) => {
    const response = await api.post('/hardware/shutdown', {
      device,
    });
    return response.data;
  },
  
  // Run diagnostics on hardware devices
  diagnoseHardware: async (device?: string, level?: string) => {
    const response = await api.post('/hardware/diagnose', {
      device,
      level,
    });
    return response.data;
  },
};

// System API endpoints
export const systemApi = {
  // Get system status
  getSystemStatus: async () => {
    const response = await api.get('/system/status');
    return response.data;
  },
  
  // Get system information
  getSystemInfo: async () => {
    const response = await api.get('/system/info');
    return response.data;
  },
  
  // Restart the VR system
  restartSystem: async (force?: boolean) => {
    const response = await api.post('/system/restart', {
      force,
    });
    return response.data;
  },
  
  // Update the VR system
  updateSystem: async (checkOnly?: boolean) => {
    const response = await api.post('/system/update', {
      check_only: checkOnly,
    });
    return response.data;
  },
};

// Monitoring API endpoints
export const monitoringApi = {
  // Get monitoring status
  getMonitoringStatus: async () => {
    const response = await api.get('/monitoring/status');
    return response.data;
  },
};

// IPC API endpoints
export const ipcApi = {
  // Get IPC status
  getIpcStatus: async () => {
    const response = await api.get('/ipc/status');
    return response.data;
  },
};

// Security API endpoints
export const securityApi = {
  // Get security status
  getSecurityStatus: async () => {
    const response = await api.get('/security/status');
    return response.data;
  },
};

export default {
  config: configApi,
  hardware: hardwareApi,
  system: systemApi,
  monitoring: monitoringApi,
  ipc: ipcApi,
  security: securityApi,
};

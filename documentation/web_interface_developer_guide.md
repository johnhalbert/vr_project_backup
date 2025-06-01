# Web Interface Developer Guide

## Introduction

This guide provides detailed information for developers who want to work with the VR headset's Web Interface. The Web Interface consists of a Rust backend and a React frontend, providing a web-based management interface for the VR headset system.

This guide assumes you are already familiar with the general concepts covered in the main Developer Guide and focuses specifically on working with the Web Interface components.

## Web Interface Architecture

The Web Interface is structured as a client-server application:

```
/system_ui/vr_web/
├── src/                # Rust backend
│   ├── api/            # API endpoints
│   ├── error.rs        # Error handling
│   ├── main.rs         # Server entry point
│   └── state.rs        # Application state
├── frontend/           # React frontend
│   └── vr_frontend/
│       ├── public/     # Static assets
│       ├── src/        # React components
│       └── package.json # Frontend dependencies
└── Cargo.toml          # Backend dependencies
```

The backend provides RESTful API endpoints that the frontend consumes to display and manage the VR headset system.

## Getting Started with Web Interface Development

### Setting Up Your Development Environment

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/vrheadset/vr_web.git
   cd vr_web
   ```

2. **Install Backend Dependencies**:
   ```bash
   # Install Rust using rustup
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install additional dependencies
   sudo apt-get update
   sudo apt-get install -y build-essential pkg-config libssl-dev
   ```

3. **Install Frontend Dependencies**:
   ```bash
   cd frontend/vr_frontend
   npm install
   ```

4. **Build and Run the Backend**:
   ```bash
   cd ../..  # Return to the project root
   cargo build
   cargo run
   ```

5. **Build and Run the Frontend**:
   ```bash
   cd frontend/vr_frontend
   npm start
   ```

### Project Structure

#### Backend Structure

The backend follows a modular architecture:

- `main.rs`: Server entry point and configuration
- `state.rs`: Application state management
- `error.rs`: Error handling and response formatting
- `api/`: API endpoint modules
  - `config.rs`: Configuration management endpoints
  - `hardware.rs`: Hardware control endpoints
  - `ipc.rs`: IPC management endpoints
  - `monitoring.rs`: System monitoring endpoints
  - `security.rs`: Security and authentication endpoints
  - `system.rs`: System management endpoints

#### Frontend Structure

The frontend follows a component-based architecture:

- `components/`: React components
  - `App.tsx`: Main application component
  - `Dashboard.tsx`: Dashboard view
  - `ConfigPanel.tsx`: Configuration panel
  - `HardwarePanel.tsx`: Hardware control panel
  - `MonitoringPanel.tsx`: System monitoring panel
  - `SystemPanel.tsx`: System management panel
  - `ui/`: Reusable UI components
- `hooks/`: Custom React hooks
- `lib/`: Utility functions and API clients
- `App.tsx`: Application entry point
- `main.tsx`: React rendering entry point

## Backend Development

### API Endpoint Development

The backend provides RESTful API endpoints for the frontend to consume. Each endpoint is defined in a separate module in the `api/` directory.

#### Example: Creating a New API Endpoint

```rust
// src/api/custom.rs
use actix_web::{web, HttpResponse, Scope};
use serde::{Deserialize, Serialize};
use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize, Deserialize)]
struct CustomRequest {
    name: String,
    value: i32,
}

#[derive(Serialize)]
struct CustomResponse {
    status: String,
    result: i32,
}

async fn get_custom(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let name = path.into_inner();
    
    // Access application state
    let data = state.custom_data.read().await;
    
    // Check if the requested item exists
    if let Some(value) = data.get(&name) {
        // Return the item
        Ok(HttpResponse::Ok().json(CustomResponse {
            status: "success".to_string(),
            result: *value,
        }))
    } else {
        // Return an error
        Err(ApiError::NotFound(format!("Custom item '{}' not found", name)))
    }
}

async fn post_custom(
    state: web::Data<AppState>,
    req: web::Json<CustomRequest>,
) -> Result<HttpResponse, ApiError> {
    let request = req.into_inner();
    
    // Access application state
    let mut data = state.custom_data.write().await;
    
    // Store the item
    data.insert(request.name.clone(), request.value);
    
    // Return success
    Ok(HttpResponse::Ok().json(CustomResponse {
        status: "success".to_string(),
        result: request.value,
    }))
}

pub fn custom_scope() -> Scope {
    web::scope("/custom")
        .route("/{name}", web::get().to(get_custom))
        .route("", web::post().to(post_custom))
}
```

#### Registering the API Endpoint

```rust
// src/main.rs
use actix_web::{App, HttpServer, web};
use crate::api::{config, hardware, ipc, monitoring, security, system, custom};
use crate::state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create application state
    let state = web::Data::new(AppState::new().await?);
    
    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(web::scope("/api")
                .service(config::config_scope())
                .service(hardware::hardware_scope())
                .service(ipc::ipc_scope())
                .service(monitoring::monitoring_scope())
                .service(security::security_scope())
                .service(system::system_scope())
                .service(custom::custom_scope())  // Add the new endpoint
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### Error Handling

The backend uses a centralized error handling system to ensure consistent error responses.

```rust
// src/error.rs
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    InternalError(String),
    Unauthorized(String),
    Forbidden(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    status: String,
    message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ApiError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ApiError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let response = ErrorResponse {
            status: "error".to_string(),
            message: self.to_string(),
        };
        
        match self {
            ApiError::NotFound(_) => HttpResponse::NotFound().json(response),
            ApiError::BadRequest(_) => HttpResponse::BadRequest().json(response),
            ApiError::InternalError(_) => HttpResponse::InternalServerError().json(response),
            ApiError::Unauthorized(_) => HttpResponse::Unauthorized().json(response),
            ApiError::Forbidden(_) => HttpResponse::Forbidden().json(response),
        }
    }
}
```

### Authentication and Authorization

The backend uses JWT (JSON Web Tokens) for authentication and role-based access control for authorization.

```rust
// src/api/security.rs
use actix_web::{web, HttpResponse, Scope};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    status: String,
    token: String,
    expires_in: i64,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    role: String,
    exp: i64,
}

async fn login(
    state: web::Data<AppState>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    let request = req.into_inner();
    
    // Validate credentials (in a real system, this would check against a database)
    if request.username == "admin" && request.password == "password" {
        // Create JWT claims
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(1))
            .expect("valid timestamp")
            .timestamp();
        
        let claims = Claims {
            sub: request.username,
            role: "admin".to_string(),
            exp: expiration,
        };
        
        // Encode the JWT
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(state.jwt_secret.as_ref()),
        )
        .map_err(|e| ApiError::InternalError(format!("JWT encoding error: {}", e)))?;
        
        // Return the token
        Ok(HttpResponse::Ok().json(LoginResponse {
            status: "success".to_string(),
            token,
            expires_in: 3600,  // 1 hour in seconds
        }))
    } else {
        // Return an error
        Err(ApiError::Unauthorized("Invalid credentials".to_string()))
    }
}

pub fn security_scope() -> Scope {
    web::scope("/security")
        .route("/login", web::post().to(login))
}
```

### Middleware for Authentication

```rust
// src/middleware/auth.rs
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::rc::Rc;
use std::task::{Context, Poll};
use crate::api::security::Claims;
use crate::error::ApiError;

pub struct Authentication {
    jwt_secret: Rc<Vec<u8>>,
}

impl Authentication {
    pub fn new(jwt_secret: Vec<u8>) -> Self {
        Authentication {
            jwt_secret: Rc::new(jwt_secret),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware {
            service,
            jwt_secret: self.jwt_secret.clone(),
        }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: S,
    jwt_secret: Rc<Vec<u8>>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Skip authentication for login endpoint
        if req.path() == "/api/security/login" {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }

        // Get the Authorization header
        let auth_header = req.headers().get("Authorization");
        let auth_header = match auth_header {
            Some(header) => header,
            None => {
                return Box::pin(async move {
                    Err(ApiError::Unauthorized("Missing Authorization header".to_string()).into())
                });
            }
        };

        // Extract the token
        let auth_str = match auth_header.to_str() {
            Ok(str) => str,
            Err(_) => {
                return Box::pin(async move {
                    Err(ApiError::Unauthorized("Invalid Authorization header".to_string()).into())
                });
            }
        };

        // Check if it's a Bearer token
        if !auth_str.starts_with("Bearer ") {
            return Box::pin(async move {
                Err(ApiError::Unauthorized("Invalid Authorization scheme".to_string()).into())
            });
        }

        // Extract the token
        let token = &auth_str[7..];

        // Decode and validate the token
        let claims = match decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(token_data) => token_data.claims,
            Err(_) => {
                return Box::pin(async move {
                    Err(ApiError::Unauthorized("Invalid token".to_string()).into())
                });
            }
        };

        // Store the claims in the request extensions
        req.extensions_mut().insert(claims);

        // Call the next service
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
```

### WebSocket Support

The backend supports WebSockets for real-time communication with the frontend.

```rust
// src/api/websocket.rs
use actix::{Actor, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use actix_web_actors::ws;
use crate::state::AppState;

struct WebSocketSession {
    state: web::Data<AppState>,
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                println!("Received message: {}", text);
                
                // Echo the message back
                ctx.text(text);
            },
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn websocket(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let session = WebSocketSession { state };
    ws::start(session, &req, stream)
}

pub fn websocket_scope() -> Scope {
    web::scope("/ws")
        .route("", web::get().to(websocket))
}
```

## Frontend Development

### Component Development

The frontend uses React components to build the user interface. Each component is defined in a separate file in the `components/` directory.

#### Example: Creating a New Component

```tsx
// src/components/CustomPanel.tsx
import React, { useState, useEffect } from 'react';
import { Card, Button, Input, Table } from './ui';
import { useApi } from '../hooks/useApi';

interface CustomItem {
  name: string;
  value: number;
}

export const CustomPanel: React.FC = () => {
  const [items, setItems] = useState<CustomItem[]>([]);
  const [name, setName] = useState('');
  const [value, setValue] = useState(0);
  const [loading, setLoading] = useState(false);
  const api = useApi();

  // Load items on component mount
  useEffect(() => {
    const fetchItems = async () => {
      setLoading(true);
      try {
        const response = await api.get('/api/custom');
        setItems(response.data);
      } catch (error) {
        console.error('Failed to fetch items:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchItems();
  }, [api]);

  // Handle form submission
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    setLoading(true);
    try {
      await api.post('/api/custom', { name, value });
      
      // Refresh the item list
      const response = await api.get('/api/custom');
      setItems(response.data);
      
      // Clear the form
      setName('');
      setValue(0);
    } catch (error) {
      console.error('Failed to create item:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card title="Custom Items">
      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <label htmlFor="name">Name</label>
          <Input
            id="name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
          />
        </div>
        
        <div className="form-group">
          <label htmlFor="value">Value</label>
          <Input
            id="value"
            type="number"
            value={value}
            onChange={(e) => setValue(parseInt(e.target.value, 10))}
            required
          />
        </div>
        
        <Button type="submit" disabled={loading}>
          {loading ? 'Saving...' : 'Save'}
        </Button>
      </form>
      
      <Table
        columns={[
          { key: 'name', title: 'Name' },
          { key: 'value', title: 'Value' },
        ]}
        data={items}
        loading={loading}
      />
    </Card>
  );
};
```

#### Adding the Component to the Dashboard

```tsx
// src/components/Dashboard.tsx
import React from 'react';
import { ConfigPanel } from './ConfigPanel';
import { HardwarePanel } from './HardwarePanel';
import { MonitoringPanel } from './MonitoringPanel';
import { SystemPanel } from './SystemPanel';
import { CustomPanel } from './CustomPanel';

export const Dashboard: React.FC = () => {
  return (
    <div className="dashboard">
      <div className="dashboard-row">
        <ConfigPanel />
        <HardwarePanel />
      </div>
      
      <div className="dashboard-row">
        <MonitoringPanel />
        <SystemPanel />
      </div>
      
      <div className="dashboard-row">
        <CustomPanel />
      </div>
    </div>
  );
};
```

### API Client

The frontend uses an API client to communicate with the backend.

```tsx
// src/lib/api.ts
import axios, { AxiosInstance, AxiosRequestConfig } from 'axios';

export class ApiClient {
  private client: AxiosInstance;
  
  constructor() {
    this.client = axios.create({
      baseURL: process.env.REACT_APP_API_URL || 'http://localhost:8080',
      timeout: 10000,
      headers: {
        'Content-Type': 'application/json',
      },
    });
    
    // Add request interceptor for authentication
    this.client.interceptors.request.use(
      (config) => {
        const token = localStorage.getItem('token');
        if (token) {
          config.headers.Authorization = `Bearer ${token}`;
        }
        return config;
      },
      (error) => {
        return Promise.reject(error);
      }
    );
    
    // Add response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => {
        return response;
      },
      (error) => {
        if (error.response && error.response.status === 401) {
          // Redirect to login page
          window.location.href = '/login';
        }
        return Promise.reject(error);
      }
    );
  }
  
  public async get(url: string, config?: AxiosRequestConfig) {
    return this.client.get(url, config);
  }
  
  public async post(url: string, data?: any, config?: AxiosRequestConfig) {
    return this.client.post(url, data, config);
  }
  
  public async put(url: string, data?: any, config?: AxiosRequestConfig) {
    return this.client.put(url, data, config);
  }
  
  public async delete(url: string, config?: AxiosRequestConfig) {
    return this.client.delete(url, config);
  }
}

export const api = new ApiClient();
```

### Authentication Hook

```tsx
// src/hooks/useAuth.ts
import { useState, useEffect, useCallback } from 'react';
import { api } from '../lib/api';

interface LoginCredentials {
  username: string;
  password: string;
}

interface AuthState {
  isAuthenticated: boolean;
  token: string | null;
  username: string | null;
}

export const useAuth = () => {
  const [authState, setAuthState] = useState<AuthState>({
    isAuthenticated: false,
    token: null,
    username: null,
  });
  
  // Initialize auth state from localStorage
  useEffect(() => {
    const token = localStorage.getItem('token');
    const username = localStorage.getItem('username');
    
    if (token && username) {
      setAuthState({
        isAuthenticated: true,
        token,
        username,
      });
    }
  }, []);
  
  // Login function
  const login = useCallback(async (credentials: LoginCredentials) => {
    try {
      const response = await api.post('/api/security/login', credentials);
      const { token } = response.data;
      
      // Store token and username in localStorage
      localStorage.setItem('token', token);
      localStorage.setItem('username', credentials.username);
      
      // Update auth state
      setAuthState({
        isAuthenticated: true,
        token,
        username: credentials.username,
      });
      
      return true;
    } catch (error) {
      console.error('Login failed:', error);
      return false;
    }
  }, []);
  
  // Logout function
  const logout = useCallback(() => {
    // Clear localStorage
    localStorage.removeItem('token');
    localStorage.removeItem('username');
    
    // Update auth state
    setAuthState({
      isAuthenticated: false,
      token: null,
      username: null,
    });
  }, []);
  
  return {
    isAuthenticated: authState.isAuthenticated,
    username: authState.username,
    login,
    logout,
  };
};
```

### WebSocket Hook

```tsx
// src/hooks/useWebSocket.ts
import { useState, useEffect, useCallback } from 'react';

interface WebSocketOptions {
  onOpen?: (event: Event) => void;
  onMessage?: (event: MessageEvent) => void;
  onClose?: (event: CloseEvent) => void;
  onError?: (event: Event) => void;
}

export const useWebSocket = (url: string, options: WebSocketOptions = {}) => {
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  
  // Connect to WebSocket
  useEffect(() => {
    const ws = new WebSocket(url);
    
    ws.addEventListener('open', (event) => {
      setIsConnected(true);
      if (options.onOpen) {
        options.onOpen(event);
      }
    });
    
    ws.addEventListener('message', (event) => {
      if (options.onMessage) {
        options.onMessage(event);
      }
    });
    
    ws.addEventListener('close', (event) => {
      setIsConnected(false);
      if (options.onClose) {
        options.onClose(event);
      }
    });
    
    ws.addEventListener('error', (event) => {
      if (options.onError) {
        options.onError(event);
      }
    });
    
    setSocket(ws);
    
    // Clean up on unmount
    return () => {
      ws.close();
    };
  }, [url, options]);
  
  // Send message function
  const sendMessage = useCallback((data: string | ArrayBufferLike | Blob | ArrayBufferView) => {
    if (socket && isConnected) {
      socket.send(data);
      return true;
    }
    return false;
  }, [socket, isConnected]);
  
  return {
    isConnected,
    sendMessage,
  };
};
```

## Integration with Core API

The Web Interface integrates with the Core API to provide a web-based management interface for the VR headset system.

### Backend Integration

The backend uses the Core API to access hardware, configuration, and system services.

```rust
// src/api/hardware.rs
use actix_web::{web, HttpResponse, Scope};
use serde::{Deserialize, Serialize};
use vr_core_api::hardware::{DeviceManager, DeviceType, DeviceError};
use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize)]
struct DeviceInfo {
    id: String,
    name: String,
    device_type: String,
    vendor: String,
    model: String,
    is_available: bool,
}

async fn get_devices(
    state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    // Get the device manager from the Core API
    let device_manager = DeviceManager::new()
        .map_err(|e| ApiError::InternalError(format!("Failed to create device manager: {}", e)))?;
    
    // Discover devices
    device_manager.discover_devices()
        .map_err(|e| ApiError::InternalError(format!("Failed to discover devices: {}", e)))?;
    
    // Get all devices
    let devices = device_manager.get_all_devices()
        .map_err(|e| ApiError::InternalError(format!("Failed to get devices: {}", e)))?;
    
    // Convert to API response format
    let device_infos: Vec<DeviceInfo> = devices.iter().map(|device| {
        let info = device.get_info();
        DeviceInfo {
            id: info.id.clone(),
            name: info.name.clone(),
            device_type: format!("{:?}", info.device_type),
            vendor: info.vendor.clone(),
            model: info.model.clone(),
            is_available: device.is_available(),
        }
    }).collect();
    
    // Return the device list
    Ok(HttpResponse::Ok().json(device_infos))
}

#[derive(Deserialize)]
struct DeviceConfigRequest {
    id: String,
    config: serde_json::Value,
}

async fn configure_device(
    state: web::Data<AppState>,
    req: web::Json<DeviceConfigRequest>,
) -> Result<HttpResponse, ApiError> {
    let request = req.into_inner();
    
    // Get the device manager from the Core API
    let device_manager = DeviceManager::new()
        .map_err(|e| ApiError::InternalError(format!("Failed to create device manager: {}", e)))?;
    
    // Get the device
    let device = device_manager.get_device(&request.id)
        .map_err(|e| ApiError::NotFound(format!("Device not found: {}", e)))?;
    
    // Configure the device
    device.configure(&request.config)
        .map_err(|e| ApiError::BadRequest(format!("Failed to configure device: {}", e)))?;
    
    // Return success
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Device configured successfully",
    })))
}

pub fn hardware_scope() -> Scope {
    web::scope("/hardware")
        .route("/devices", web::get().to(get_devices))
        .route("/configure", web::post().to(configure_device))
}
```

### Frontend Integration

The frontend uses the API client to communicate with the backend, which in turn uses the Core API.

```tsx
// src/components/HardwarePanel.tsx
import React, { useState, useEffect } from 'react';
import { Card, Button, Table } from './ui';
import { useApi } from '../hooks/useApi';

interface Device {
  id: string;
  name: string;
  device_type: string;
  vendor: string;
  model: string;
  is_available: boolean;
}

export const HardwarePanel: React.FC = () => {
  const [devices, setDevices] = useState<Device[]>([]);
  const [loading, setLoading] = useState(false);
  const api = useApi();

  // Load devices on component mount
  useEffect(() => {
    const fetchDevices = async () => {
      setLoading(true);
      try {
        const response = await api.get('/api/hardware/devices');
        setDevices(response.data);
      } catch (error) {
        console.error('Failed to fetch devices:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchDevices();
  }, [api]);

  // Handle device configuration
  const handleConfigure = async (device: Device) => {
    // In a real application, this would open a configuration dialog
    console.log('Configure device:', device);
  };

  return (
    <Card title="Hardware Devices">
      <Table
        columns={[
          { key: 'name', title: 'Name' },
          { key: 'device_type', title: 'Type' },
          { key: 'vendor', title: 'Vendor' },
          { key: 'model', title: 'Model' },
          { key: 'is_available', title: 'Available', render: (value) => value ? 'Yes' : 'No' },
          {
            key: 'actions',
            title: 'Actions',
            render: (_, device) => (
              <Button onClick={() => handleConfigure(device)}>Configure</Button>
            ),
          },
        ]}
        data={devices}
        loading={loading}
      />
    </Card>
  );
};
```

## Best Practices for Web Interface Development

### Backend Best Practices

1. **API Design**:
   - Use RESTful principles for API design
   - Use consistent URL patterns
   - Use appropriate HTTP methods
   - Return appropriate HTTP status codes
   - Use JSON for request and response bodies

2. **Error Handling**:
   - Use a centralized error handling system
   - Return consistent error responses
   - Include helpful error messages
   - Log errors for debugging

3. **Authentication and Authorization**:
   - Use JWT for authentication
   - Implement role-based access control
   - Protect sensitive endpoints
   - Use HTTPS in production

4. **Performance**:
   - Use async/await for I/O-bound operations
   - Implement caching where appropriate
   - Optimize database queries
   - Use connection pooling

### Frontend Best Practices

1. **Component Design**:
   - Use functional components with hooks
   - Keep components small and focused
   - Use TypeScript for type safety
   - Implement proper error handling

2. **State Management**:
   - Use React hooks for local state
   - Consider Redux or Context API for global state
   - Minimize state duplication
   - Use immutable state updates

3. **Performance**:
   - Use memoization for expensive calculations
   - Implement virtualization for large lists
   - Optimize re-renders with React.memo
   - Use code splitting for large applications

4. **User Experience**:
   - Implement responsive design
   - Provide loading indicators
   - Handle errors gracefully
   - Implement form validation

## Troubleshooting

### Common Backend Issues

1. **API Endpoint Not Found**:
   - Check the URL path
   - Verify that the endpoint is registered
   - Check for typos in the route definition

2. **Authentication Failures**:
   - Check JWT secret
   - Verify token expiration
   - Check for proper Authorization header

3. **Database Connection Issues**:
   - Check connection string
   - Verify database credentials
   - Check network connectivity

4. **Performance Issues**:
   - Profile API endpoints
   - Check for N+1 queries
   - Implement caching
   - Optimize database queries

### Common Frontend Issues

1. **API Request Failures**:
   - Check network connectivity
   - Verify API URL
   - Check for CORS issues
   - Verify authentication token

2. **Rendering Issues**:
   - Check for React key warnings
   - Verify component props
   - Check for null or undefined values
   - Use React DevTools for debugging

3. **State Management Issues**:
   - Check for state mutations
   - Verify state updates
   - Check for race conditions
   - Use Redux DevTools for debugging

4. **Performance Issues**:
   - Check for unnecessary re-renders
   - Implement memoization
   - Use React.memo for pure components
   - Implement virtualization for large lists

## Conclusion

The Web Interface provides a powerful and flexible way to manage the VR headset system. By following the guidelines in this document, you can create robust, secure, and performant web applications that integrate seamlessly with the Core API.

For more information, refer to the API documentation and example code provided with the Web Interface.

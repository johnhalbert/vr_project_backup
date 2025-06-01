//! HTTPS server implementation for the VR headset.
//!
//! This module provides a simple HTTPS server implementation using hyper and rustls.

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn, Service};
use hyper::{Body, Request, Response};
use log::{debug, error, info};
use rustls::ServerConfig;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use super::cookie::CookieManager;

/// Server error type.
#[derive(Debug, Error)]
pub enum ServerError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Hyper error
    #[error("Hyper error: {0}")]
    Hyper(String),
    
    /// TLS error
    #[error("TLS error: {0}")]
    Tls(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Server result type.
pub type ServerResult<T> = Result<T, ServerError>;

/// HTTPS server.
pub struct HttpsServer {
    /// Server address
    addr: SocketAddr,
    
    /// TLS configuration
    tls_config: Arc<ServerConfig>,
    
    /// Cookie manager
    cookie_manager: CookieManager,
    
    /// Request handlers
    handlers: Arc<Vec<Box<dyn RequestHandler>>>,
}

impl HttpsServer {
    /// Create a new HTTPS server.
    pub fn new(
        addr: SocketAddr,
        tls_config: Arc<ServerConfig>,
    ) -> Self {
        Self {
            addr,
            tls_config,
            cookie_manager: CookieManager::new(),
            handlers: Arc::new(Vec::new()),
        }
    }
    
    /// Set the cookie manager.
    pub fn with_cookie_manager(mut self, cookie_manager: CookieManager) -> Self {
        self.cookie_manager = cookie_manager;
        self
    }
    
    /// Add a request handler.
    pub fn with_handler<H: RequestHandler + 'static>(mut self, handler: H) -> Self {
        let handlers = Arc::get_mut(&mut self.handlers)
            .expect("Cannot modify handlers after server is started");
        
        handlers.push(Box::new(handler));
        self
    }
    
    /// Start the server.
    pub async fn start(self) -> ServerResult<()> {
        // Create the TLS acceptor
        let acceptor = TlsAcceptor::from(self.tls_config);
        
        // Create the TCP listener
        let listener = TcpListener::bind(self.addr).await?;
        let addr = listener.local_addr()?;
        
        info!("HTTPS server listening on {}", addr);
        
        // Create the service
        let handlers = self.handlers.clone();
        let cookie_manager = self.cookie_manager;
        
        let make_svc = make_service_fn(move |_| {
            let handlers = handlers.clone();
            let cookie_manager = cookie_manager.clone();
            
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                    let handlers = handlers.clone();
                    let cookie_manager = cookie_manager.clone();
                    
                    async move {
                        // Process the request
                        for handler in handlers.iter() {
                            if handler.can_handle(&req) {
                                return handler.handle(req, &cookie_manager).await;
                            }
                        }
                        
                        // No handler found
                        Ok::<_, hyper::Error>(Response::builder()
                            .status(404)
                            .body(Body::from("Not Found"))
                            .unwrap())
                    }
                }))
            }
        });
        
        // Accept connections
        loop {
            let (stream, peer_addr) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    continue;
                }
            };
            
            debug!("Accepted connection from {}", peer_addr);
            
            let acceptor = acceptor.clone();
            let make_svc = make_svc.clone();
            
            // Accept the TLS connection
            let tls_stream = match acceptor.accept(stream).await {
                Ok(stream) => stream,
                Err(e) => {
                    error!("Failed to accept TLS connection: {}", e);
                    continue;
                }
            };
            
            // Serve the connection
            tokio::spawn(async move {
                // Create a new service for this connection
                let mut svc_maker = make_svc;
                let svc = match svc_maker.call(&()).await {
                    Ok(svc) => svc,
                    Err(e) => {
                        error!("Failed to create service: {}", e);
                        return;
                    }
                };
                
                if let Err(e) = hyper::server::conn::Http::new()
                    .serve_connection(tls_stream, svc)
                    .await
                {
                    error!("Failed to serve connection: {}", e);
                }
            });
        }
    }
}

/// Request handler trait.
pub trait RequestHandler: Send + Sync {
    /// Check if this handler can handle the request.
    fn can_handle(&self, req: &Request<Body>) -> bool;
    
    /// Handle the request.
    fn handle(&self, req: Request<Body>, cookie_manager: &CookieManager) -> Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send>>;
}

/// Simple request handler.
pub struct SimpleHandler {
    /// Path prefix
    path_prefix: String,
    
    /// Handler function
    handler: Box<dyn Fn(Request<Body>, &CookieManager) -> Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send>> + Send + Sync>,
}

impl SimpleHandler {
    /// Create a new simple handler.
    pub fn new<F, Fut>(path_prefix: &str, handler: F) -> Self
    where
        F: Fn(Request<Body>, &CookieManager) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response<Body>, hyper::Error>> + Send + 'static,
    {
        Self {
            path_prefix: path_prefix.to_string(),
            handler: Box::new(move |req, cm| Box::pin(handler(req, cm))),
        }
    }
}

impl RequestHandler for SimpleHandler {
    fn can_handle(&self, req: &Request<Body>) -> bool {
        req.uri().path().starts_with(&self.path_prefix)
    }
    
    fn handle(&self, req: Request<Body>, cookie_manager: &CookieManager) -> Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send>> {
        (self.handler)(req, cookie_manager)
    }
}

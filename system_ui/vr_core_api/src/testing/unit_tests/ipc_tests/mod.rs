//! IPC unit tests module for the VR headset system.
//!
//! This module contains unit tests for the IPC components of the VR headset system,
//! including message serialization, Unix sockets, D-Bus, and WebSockets.

use crate::testing::{Test, TestCategory, TestEnvironment, TestResult, TestStatus};
use crate::testing::fixtures::{TestFixture, SimulationTestFixture, HardwareTestFixture, CombinedTestFixture};
use crate::testing::mocks::{MockDevice, MockDisplayDevice, MockCameraDevice, MockImuDevice};
use crate::testing::utils::{assert_approx_eq, assert_vec3_approx_eq, measure_time, TestLogger};
use crate::testing::unit_tests::UnitTest;

use crate::ipc::common::message::{Message, MessageType, MessageHeader, MessagePayload};
use crate::ipc::common::error::IpcError;
use crate::ipc::common::serialization::{Serializable, Deserializable};
use crate::ipc::unix_socket::{UnixSocketConnection, UnixSocketServer, UnixSocketClient};
use crate::ipc::dbus::{DbusInterface, DbusObject, DbusService, DbusClient};
use crate::ipc::websocket::{WebSocketProtocol, WebSocketConnection, WebSocketServer, WebSocketClient};

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::thread;

/// Add IPC tests to the test suite
pub fn add_tests(suite: &mut crate::testing::TestSuite) {
    // Add message tests
    add_message_tests(suite);
    
    // Add serialization tests
    add_serialization_tests(suite);
    
    // Add Unix socket tests
    add_unix_socket_tests(suite);
    
    // Add D-Bus tests
    add_dbus_tests(suite);
    
    // Add WebSocket tests
    add_websocket_tests(suite);
}

/// Add message tests to the test suite
fn add_message_tests(suite: &mut crate::testing::TestSuite) {
    // Test message creation and properties
    let sim_fixture = SimulationTestFixture::new("message_creation_sim");
    let message_creation_test = UnitTest::new(
        "message_creation",
        "Test message creation and properties",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a message
            let header = MessageHeader::new(
                "sender_id",
                "receiver_id",
                MessageType::Request,
                "test_action",
                1,
            );
            
            let payload = MessagePayload::new("test payload");
            
            let message = Message::new(header, payload);
            
            // Check message properties
            assert_eq!(message.header().sender_id(), "sender_id", "Unexpected sender ID");
            assert_eq!(message.header().receiver_id(), "receiver_id", "Unexpected receiver ID");
            assert_eq!(message.header().message_type(), MessageType::Request, "Unexpected message type");
            assert_eq!(message.header().action(), "test_action", "Unexpected action");
            assert_eq!(message.header().sequence_number(), 1, "Unexpected sequence number");
            assert_eq!(message.payload().data(), "test payload", "Unexpected payload data");
            
            // Create a response message
            let response = message.create_response("response payload");
            
            // Check response properties
            assert_eq!(response.header().sender_id(), "receiver_id", "Response sender ID should be original receiver ID");
            assert_eq!(response.header().receiver_id(), "sender_id", "Response receiver ID should be original sender ID");
            assert_eq!(response.header().message_type(), MessageType::Response, "Response should have Response type");
            assert_eq!(response.header().action(), "test_action", "Response should have same action");
            assert_eq!(response.header().sequence_number(), 1, "Response should have same sequence number");
            assert_eq!(response.payload().data(), "response payload", "Unexpected response payload data");
            
            // Create test result
            TestResult::new(
                "message_creation",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Message creation and properties test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(message_creation_test);
}

/// Add serialization tests to the test suite
fn add_serialization_tests(suite: &mut crate::testing::TestSuite) {
    // Test message serialization and deserialization
    let sim_fixture = SimulationTestFixture::new("message_serialization_sim");
    let message_serialization_test = UnitTest::new(
        "message_serialization",
        "Test message serialization and deserialization",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a message
            let header = MessageHeader::new(
                "sender_id",
                "receiver_id",
                MessageType::Request,
                "test_action",
                1,
            );
            
            let payload = MessagePayload::new("test payload");
            
            let message = Message::new(header, payload);
            
            // Serialize the message
            let serialized = message.serialize();
            assert!(!serialized.is_empty(), "Serialized message should not be empty");
            
            // Deserialize the message
            let deserialized = Message::deserialize(&serialized);
            assert!(deserialized.is_ok(), "Deserialization failed: {:?}", deserialized.err());
            
            let deserialized_message = deserialized.unwrap();
            
            // Check deserialized message properties
            assert_eq!(deserialized_message.header().sender_id(), "sender_id", "Unexpected deserialized sender ID");
            assert_eq!(deserialized_message.header().receiver_id(), "receiver_id", "Unexpected deserialized receiver ID");
            assert_eq!(deserialized_message.header().message_type(), MessageType::Request, "Unexpected deserialized message type");
            assert_eq!(deserialized_message.header().action(), "test_action", "Unexpected deserialized action");
            assert_eq!(deserialized_message.header().sequence_number(), 1, "Unexpected deserialized sequence number");
            assert_eq!(deserialized_message.payload().data(), "test payload", "Unexpected deserialized payload data");
            
            // Create test result
            TestResult::new(
                "message_serialization",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Message serialization and deserialization test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(message_serialization_test);
}

/// Add Unix socket tests to the test suite
fn add_unix_socket_tests(suite: &mut crate::testing::TestSuite) {
    // Test Unix socket connection
    let sim_fixture = SimulationTestFixture::new("unix_socket_connection_sim");
    let unix_socket_connection_test = UnitTest::new(
        "unix_socket_connection",
        "Test Unix socket connection",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a temporary socket path
            let socket_path = format!("/tmp/vr_test_socket_{}", rand::random::<u64>());
            
            // Create a server in a separate thread
            let server_path = socket_path.clone();
            let server_thread = thread::spawn(move || {
                // Create a server
                let mut server = UnixSocketServer::new(&server_path);
                
                // Start the server
                if let Err(e) = server.start() {
                    return Err(format!("Failed to start server: {:?}", e));
                }
                
                // Accept a connection
                let connection = server.accept();
                if let Err(e) = connection {
                    return Err(format!("Failed to accept connection: {:?}", e));
                }
                let mut connection = connection.unwrap();
                
                // Receive a message
                let message = connection.receive();
                if let Err(e) = message {
                    return Err(format!("Failed to receive message: {:?}", e));
                }
                let message = message.unwrap();
                
                // Check message properties
                if message.header().sender_id() != "client" {
                    return Err(format!("Unexpected sender ID: {}", message.header().sender_id()));
                }
                if message.header().receiver_id() != "server" {
                    return Err(format!("Unexpected receiver ID: {}", message.header().receiver_id()));
                }
                if message.header().message_type() != MessageType::Request {
                    return Err(format!("Unexpected message type: {:?}", message.header().message_type()));
                }
                if message.header().action() != "test_action" {
                    return Err(format!("Unexpected action: {}", message.header().action()));
                }
                if message.payload().data() != "test payload" {
                    return Err(format!("Unexpected payload data: {}", message.payload().data()));
                }
                
                // Send a response
                let response = message.create_response("response payload");
                if let Err(e) = connection.send(&response) {
                    return Err(format!("Failed to send response: {:?}", e));
                }
                
                // Stop the server
                if let Err(e) = server.stop() {
                    return Err(format!("Failed to stop server: {:?}", e));
                }
                
                Ok(())
            });
            
            // Give the server time to start
            thread::sleep(Duration::from_millis(100));
            
            // Create a client
            let mut client = UnixSocketClient::new("client");
            
            // Connect to the server
            let connection = client.connect(&socket_path);
            assert!(connection.is_ok(), "Failed to connect to server: {:?}", connection.err());
            let mut connection = connection.unwrap();
            
            // Create a message
            let header = MessageHeader::new(
                "client",
                "server",
                MessageType::Request,
                "test_action",
                1,
            );
            
            let payload = MessagePayload::new("test payload");
            
            let message = Message::new(header, payload);
            
            // Send the message
            let send_result = connection.send(&message);
            assert!(send_result.is_ok(), "Failed to send message: {:?}", send_result.err());
            
            // Receive the response
            let response = connection.receive();
            assert!(response.is_ok(), "Failed to receive response: {:?}", response.err());
            let response = response.unwrap();
            
            // Check response properties
            assert_eq!(response.header().sender_id(), "server", "Unexpected response sender ID");
            assert_eq!(response.header().receiver_id(), "client", "Unexpected response receiver ID");
            assert_eq!(response.header().message_type(), MessageType::Response, "Unexpected response message type");
            assert_eq!(response.header().action(), "test_action", "Unexpected response action");
            assert_eq!(response.header().sequence_number(), 1, "Unexpected response sequence number");
            assert_eq!(response.payload().data(), "response payload", "Unexpected response payload data");
            
            // Wait for the server thread to finish
            let server_result = server_thread.join().unwrap();
            assert!(server_result.is_ok(), "Server error: {}", server_result.err().unwrap());
            
            // Clean up the socket file
            let _ = fs::remove_file(&socket_path);
            
            // Create test result
            TestResult::new(
                "unix_socket_connection",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "Unix socket connection test successful",
                0,
            )
        },
        500,
    );
    suite.add_test(unix_socket_connection_test);
}

/// Add D-Bus tests to the test suite
fn add_dbus_tests(suite: &mut crate::testing::TestSuite) {
    // Test D-Bus interface
    let sim_fixture = SimulationTestFixture::new("dbus_interface_sim");
    let dbus_interface_test = UnitTest::new(
        "dbus_interface",
        "Test D-Bus interface",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a D-Bus interface
            let interface = DbusInterface::new("org.vr.headset.Test");
            
            // Add methods
            interface.add_method("TestMethod", vec!["string", "int32"], vec!["string"]);
            
            // Add signals
            interface.add_signal("TestSignal", vec!["string", "int32"]);
            
            // Add properties
            interface.add_property("TestProperty", "string", true, true);
            
            // Check interface properties
            assert_eq!(interface.name(), "org.vr.headset.Test", "Unexpected interface name");
            
            // Check methods
            let methods = interface.methods();
            assert!(methods.contains_key("TestMethod"), "Method not found");
            let method = methods.get("TestMethod").unwrap();
            assert_eq!(method.input_signature(), "si", "Unexpected method input signature");
            assert_eq!(method.output_signature(), "s", "Unexpected method output signature");
            
            // Check signals
            let signals = interface.signals();
            assert!(signals.contains_key("TestSignal"), "Signal not found");
            let signal = signals.get("TestSignal").unwrap();
            assert_eq!(signal.signature(), "si", "Unexpected signal signature");
            
            // Check properties
            let properties = interface.properties();
            assert!(properties.contains_key("TestProperty"), "Property not found");
            let property = properties.get("TestProperty").unwrap();
            assert_eq!(property.signature(), "s", "Unexpected property signature");
            assert!(property.readable(), "Property should be readable");
            assert!(property.writable(), "Property should be writable");
            
            // Create test result
            TestResult::new(
                "dbus_interface",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "D-Bus interface test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(dbus_interface_test);
}

/// Add WebSocket tests to the test suite
fn add_websocket_tests(suite: &mut crate::testing::TestSuite) {
    // Test WebSocket protocol
    let sim_fixture = SimulationTestFixture::new("websocket_protocol_sim");
    let websocket_protocol_test = UnitTest::new(
        "websocket_protocol",
        "Test WebSocket protocol",
        TestEnvironment::Simulation,
        sim_fixture,
        |fixture| {
            // Create a WebSocket protocol
            let protocol = WebSocketProtocol::new("vr-headset-protocol");
            
            // Check protocol properties
            assert_eq!(protocol.name(), "vr-headset-protocol", "Unexpected protocol name");
            
            // Create a message
            let header = MessageHeader::new(
                "sender_id",
                "receiver_id",
                MessageType::Request,
                "test_action",
                1,
            );
            
            let payload = MessagePayload::new("test payload");
            
            let message = Message::new(header, payload);
            
            // Encode the message
            let encoded = protocol.encode_message(&message);
            assert!(!encoded.is_empty(), "Encoded message should not be empty");
            
            // Decode the message
            let decoded = protocol.decode_message(&encoded);
            assert!(decoded.is_ok(), "Decoding failed: {:?}", decoded.err());
            
            let decoded_message = decoded.unwrap();
            
            // Check decoded message properties
            assert_eq!(decoded_message.header().sender_id(), "sender_id", "Unexpected decoded sender ID");
            assert_eq!(decoded_message.header().receiver_id(), "receiver_id", "Unexpected decoded receiver ID");
            assert_eq!(decoded_message.header().message_type(), MessageType::Request, "Unexpected decoded message type");
            assert_eq!(decoded_message.header().action(), "test_action", "Unexpected decoded action");
            assert_eq!(decoded_message.header().sequence_number(), 1, "Unexpected decoded sequence number");
            assert_eq!(decoded_message.payload().data(), "test payload", "Unexpected decoded payload data");
            
            // Create test result
            TestResult::new(
                "websocket_protocol",
                TestCategory::Unit,
                TestEnvironment::Simulation,
                TestStatus::Passed,
                "WebSocket protocol test successful",
                0,
            )
        },
        100,
    );
    suite.add_test(websocket_protocol_test);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        // Create a message
        let header = MessageHeader::new(
            "sender_id",
            "receiver_id",
            MessageType::Request,
            "test_action",
            1,
        );
        
        let payload = MessagePayload::new("test payload");
        
        let message = Message::new(header, payload);
        
        // Check message properties
        assert_eq!(message.header().sender_id(), "sender_id");
        assert_eq!(message.header().receiver_id(), "receiver_id");
        assert_eq!(message.header().message_type(), MessageType::Request);
        assert_eq!(message.header().action(), "test_action");
        assert_eq!(message.header().sequence_number(), 1);
        assert_eq!(message.payload().data(), "test payload");
    }
    
    #[test]
    fn test_message_serialization() {
        // Create a message
        let header = MessageHeader::new(
            "sender_id",
            "receiver_id",
            MessageType::Request,
            "test_action",
            1,
        );
        
        let payload = MessagePayload::new("test payload");
        
        let message = Message::new(header, payload);
        
        // Serialize the message
        let serialized = message.serialize();
        assert!(!serialized.is_empty());
        
        // Deserialize the message
        let deserialized = Message::deserialize(&serialized);
        assert!(deserialized.is_ok());
        
        let deserialized_message = deserialized.unwrap();
        
        // Check deserialized message properties
        assert_eq!(deserialized_message.header().sender_id(), "sender_id");
        assert_eq!(deserialized_message.header().receiver_id(), "receiver_id");
        assert_eq!(deserialized_message.header().message_type(), MessageType::Request);
        assert_eq!(deserialized_message.header().action(), "test_action");
        assert_eq!(deserialized_message.header().sequence_number(), 1);
        assert_eq!(deserialized_message.payload().data(), "test payload");
    }
}

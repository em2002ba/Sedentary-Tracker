use crate::models::{ProcessedState, RawReading};
use redis::AsyncCommands;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Duration;
use tokio::sync::broadcast;

// CLASSIFICATION THRESHOLDS
const THRESH_FIDGET: f32 = 0.020; // Smoothed acceleration delta threshold for fidgeting
const THRESH_ACTIVE: f32 = 0.040; // Smoothed acceleration delta threshold for active
const SMOOTHING_WINDOW: usize = 10; // Number of samples for smoothing buffer

// ALERT CONFIGURATION
const ALERT_LIMIT_SEC: u64 = 1200; // 20 minutes

/// Classifies activity state based on PIR and smoothed acceleration
fn classify_state(pir: i32, smoothed_acc: f32) -> &'static str {
    if pir == 1 || smoothed_acc > THRESH_ACTIVE {
        "ACTIVE"
    } else if smoothed_acc > THRESH_FIDGET {
        "FIDGET"
    } else {
        "SEDENTARY"
    }
}

pub fn spawn_serial_listener(tx: broadcast::Sender<String>, redis_client: redis::Client) {
    thread::spawn(move || {
        let port_name = "/dev/ttyACM0";
        let baud_rate = 115200;

        println!("Connecting to Arduino at {}...", port_name);

        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(1000))
            .open();

        // Create a dedicated async runtime for the serial thread
        let rt = tokio::runtime::Runtime::new().unwrap();

        // State tracking
        let mut acc_buffer: VecDeque<f32> = VecDeque::with_capacity(SMOOTHING_WINDOW);
        let mut sedentary_timer: u64 = 0;
        let mut last_second: Option<String> = None;

        match port {
            Ok(p) => {
                println!("Serial Connected! Processing raw sensor data...");
                let mut reader = BufReader::new(p);
                let mut line = String::new();

                loop {
                    line.clear();
                    if let Ok(bytes_read) = reader.read_line(&mut line) {
                        if bytes_read == 0 {
                            continue;
                        }

                        let clean_line = line.trim();
                        if clean_line.starts_with('{') {
                            // Parse raw Arduino data
                            if let Ok(reading) = serde_json::from_str::<RawReading>(clean_line) {
                                // Add to smoothing buffer
                                if acc_buffer.len() >= SMOOTHING_WINDOW {
                                    acc_buffer.pop_front();
                                }
                                acc_buffer.push_back(reading.acc);

                                // Calculate smoothed acceleration (mean of buffer)
                                let smoothed_acc: f32 = if acc_buffer.is_empty() {
                                    0.0
                                } else {
                                    acc_buffer.iter().sum::<f32>() / acc_buffer.len() as f32
                                };

                                // Classify state
                                let state = classify_state(reading.pir, smoothed_acc);

                                // Update sedentary timer (once per second based on timestamp)
                                let current_second = reading.ts.clone();
                                if last_second.as_ref() != Some(&current_second) {
                                    last_second = Some(current_second);

                                    match state {
                                        "ACTIVE" => sedentary_timer = 0,     // Reset on activity
                                        "FIDGET" => {}                       // Pause
                                        "SEDENTARY" => sedentary_timer += 1, // Increment
                                        _ => {}
                                    }
                                }

                                // Build processed output
                                let output = ProcessedState {
                                    state: state.to_string(),
                                    timer: sedentary_timer,
                                    val: smoothed_acc,
                                    alert: sedentary_timer >= ALERT_LIMIT_SEC,
                                    timestamp: reading.ts,
                                };

                                let json_out = serde_json::to_string(&output).unwrap();

                                // Broadcast to WebSocket and cache in Redis
                                rt.block_on(async {
                                    // Redis cache for reconnection
                                    if let Ok(mut con) =
                                        redis_client.get_multiplexed_async_connection().await
                                    {
                                        let _: () = con
                                            .lpush("sensor_history", &json_out)
                                            .await
                                            .unwrap_or(());
                                        let _: () =
                                            con.ltrim("sensor_history", 0, 99).await.unwrap_or(());
                                    }
                                    // Push to WebSocket
                                    let _ = tx.send(json_out);
                                });
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Serial Error: {}", e),
        }
    });
}

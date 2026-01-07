# Sedentary Activity Tracker

A real-time sedentary behavior monitoring system that tracks physical activity using an Arduino with MPU6050 accelerometer and PIR motion sensor. The system classifies activity into three states (Active, Fidgeting, Sedentary) and alerts users after prolonged inactivity.



##  Table of Contents

- [Features](#-features)
- [Architecture](#-architecture)
- [Hardware Requirements](#-hardware-requirements)
- [Installation](#-installation)
- [Usage](#-usage)
- [API Endpoints](#-api-endpoints)
- [Configuration](#-configuration)
- [Project Structure](#-project-structure)

---

##  Features

###  Core Features

| Feature | Description |
|---------|-------------|
| **3-State Activity Classification** | Classifies movement into ACTIVE (walking), FIDGET (small movements), and SEDENTARY (still) states |
| **Real-Time Monitoring** | 10Hz sensor sampling with live WebSocket streaming to browser |
| **Sedentary Timer** | Counts inactive time, pauses during fidgeting, resets on activity |
| **20-Minute Alert** | Triggers notification after prolonged sedentary behavior |
| **Instant Reconnection** | Redis caches last 100 readings for immediate graph population on page load |

### Dashboard Features

| Component | Description |
|-----------|-------------|
| **Activity Status Indicator** | Large visual indicator showing current state |
| **Sedentary Timer Display** | Shows MM:SS format of current inactivity duration |
| **Activity Timeline** | Color-coded bar chart showing state history |
| **Acceleration Graph** | Real-time line chart of smoothed accelerometer delta values |
| **Session Summary** | Donut chart showing Active vs Inactive time percentage |
| **Alert History** | List of triggered sedentary alerts with timestamps |
| **Statistics Cards** | Total readings, active percentage, longest inactive period, alert count |

###  Healthcare Integration

| Feature | Description |
|---------|-------------|
| **FHIR Compliance** | REST API returns data in HL7 FHIR Observation format |
| **LOINC Codes** | Standardized medical coding for interoperability |
| **Hospital Ready** | Can integrate with Electronic Health Record (EHR) systems |

###  Machine Learning (Nightly Analysis)

| Feature | Description |
|---------|-------------|
| **KMeans Clustering** | Identifies behavioral patterns from daily data |
| **Adaptive Thresholds** | Suggests calibration adjustments based on user patterns |
| **Daily Summaries** | Calculates activity score (0-100) and dominant state |

---

##  Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              TWO-PATH ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────┐        ┌─────────────────────────────────────────────┐   │
│   │   Arduino   │        │              Rust Server (:8000)            │   │
│   │             │ Serial │                                             │   │
│   │  MPU6050    │───────►│  serial.rs ──┬──► Redis ──► WebSocket ──────┼───┼──► Browser
│   │  PIR Sensor │ 115200 │              │   (Cache)   (Real-time)      │   │   (D3.js)
│   │  RTC Clock  │  JSON  │              │                              │   │
│   └─────────────┘        │              └──► PostgreSQL ───────────────┼───┼──► FHIR API
│                          │                  (Storage)                  │   │
│                          └─────────────────────────────────────────────┘   │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                    Python ML Service (Nightly)                      │   │
│   │                                                                     │   │
│   │   PostgreSQL ──► Pandas ──► KMeans ──► Daily Summary ──► PostgreSQL │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **Arduino** samples sensors at 10Hz and sends JSON: 
2. **Rust Server** receives via serial port and broadcasts to two paths:
   - **Path 1 (Real-Time)**: Redis cache → WebSocket → Browser dashboard
   - **Path 2 (Storage)**: PostgreSQL database for persistence and analysis
3. **Frontend** receives WebSocket messages and updates D3.js charts in real-time
4. **ML Service** runs nightly to analyze patterns and generate daily summaries

---

## Hardware Requirements

| Component | Model | Purpose |
|-----------|-------|---------|
| Microcontroller | Arduino Uno/Nano | Main processor |
| Accelerometer | MPU6050 (GY-521) | Motion detection (I2C address: 0x69) |
| Motion Sensor | HC-SR501 PIR | Large body movement detection |
| Real-Time Clock | DS3231 RTC | Timestamps |

### Wiring

| Component | Arduino Pin |
|-----------|-------------|
| MPU6050 SDA | A4 |
| MPU6050 SCL | A5 |
| DS3231 SDA | A4 (shared I2C) |
| DS3231 SCL | A5 (shared I2C) |
| PIR OUT | D7 |

---

## Installation

### Prerequisites

- Rust (1.70+)
- Docker & Docker Compose
- Arduino IDE
- Python 3.10+ (for ML service)
- PostgreSQL client (`psql`)
- sqlx-cli (`cargo install sqlx-cli`)

### 1. Clone Repository

```bash
git clone <repository-url>
cd sedentary_tracker
```

### 2. Install Development Tools

```bash
# Install PostgreSQL client
sudo apt-get install postgresql-client

# Install sqlx-cli
cargo install --version='~0.7' sqlx-cli --no-default-features --features rustls,postgres
```

### 3. Start Database Services

**Option A: Using init script (recommended)**
```bash
./scripts/init_db.sh
```

**Option B: Using Docker Compose**
```bash
docker-compose up -d
```

This starts:
- PostgreSQL on port 5432
- Redis on port 6379

### 4. Run Database Migration

```bash
# If using init script, migrations run automatically
# Otherwise:
export DATABASE_URL=postgres://postgres:password@localhost:5432/sedentary_tracker
sqlx database create
sqlx migrate run
```

### 5. Configure Environment

The `.env` file is pre-configured:

```env
DATABASE_URL=postgres://postgres:password@localhost:5432/sedentary_tracker
SERVER_ADDRESS=0.0.0.0:8000
```

### 5. Upload Arduino Code

1. Open Arduino IDE
2. Install libraries: `Adafruit MPU6050`, `RTClib`
3. Upload the sketch from `arduino/sedentary_tracker.ino`
4. Note the serial port (e.g., `/dev/ttyACM0`)

### 6. Build and Run Server

```bash
# Build release version
cargo build --release

# Set serial port permissions
sudo chmod 666 /dev/ttyACM0

# Run server
./target/release/server
```

### 7. Open Dashboard

Navigate to: **http://localhost:8000**

---

##  Usage

### Starting the System

```bash
# 1. Start containers
docker-compose up -d

# 2. Run server
./target/release/server
```

### Dashboard Indicators

| State | Color | Icon | Timer Behavior |
|-------|-------|------|----------------|
| **ACTIVE** |  Green |  | Resets to 0 |
| **FIDGET** |  Yellow | | Pauses (no change) |
| **SEDENTARY** |  Red | ⏸️ | Counts up |

### Classification Thresholds

| Threshold | Value | Meaning |
|-----------|-------|---------|
| `THRESH_FIDGET` | 0.020 | Acceleration delta above this = Fidgeting |
| `THRESH_ACTIVE` | 0.040 | Acceleration delta above this = Active |
| `ALERT_LIMIT` | 1200s | 20 minutes triggers sedentary alert |

---

## 🔌 API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Serves the D3.js dashboard |
| `/ws` | WebSocket | Real-time sensor data stream |
| `/api/fhir/observation/latest` | GET | Latest reading in FHIR format |
| `/health` | GET | Server health check |

### WebSocket Message Format

```json
{
  "state": "SEDENTARY",
  "timer": 123,
  "val": 0.015,
  "alert": false,
  "timestamp": "14:30:25"
}
```

### FHIR Response Format

```json
[
  {
    "resourceType": "Observation",
    "id": "123-state",
    "status": "final",
    "code": {
      "coding": [{
        "system": "http://loinc.org",
        "code": "CUSTOM-STATE",
        "display": "Sedentary State"
      }]
    },
    "valueString": "SEDENTARY"
  },
  {
    "resourceType": "Observation",
    "id": "123-timer",
    "code": {
      "coding": [{
        "system": "http://loinc.org",
        "code": "CUSTOM-TIMER",
        "display": "Inactive Duration (Seconds)"
      }]
    },
    "valueInteger": 123
  }
]
```

---

## ⚙️ Configuration

### Server Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | Required | PostgreSQL connection string |
| `SERIAL_PORT` | `/dev/ttyACM0` | Arduino serial port |
| `BAUD_RATE` | 115200 | Serial communication speed |
| `ALERT_LIMIT_SEC` | 1200 | Seconds before alert (20 min) |

### Arduino Configuration

| Constant | Default | Description |
|----------|---------|-------------|
| `PIR_PIN` | 7 | PIR sensor digital pin |
| `MPU_ADDR` | 0x69 | I2C address of MPU6050 |
| `SAMPLE_RATE_MS` | 100 | Sampling interval (10Hz) |
| `THRESHOLD_FIDGET` | 0.020 | Fidget detection threshold |
| `THRESHOLD_ACTIVE` | 0.040 | Activity detection threshold |

---

## Project Structure

```
sedentary_tracker/
├── README.md                  # This file
├── Cargo.toml                 # Rust workspace manifest
├── docker-compose.yml         # PostgreSQL + Redis containers
├── .env                       # Environment variables
│
├── server/                    # Rust backend
│   └── src/
│       ├── main.rs            # Entry point, routes
│       ├── state.rs           # Shared application state
│       ├── serial.rs          # Arduino serial reader
│       ├── models.rs          # Data structures
│       ├── models_tests.rs    # Unit tests for models
│       ├── db_worker.rs       # Async database writer
│       ├── websocket.rs       # WebSocket handler
│       ├── fhir.rs            # FHIR API endpoint
│       └── fhir_tests.rs      # Unit tests for FHIR
│
├── frontend/                  # Web dashboard
│   ├── index.html             # Dashboard HTML
│   ├── styles.css             # Dark theme CSS
│   └── app.js                 # D3.js charts + WebSocket
│
├── db/                        # Database utilities
│   └── src/lib.rs             # Connection pool
│   └── tests/
│       └── integration_test.rs # Database integration tests
│
├── logic/                     # Signal processing
│   └── src/
│       ├── lib.rs             # Hjorth parameters, stationarity
│       └── tests.rs           # Unit tests
│   └── tests/
│       └── integration_test.rs # Integration tests
│
├── errors/                    # Utility functions
│   └── src/
│       ├── lib.rs             # Math utilities
│       └── tests.rs           # Unit tests
│   └── tests/
│       └── integration_test.rs # Integration tests
│
├── scripts/                   # DevOps scripts
│   └── init_db.sh             # Database initialization
│
├── ml_classification/         # Python ML service
│   ├── model_classification.py
│   └── venv/                  # Python virtual environment
│
└── migrations/                # SQL migrations
    └── 20260101165438_create_observations.sql
```

---

## 🗄️ Database Schema

### `sedentary_log` (Real-time data)

| Column | Type | Description |
|--------|------|-------------|
| `id` | SERIAL | Primary key |
| `state` | VARCHAR(20) | ACTIVE, FIDGET, or STILL |
| `timer_seconds` | INTEGER | Sedentary timer value |
| `acceleration_val` | REAL | Smoothed acceleration delta |
| `created_at` | TIMESTAMPTZ | Timestamp |

### `activity_summary` (Daily summaries)

| Column | Type | Description |
|--------|------|-------------|
| `date` | DATE | Summary date (unique) |
| `sedentary_minutes` | REAL | Total sedentary time |
| `active_minutes` | REAL | Total active time |
| `dominant_state` | VARCHAR | Most common state |
| `activity_score` | INTEGER | Health score 0-100 |

---

## Testing

This project has a comprehensive test suite with **74 tests** covering unit tests, integration tests, and database tests.

### Test Summary

| Crate | Unit Tests | Integration Tests | Total |
|-------|------------|-------------------|-------|
| db | 0 | 4 | 4 |
| errors | 18 | 5 | 23 |
| logic | 16 | 6 | 22 |
| server | 25 | 0 | 25 |
| **Total** | **59** | **15** | **74** |

### Running Tests

```bash
# Run all tests (requires database)
cargo test --all

# Run tests without database dependency
cargo test -p logic -p errors

# Run only database integration tests
cargo test -p db

# Run a specific test
cargo test test_database_persistence
```

### Test Categories

#### Unit Tests (`src/tests.rs`)
- **logic**: Signal processing functions (Hjorth parameters, stationarity checks)
- **errors**: Math utility functions (add, checked_sub, checked_mul, checked_div)
- **server/models**: Data structure serialization/deserialization
- **server/fhir**: FHIR data model serialization

#### Integration Tests (`tests/integration_test.rs`)
- **logic**: End-to-end signal processing workflows
- **errors**: Chained math operations
- **db**: Database CRUD operations against real PostgreSQL

### Database Setup for Tests

```bash
# Initialize test database (Docker + migrations)
./scripts/init_db.sh

# Or manually:
docker run --name sedentary_tracker_db \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=sedentary_tracker \
  -p 5432:5432 -d postgres:15

export DATABASE_URL=postgres://postgres:password@localhost:5432/sedentary_tracker
sqlx database create
sqlx migrate run
```

---

## Pre-commit Hook (CI/CD)

A Git pre-commit hook ensures code quality before every commit:

```bash
# Located at .git/hooks/pre-commit
# Runs automatically on 'git commit'
```

**Checks performed:**
1. **Formatting** - `cargo fmt --check`
2. **Linting** - `cargo clippy -- -D warnings`
3. **Tests** - `cargo test -p logic -p errors`

If any check fails, the commit is blocked.

### Manual Quality Checks

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run all checks manually
cargo fmt -- --check && cargo clippy -- -D warnings && cargo test -p logic -p errors
```

---

## Development Scripts

### `scripts/init_db.sh`

Initializes the development database:

```bash
./scripts/init_db.sh
```

**What it does:**
1. Checks for `psql` and `sqlx-cli`
2. Starts PostgreSQL in Docker container
3. Waits for database to be ready
4. Creates database and runs migrations

**Environment variables:**
| Variable | Default | Description |
|----------|---------|-------------|
| `POSTGRES_USER` | postgres | Database user |
| `POSTGRES_PASSWORD` | password | Database password |
| `POSTGRES_DB` | sedentary_tracker | Database name |
| `POSTGRES_PORT` | 5432 | Port number |
| `SKIP_DOCKER` | (unset) | Set to skip Docker launch |

---

## Manual Testing

### Test Serial Connection

```bash
stty -F /dev/ttyACM0 115200 raw -echo
cat /dev/ttyACM0
```

### Test Redis Cache

```bash
redis-cli lrange sensor_history 0 5
```

### Test Database

```bash
docker exec sedentary_tracker_db psql -U postgres -d sedentary_tracker \
  -c "SELECT * FROM sedentary_log ORDER BY id DESC LIMIT 5"
```

### Test FHIR API

```bash
curl http://localhost:8000/api/fhir/observation/latest
```

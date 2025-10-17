use axum::{Json, response::IntoResponse};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use utoipa::{OpenApi, ToSchema};

/// # API Documentation
///
/// `ApiDoc` generates the OpenAPI specification for the Ground Station API,
/// exposing endpoints for interacting with a ground station running instance.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::add_job,
        crate::root
    ),
    components(
        schemas(JobRequestDTO, TleData)
    ),
    tags(
        (name = "Ground Station API", description = "API for interacting with a running ground station instance")
    )
)]
pub struct ApiDoc;

/// # Two-Line Element (TLE) Data
///
/// Represents the standard orbital elements used to define a satellite's orbit.
/// These three lines are required to accurately track a satellite pass.
///
/// You can find up to date TLE data at []
///
/// ## Example
/// ```text
/// ISS (ZARYA)
/// 1 25544U 98067A   25235.75642456  .00011222  00000+0  20339-3 0  9993
/// 2 25544  51.6355 332.1708 0003307 260.2831  99.7785 15.50129787525648
/// ```
#[derive(Deserialize, ToSchema, Debug)]
pub struct TleData {
    /// Satellite name or catalog ID (first line of a TLE set)
    #[schema(example = "ISS (ZARYA)")]
    tle0: String,
    /// The first data line of the TLE
    #[schema(example = "1 25544U 98067A   25235.75642456  .00011222  00000+0  20339-3 0  9993")]
    tle1: String,
    /// The second data line of the TLE
    #[schema(example = "2 25544  51.6355 332.1708 0003307 260.2831  99.7785 15.50129787525648")]
    tle2: String,
}

/// # Job Request DTO
///
/// Represents the payload for scheduling a **new tracking job**.  
/// A job instructs the ground station to track a specific satellite pass,
/// specifying:
///
/// - **Time window**: When tracking should start and end (`start`, `end`)
/// - **Satellite orbital data**: Two-Line Element set (`tle`)
/// - **Transceiver frequencies**: Downlink (`rx_frequency`) and uplink (`tx_frequency`)
///
/// Example JSON:
/// ```json
/// {
///   "start": "2025-09-19T12:00:00Z",
///   "end": "2025-09-19T12:15:00Z",
///   "tle": {
///     "tle0": "ISS (ZARYA)",
///     "tle1": "1 25544U 98067A   25235.75642456  .00011222  00000+0  20339-3 0  9993",
///     "tle2": "2 25544  51.6355 332.1708 0003307 260.2831  99.7785 15.50129787525648"
///   },
///   "rx_frequency": 145800000,
///   "tx_frequency": 437500000
/// }
/// ```
///
/// ## Notes:
/// - `start` and `end` must be **UTC timestamps** in ISO-8601 format. (Use https://www.utctime.net/ for getting the current UTC timestamp.)
/// - `tle1` and `tle2` **must be exactly 69 characters long** with valid checksums.
/// - `rx_frequency` and `tx_frequency` are expressed in **Hertz**.
#[allow(dead_code)]
#[derive(Deserialize, ToSchema, Debug)]
pub struct JobRequestDTO {
    /// UTC timestamp for when the tracking should **begin**.
    ///
    /// This marks the *Acquisition of Signal* (AOS) time.
    ///
    /// Example: `"2025-09-19T12:00:00Z"`
    #[schema(value_type = String, format = "date-time", example = "2025-09-19T12:00:00Z")]
    start: DateTime<Utc>,

    /// UTC timestamp for when the tracking should **end**.
    ///
    /// This marks the *Loss of Signal* (LOS) time.
    ///
    /// Example: `"2025-09-19T12:15:00Z"`
    #[schema(value_type = String, format = "date-time", example = "2025-09-19T12:15:00Z")]
    end: DateTime<Utc>,

    /// Orbital data (Two-Line Element set) for the satellite to be tracked.
    ///
    /// - `tle0`: Human-readable satellite name or catalog identifier.
    /// - `tle1`: First TLE line (exactly 69 characters).
    /// - `tle2`: Second TLE line (exactly 69 characters).
    ///
    /// Example:
    /// ```json
    /// {
    ///   "tle0": "ISS (ZARYA)",
    ///   "tle1": "1 25544U 98067A   25235.75642456  .00011222  00000+0  20339-3 0  9993",
    ///   "tle2": "2 25544  51.6355 332.1708 0003307 260.2831  99.7785 15.50129787525648"
    /// }
    /// ```
    #[schema(
        example = json!({
            "tle0": "ISS (ZARYA)",
            "tle1": "1 25544U 98067A   25235.75642456  .00011222  00000+0  20339-3 0  9993",
            "tle2": "2 25544  51.6355 332.1708 0003307 260.2831  99.7785 15.50129787525648"
        })
    )]
    tle: TleData,

    /// **Receiver frequency** in Hertz (Hz).
    ///
    /// This is the **downlink frequency** for receiving telemetry or data
    /// from the satellite.
    ///
    /// Examples:
    /// - `145800000` → 145.8 MHz (VHF downlink, common for many CubeSats)
    /// - `437500000` → 437.5 MHz (UHF downlink, common for amateur satellites)
    #[schema(example = 145800000)]
    rx_frequency: f64,

    /// **Transmitter frequency** in Hertz (Hz).
    ///
    /// This is the **uplink frequency** for sending commands to the satellite.
    ///
    /// Examples:
    /// - `437500000` → 437.5 MHz (UHF uplink, common for many satellites)
    #[schema(example = 437500000)]
    tx_frequency: f64,
}

#[utoipa::path(
    post,
    path = "/jobs",
    tag = "Jobs",
    request_body = JobRequestDTO,
    responses(
    )
)]
pub async fn add_job(
    axum::extract::State(job_tx): axum::extract::State<
        tokio::sync::mpsc::UnboundedSender<jobs::Job>,
    >,
    Json(payload): Json<JobRequestDTO>,
) -> impl IntoResponse {
    println!("[API] Received job request: {:#?}", &payload);

    // Convert JobRequestDTO to Job
    let elements = match tracking::Elements::from_tle(
        Some(payload.tle.tle0.clone()),
        payload.tle.tle1.as_bytes(),
        payload.tle.tle2.as_bytes(),
    ) {
        Ok(elements) => elements,
        Err(e) => {
            eprintln!("Failed to parse TLE: {:?}", e);

            return Json(json!({"status": "error", "message": "Invalid TLE data"}));
        }
    };

    let job = jobs::Job {
        timestamp: payload.start,
        elements,
        satellite_name: payload.tle.tle0.clone(),
    };

    // TODO: somehow manage the fact that the job may have been sent successfully, but not scheduled

    // Send job through channel
    if job_tx.send(job).is_err() {
        eprintln!("Failed to send job to scheduler");

        return Json(json!({"status": "error", "message": "Failed to add job to scheduler"}));
    }

    Json(json!({"status": "ok", "message": "Job added to scheduler successfully"}))
}

#[utoipa::path(get, path = "/", tag = "Ground Station", responses())]
pub async fn root() -> impl IntoResponse {
    Json(json!({ "status": "ok", "message": "Ground Station API is running 🚀" }))
}

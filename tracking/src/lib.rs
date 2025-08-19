use std::time::Duration;

use chrono::{DateTime, Utc};
use predict_rs;
use sgp4;

pub type Degrees = f64;
pub type Meters = f64;

/// The observer is the location of the ground station.
#[derive(Clone)]
pub struct Observer {
    /// Ground station latitude, in degrees.
    latitude: Degrees,
    /// Ground station longitude, in degrees.
    longitude: Degrees,
    /// Ground station altitude, in meters.
    altitude: Meters,
}

impl Observer {
    pub fn new(latitude: Degrees, longitude: Degrees, altitude: Meters) -> Self {
        Self {
            latitude,
            longitude,
            altitude,
        }
    }
}

/// The predicted observation.
#[derive(Debug)]
pub struct Observation {
    /// Azimuth, in degrees.
    pub azimuth: Degrees,
    /// Elevation, in degrees.
    pub elevation: Degrees,
}

#[derive(Debug)]
pub enum TrackerError {
    ElementsError(sgp4::ElementsError),
    OrbitPredictionError(predict_rs::orbit::OrbitPredictionError),
}

/// The tracker is used to predict the position of a satellite, given its orbital elements, relative to the ground station.
pub struct Tracker<'a> {
    observer: predict_rs::predict::PredictObserver,
    elements: &'a sgp4::Elements,
    constants: sgp4::Constants,
}

impl<'a> Tracker<'a> {
    /// Create a new tracker given the observer and the satellite's orbital elements.
    pub fn new(observer: &Observer, elements: &'a sgp4::Elements) -> Result<Self, TrackerError> {
        let constants =
            sgp4::Constants::from_elements(elements).map_err(TrackerError::ElementsError)?;

        let observer = predict_rs::predict::PredictObserver {
            name: "".to_string(),
            latitude: observer.latitude * predict_rs::consts::DEG_TO_RAD,
            longitude: observer.longitude * predict_rs::consts::DEG_TO_RAD,
            altitude: observer.altitude,
            min_elevation: 0.0,
        };

        Ok(Self {
            observer,
            elements,
            constants,
        })
    }

    /// Predict the observation of the satellite at a given time.
    pub fn track(&self, at: DateTime<Utc>) -> Result<Observation, TrackerError> {
        let orbit =
            predict_rs::orbit::predict_orbit(self.elements, &self.constants, at.timestamp() as f64)
                .map_err(TrackerError::OrbitPredictionError)?;

        let observation = predict_rs::observer::predict_observe_orbit(&self.observer, &orbit);

        Ok(Observation {
            azimuth: observation.azimuth * predict_rs::consts::RAD_TO_DEG,
            elevation: observation.elevation * predict_rs::consts::RAD_TO_DEG,
        })
    }

    /// Predict the next pass of the satellite over the ground station, starting from a given time and within a specified time window.
    pub fn next_pass(
        &self,
        from: DateTime<Utc>,
        window: Duration,
    ) -> Option<predict_rs::predict::Pass> {
        let oe = predict_rs::predict::ObserverElements {
            observer: &self.observer,
            elements: self.elements,
            constants: &self.constants,
        };

        let start_utc = from.timestamp() as u64;
        let stop_utc = start_utc + window.as_secs();

        let passes =
            predict_rs::observer::get_passes(&oe, start_utc as f64, stop_utc as f64).ok()?;

        passes.passes.into_iter().next()
    }
}

pub fn get_next_pass(
    observer: &Observer,
    elements: &sgp4::Elements,
    from: DateTime<Utc>,
    window: Duration,
) -> Option<predict_rs::predict::Pass> {
    let observer = predict_rs::predict::PredictObserver {
        name: "".to_string(),
        latitude: observer.latitude * predict_rs::consts::DEG_TO_RAD,
        longitude: observer.longitude * predict_rs::consts::DEG_TO_RAD,
        altitude: observer.altitude,
        min_elevation: 0.0,
    };

    let constants = sgp4::Constants::from_elements(elements)
        .map_err(TrackerError::ElementsError)
        .unwrap();

    let oe = predict_rs::predict::ObserverElements {
        observer: &observer,
        elements: elements,
        constants: &constants,
    };

    let start_utc = from.timestamp() as u64;
    let stop_utc = start_utc + window.as_secs();

    let passes = predict_rs::observer::get_passes(&oe, start_utc as f64, stop_utc as f64)
        .ok()
        .unwrap();

    passes.passes.into_iter().next()
}

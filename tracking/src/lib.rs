use predict_rs::{
    consts::{DEG_TO_RAD, RAD_TO_DEG},
    observer, orbit,
    predict::PredictObserver,
};

pub type Degrees = f64;
pub type Meters = f64;

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
    OrbitPredictionError(orbit::OrbitPredictionError),
}

pub struct Tracker<'a> {
    observer: PredictObserver,
    elements: &'a sgp4::Elements,
    constants: sgp4::Constants,
}

impl<'a> Tracker<'a> {
    pub fn new(observer: &Observer, elements: &'a sgp4::Elements) -> Result<Self, TrackerError> {
        let constants =
            sgp4::Constants::from_elements(elements).map_err(TrackerError::ElementsError)?;

        let observer = PredictObserver {
            name: "".to_string(),
            latitude: observer.latitude * DEG_TO_RAD,
            longitude: observer.longitude * DEG_TO_RAD,
            altitude: observer.altitude,
            min_elevation: 0.0,
        };

        Ok(Self {
            observer,
            elements,
            constants,
        })
    }

    pub fn track(&self, at: i64) -> Result<Observation, TrackerError> {
        let orbit = orbit::predict_orbit(self.elements, &self.constants, at as f64)
            .map_err(TrackerError::OrbitPredictionError)?;

        let observation = observer::predict_observe_orbit(&self.observer, &orbit);

        Ok(Observation {
            azimuth: observation.azimuth * RAD_TO_DEG,
            elevation: observation.elevation * RAD_TO_DEG,
        })
    }
}

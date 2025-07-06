use predict_rs::{
    observer, orbit,
    predict::{self, PredictObservation},
};

#[derive(Debug)]
pub enum TrackerError {
    ElementsError(sgp4::ElementsError),
    OrbitPredictionError(orbit::OrbitPredictionError),
}

pub struct Tracker<'a> {
    observer: &'a predict::PredictObserver,
    elements: &'a sgp4::Elements,
    constants: sgp4::Constants,
}

impl<'a> Tracker<'a> {
    pub fn new(
        observer: &'a predict::PredictObserver,
        elements: &'a sgp4::Elements,
    ) -> Result<Self, TrackerError> {
        let constants = sgp4::Constants::from_elements(elements)
            .map_err(|err| TrackerError::ElementsError(err))?;

        Ok(Self {
            observer,
            elements,
            constants,
        })
    }

    pub fn track(&self, at: i64) -> Result<PredictObservation, TrackerError> {
        let orbit = orbit::predict_orbit(self.elements, &self.constants, at as f64)
            .map_err(|err| TrackerError::OrbitPredictionError(err))?;

        let observation = observer::predict_observe_orbit(self.observer, &orbit);

        Ok(observation)
    }
}

pub mod mock;
pub mod serial;

pub trait AntennaController {
    type Error;

    /// Sends data to the antenna.
    fn send(
        &mut self,
        azimuth: f64,
        elevation: f64,
        sat_name: &str,
        downlink_number: i64,
    ) -> Result<(), Self::Error>;
}

use crate::AntennaController;
pub struct MockController;

#[derive(Debug)]
pub struct MockControllerError;

impl AntennaController for MockController {
    type Error = MockControllerError;

    fn send(
        &mut self,
        azimuth: f64,
        elevation: f64,
        sat_name: &str,
        downlink_number: i64,
    ) -> Result<(), Self::Error> {
        println!(
            "SN={},AZ={:.1},EL={:.1},DN={}",
            sat_name, azimuth, elevation, downlink_number
        );

        Ok(())
    }
}

use chrono::{Datelike, Timelike};
use satkit::frametransform::qteme2itrf;
use satkit::sgp4::{GravConst, OpsMode, sgp4_full};
use satkit::types::Vector3;
use satkit::{ITRFCoord, Instant, TLE};
use serialport;
use std::thread::sleep;

/// Calcular el azimut desde un vector NED
fn ned_azimuth(ned: &Vector3) -> f64 {
    let az_rad = ned.y.atan2(ned.x); // y = East, x = North
    (az_rad.to_degrees() + 360.0) % 360.0
}

/// Calcular la elevación desde un vector NED
fn ned_elevation(ned: &Vector3) -> f64 {
    let horizontal_dist = (ned.x.powi(2) + ned.y.powi(2)).sqrt();
    ned.z.atan2(horizontal_dist).to_degrees()
}

/// Función para enviar azimut y elevación a la antena
fn enviar_a_antena(num_satelite: &i32, azimut: f64, elevacion: f64) {
    let comando = format!("AZ={:.1},EL={:.1}", azimut, elevacion);

    println!("{}", comando);

    // Enviar por serial usando crate serialport
}

fn main() {
    /* // Print the directoyr where data will be stored
    println!("Data directory: {:?}", satkit::utils::datadir());
    // Update the data files (download those that are missing; refresh those that are out of date)
    // This will always download the most-recent space weather data and Earth Orientation Parameters
    // Other data files will be skipped if they are already present
    satkit::utils::update_datafiles(None, false); */

    let line1 = "1 43641U 18076A   25152.17401761  .00000566  00000-0  77716-4 0  9991";
    let line2 = "2 43641  97.8889 339.3866 0001435  91.1486 268.9891 14.82150509359666";

    let mut tle = TLE::load_2line(line1, line2).unwrap();

    let update_interval = std::time::Duration::from_secs(20);

    loop {
        // 1. Obtener hora actual
        let now = chrono::Utc::now();

        // 2. Convertir a formato de fecha y hora
        let current_epoch = Instant::from_datetime(
            now.year().try_into().unwrap(),
            now.month().try_into().unwrap(),
            now.day().try_into().unwrap(),
            now.hour().try_into().unwrap(),
            now.minute().try_into().unwrap(),
            now.second().try_into().unwrap(),
        );

        // 3. Propagar posición y velocidad en TEME
        let (pteme, vteme, _errs) = sgp4_full(
            &mut tle,
            &[current_epoch],
            GravConst::WGS84,
            OpsMode::IMPROVED,
        );

        // 4. Convertir de TEME a ITRF (ECEF)
        let q = qteme2itrf(&current_epoch);
        let rot = q.to_rotation_matrix();

        // Estan en m y m/s
        let r_itrf = rot * pteme;
        let v_itrf = rot * vteme;

        let r_km: Vec<f64> = r_itrf.as_slice().iter().map(|x| x / 1000.0).collect();
        let v_km_s: Vec<f64> = v_itrf.as_slice().iter().map(|x| x / 1000.0).collect();

        // Imprimir en formato cartesiano (ECEF)
        /* println!("ITRS R (km): {:?}", r_km);
        println!("ITRS V (km/s): {:?}", v_km_s); */

        // Coordenada del satélite como ITRFCoord
        let sat_coord = ITRFCoord::from_slice(r_itrf.as_slice()).unwrap();

        // // 5. Posición de la estación en ITRF en Buenos Aires
        let estacion = ITRFCoord::from_geodetic_deg(-34.6037, -58.3816, 25.0);

        // 6. Vector NED relativo a la estación
        let ned = sat_coord.to_ned(&estacion);

        // 7. Calcular azimut y elevación
        let azimuth = ned_azimuth(&ned);
        let elevation = ned_elevation(&ned);

        // 8. Enviar azimut y elevación a la antena
        enviar_a_antena(&tle.sat_num, azimuth, elevation);

        // Esperar al siguiente update
        sleep(update_interval);
    }
}

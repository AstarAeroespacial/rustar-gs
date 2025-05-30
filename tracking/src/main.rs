use chrono::{Datelike, Timelike};
use satkit::frametransform::qteme2itrf;
use satkit::sgp4::{GravConst, OpsMode, sgp4, sgp4_full};
use satkit::types::Vector3;
use satkit::{ITRFCoord, Instant, TLE};
use serialport;
use std::thread::sleep;

/// Función para calcular el azimut y la elevación de un vector ENU (East, North, Up)
fn enu_azimuth(enu: &Vector3) -> f64 {
    let az = enu.y.atan2(enu.x).to_degrees();
    (az + 360.0) % 360.0
}

/// Función para calcular la elevación de un vector ENU (East, North, Up)
fn enu_elevation(enu: &Vector3) -> f64 {
    let horizontal_dist = (enu.x.powi(2) + enu.y.powi(2)).sqrt();
    enu.z.atan2(horizontal_dist).to_degrees()
}

/// Función para enviar azimut y elevación a la antena
fn enviar_a_antena(azimut: f64, elevacion: f64) {
    let comando = format!("AZ:{:.2},EL:{:.2}\n", azimut, elevacion);

    println!("Enviando comando a la antena: {}", comando);

    // Enviar por serial usando crate serialport
}

fn main() {
    /* // Print the directoyr where data will be stored
    println!("Data directory: {:?}", satkit::utils::datadir());
    // Update the data files (download those that are missing; refresh those that are out of date)
    // This will always download the most-recent space weather data and Earth Orientation Parameters
    // Other data files will be skipped if they are already present
    satkit::utils::update_datafiles(None, false); */

    let line1 = "1 25544U 98067A   19343.69339541  .00001764  00000-0  38792-4 0  9991";
    let line2 = "2 25544  51.6439 211.2001 0007417  17.6667  85.6398 15.50103472202482";

    let mut tle = TLE::load_2line(line1, line2).unwrap();

    let update_interval = std::time::Duration::from_secs(5);

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
        let (pteme, vteme, errs) = sgp4_full(
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
        println!("ITRS R (km): {:?}", r_km);
        println!("ITRS V (km/s): {:?}", v_km_s);

        // Coordenada del satélite como ITRFCoord
        let sat_coord = ITRFCoord::from_slice(r_itrf.as_slice()).unwrap();

        // // 5. Posición de la estación en ITRF en Buenos Aires
        let estacion = ITRFCoord::from_geodetic_deg(-34.6037, -58.3816, 25.0);

        // 6. Vector ENU relativo a la estación
        let enu = sat_coord.to_enu(&estacion);

        // 7. Calcular azimut y elevación
        let azimuth = enu_azimuth(&enu);
        let elevation = enu_elevation(&enu);

        // 8. Enviar azimut y elevación a la antena
        enviar_a_antena(azimuth, elevation); // <- función tuya

        // Esperar al siguiente update
        sleep(update_interval);
    }
}

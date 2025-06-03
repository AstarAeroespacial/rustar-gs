use chrono::{Datelike, TimeZone, Timelike, Utc};
use satkit::frametransform::qteme2itrf;
use satkit::sgp4::{GravConst, OpsMode, sgp4_full};
use satkit::types::Vector3;
use satkit::{ITRFCoord, Instant, TLE};
use serialport;
use std::thread::sleep;

/// Función para calcular el azimut de un vector ENU (East, North, Up)
fn enu_azimuth(enu: &Vector3) -> f64 {
    let az_rad = enu.x.atan2(enu.y);
    (az_rad.to_degrees() + 360.0) % 360.0
}

/// Función para calcular la elevación de un vector ENU (East, North, Up)
fn enu_elevation(enu: &Vector3) -> f64 {
    let horizontal_dist = (enu.x.powi(2) + enu.y.powi(2)).sqrt();
    let elevation = enu.z.atan2(horizontal_dist).to_degrees();

    elevation
}

fn enviar_a_antena(num_satelite: &i32, azimut: f64, elevacion: f64) {
    // Imprimir los ángulos relativos a los que debe moverse la antena
    println!("Az={:.1}°, El={:.1}°", azimut, elevacion);

    // Aquí se enviaría el comando por el puerto serie o red, dependiendo de tu implementación
}

fn main() {
    // Print the directoyr where data will be stored
    println!("Data directory: {:?}", satkit::utils::datadir());
    // Update the data files (download those that are missing; refresh those that are out of date)
    // This will always download the most-recent space weather data and Earth Orientation Parameters
    // Other data files will be skipped if they are already present
    satkit::utils::update_datafiles(None, true);

    /*     ISS
    let line1 = "1 25544U 98067A   25152.85500044  .00015665  00000-0  28262-3 0  9997";
    let line2 = "2 25544  51.6369  23.0509 0001835 179.7810 180.3180 15.49928440512781"; */

    /*     SAOCOM
    let line1 = "1 43641U 18076A   25153.18667784  .00000524  00000-0  72414-4 0  9996";
    let line2 = "2 43641  97.8889 340.3845 0001448  91.2759 268.8620 14.82151789359810"; */

    let line1 = "1 43641U 18076A   25153.18667784  .00000524  00000-0  72414-4 0  9996";
    let line2 = "2 43641  97.8889 340.3845 0001448  91.2759 268.8620 14.82151789359810";

    let mut tle = TLE::load_2line(line1, line2).unwrap();

    let update_interval = std::time::Duration::from_secs(5);
    // let mut time = Utc.with_ymd_and_hms(2025, 5, 31, 21, 00, 00).unwrap();

    // Posición de la estación en ITRF en Buenos Aires
    let estacion = ITRFCoord::from_geodetic_deg(-34.5829, -58.4829, 25.0);

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

        /*   let r_km: Vec<f64> = r_itrf.as_slice().iter().map(|x| x / 1000.0).collect();
        let v_km_s: Vec<f64> = v_itrf.as_slice().iter().map(|x| x / 1000.0).collect(); */

        // Imprimir en formato cartesiano (ECEF)
        /*  println!("ITRS R (km): {:?}", r_km);
        println!("ITRS V (km/s): {:?}", v_km_s); */

        // Coordenada del satélite como ITRFCoord
        let sat_coord = ITRFCoord::from_slice(r_itrf.as_slice()).unwrap();

        // println!("{}", sat_coord);

        // 5. Vector ENU relativo a la estación
        let enu = sat_coord.to_enu(&estacion);

        // 6. Calcular azimut y elevación
        let azimuth = enu_azimuth(&enu);
        let elevation = enu_elevation(&enu);

        // Verificar si el satélite está fuera de la visual
        if elevation <= 0.0 {
            println!("📡 El satélite ya no es visible. ");
        }

        // 8. Enviar azimut y elevación a la antena
        enviar_a_antena(&tle.sat_num, azimuth, elevation);

        // time = time + chrono::Duration::seconds(5);
        // Esperar al siguiente update
        sleep(update_interval);
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Importa enu_azimuth, enu_elevation y Vector3 (de satkit)
    const EPSILON: f64 = 1e-9; // Para comparaciones de punto flotante

    // Helper para comparar floats con una tolerancia
    fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
        assert!(
            (a - b).abs() < epsilon,
            "Expected {} ({:?}) to be close to {} ({:?})",
            a,
            a.to_bits(),
            b,
            b.to_bits()
        );
    }

    // Tests para enu_azimuth
    #[test]
    fn test_azimuth_north() {
        let v = Vector3::new(0.0, 1.0, 0.0);
        assert_approx_eq(enu_azimuth(&v), 0.0, EPSILON);
    }

    #[test]
    fn test_azimuth_east() {
        let v = Vector3::new(1.0, 0.0, 0.0);
        assert_approx_eq(enu_azimuth(&v), 90.0, EPSILON);
    }

    #[test]
    fn test_azimuth_south() {
        let v = Vector3::new(0.0, -1.0, 0.0);
        assert_approx_eq(enu_azimuth(&v), 180.0, EPSILON);
    }

    #[test]
    fn test_azimuth_west() {
        let v = Vector3::new(-1.0, 0.0, 0.0);
        assert_approx_eq(enu_azimuth(&v), 270.0, EPSILON);
    }

    #[test]
    fn test_azimuth_northeast() {
        let v = Vector3::new(1.0, 1.0, 0.0);
        assert_approx_eq(enu_azimuth(&v), 45.0, EPSILON);
    }

    #[test]
    fn test_azimuth_northwest() {
        let v = Vector3::new(-1.0, 1.0, 0.0);
        assert_approx_eq(enu_azimuth(&v), 315.0, EPSILON);
    }

    #[test]
    fn test_azimuth_southeast() {
        let v = Vector3::new(1.0, -1.0, 0.0);
        assert_approx_eq(enu_azimuth(&v), 135.0, EPSILON);
    }

    #[test]
    fn test_azimuth_southwest() {
        let v = Vector3::new(-1.0, -1.0, 0.0);
        assert_approx_eq(enu_azimuth(&v), 225.0, EPSILON);
    }

    #[test]
    fn test_azimuth_zero_vector_xy() {
        // atan2(0.0, 0.0) es 0.0, por lo que el azimut será 0.0
        let v = Vector3::new(0.0, 0.0, 1.0);
        assert_approx_eq(enu_azimuth(&v), 0.0, EPSILON);
    }

    // Tests para enu_elevation
    #[test]
    fn test_elevation_directly_up() {
        let v = Vector3::new(0.0, 0.0, 1.0);
        assert_approx_eq(enu_elevation(&v), 90.0, EPSILON);
    }

    #[test]
    fn test_elevation_directly_down() {
        let v = Vector3::new(0.0, 0.0, -1.0);
        assert_approx_eq(enu_elevation(&v), -90.0, EPSILON);
    }

    #[test]
    fn test_elevation_horizon_north() {
        let v = Vector3::new(0.0, 1.0, 0.0);
        assert_approx_eq(enu_elevation(&v), 0.0, EPSILON);
    }

    #[test]
    fn test_elevation_horizon_east() {
        let v = Vector3::new(1.0, 0.0, 0.0);
        assert_approx_eq(enu_elevation(&v), 0.0, EPSILON);
    }

    #[test]
    fn test_elevation_45_degrees_up_north_east() {
        // Para 45 grados de elevación, z = distancia_horizontal
        // Distancia horizontal = sqrt(1^2 + 1^2) = sqrt(2)
        let z_val = (1.0_f64.powi(2) + 1.0_f64.powi(2)).sqrt();
        let v = Vector3::new(1.0, 1.0, z_val);
        assert_approx_eq(enu_elevation(&v), 45.0, EPSILON);
    }

    #[test]
    fn test_elevation_45_degrees_up_east() {
        // Distancia horizontal = sqrt(1^2 + 0^2) = 1. Para 45 deg, z = 1.
        let v = Vector3::new(1.0, 0.0, 1.0);
        assert_approx_eq(enu_elevation(&v), 45.0, EPSILON);
    }

    #[test]
    fn test_elevation_30_degrees_up() {
        // Para elevación de 30 grados, z = horizontal_dist * tan(30_deg_rad)
        // tan(30 deg) = 1/sqrt(3). Si z = 1, horizontal_dist = sqrt(3).
        // Podemos elegir x = sqrt(3), y = 0.
        let horizontal_dist = (3.0_f64).sqrt();
        let v = Vector3::new(horizontal_dist, 0.0, 1.0);
        assert_approx_eq(enu_elevation(&v), 30.0, EPSILON);
    }

    #[test]
    fn test_elevation_minus_30_degrees_down() {
        let horizontal_dist = (3.0_f64).sqrt();
        let v = Vector3::new(horizontal_dist, 0.0, -1.0);
        assert_approx_eq(enu_elevation(&v), -30.0, EPSILON);
    }

    #[test]
    fn test_elevation_zero_vector_completely() {
        // Si x, y, z son 0, horizontal_dist es 0.
        // atan2(0.0, 0.0) es 0.0
        let v_zero = Vector3::new(0.0, 0.0, 0.0);
        assert_approx_eq(enu_elevation(&v_zero), 0.0, EPSILON);
    }
}

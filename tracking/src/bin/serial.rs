use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    println!("Esperando conexión de Gpredict en puerto 4533...");
    let listener = TcpListener::bind("127.0.0.1:4533")?;

    // Posición actual de la antena
    let mut azimuth = 0.0;
    let mut elevation = 0.0;

    for stream in listener.incoming() {
        let mut stream = stream?;
        println!("Conexión establecida con Gpredict.");
        let reader = BufReader::new(stream.try_clone()?);

        for line in reader.lines() {
            match line {
                // Responder según el comando
                Ok(mut linea) => {
                    if linea.starts_with("P ") {
                        // Comando P azimuth elevation - mover antena
                        linea = linea.replace(",", ".");
                        let partes: Vec<&str> = linea.split_whitespace().collect();
                        if partes.len() >= 3 {
                            if let (Ok(az), Ok(el)) =
                                (partes[1].parse::<f64>(), partes[2].parse::<f64>())
                            {
                                azimuth = az;
                                elevation = el;
                                println!("📡 Moviendo a: Az={:.1}°, El={:.1}°", azimuth, elevation);
                                stream.write_all(b"RPRT 0\n")?;
                            }
                        }
                    } else if linea == "p" {
                        // Comando p - enviar posición actual
                        /* println!(
                            "📍 Enviando posición: Az={:.1}°, El={:.1}°",
                            azimuth, elevation
                        ); */
                        let respuesta = format!("{:.6}\n{:.6}\n", azimuth, elevation);
                        stream.write_all(respuesta.as_bytes())?;
                    }
                }
                Err(err) => println!("Error al leer línea: {}", err),
            }
        }
    }

    Ok(())
}

use serialport::SerialPort;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port_name = "COM4"; // este recibe lo que Orbitron manda a COM3
    let baud_rate = 9600;

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(std::time::Duration::from_secs(30))
        .open()?;

    println!("Escuchando en {}...", port_name);

    let mut buffer: Vec<u8> = vec![0; 1024];
    loop {
        match port.read(&mut buffer) {
            Ok(n) => {
                let s = String::from_utf8_lossy(&buffer[..n]);
                println!("Recibido: {}", s);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
            Err(e) => return Err(Box::new(e)),
        }
    }
}

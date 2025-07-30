use antenna_controller::AntennaController;
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Duration;

/*
 * Before running this example, use the command in the terminal:
 *
 * socat -d -d pty,raw,echo=0 pty,raw,echo=0
 *
 * to create two virtual ports that simulate communication between a sender and a receiver.
 */

fn main() {
    let sender_port = "/dev/pts/2".to_string();
    let receiver_port = "/dev/pts/3".to_string();
    let baud_rate = 9600;

    // Sender thread
    let sender = thread::spawn(move || {
        let mut controller = AntennaController::new(&sender_port, baud_rate)
            .expect("Failed to open serial port (sender)");

        controller
            .send(b"Hello from sender!\n")
            .expect("Failed to send data");

        println!("Sender: Message sent");
    });

    // Receiver thread
    let receiver = thread::spawn(move || {
        let controller = AntennaController::new(&receiver_port, baud_rate)
            .expect("Failed to open serial port (receiver)");

        thread::sleep(Duration::from_millis(500));

        let mut reader = BufReader::new(controller.port);

        let mut line = String::new();

        match reader.read_line(&mut line) {
            Ok(n) if n > 0 => {
                println!("Receiver: Received line: {}", line.trim_end());
            }
            Ok(_) => println!("Receiver: No data received"),
            Err(e) => println!("Receiver: Failed to read line: {}", e),
        }
    });

    sender.join().unwrap();
    receiver.join().unwrap();
}

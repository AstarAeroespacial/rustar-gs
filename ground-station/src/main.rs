use tokio::{
    sync::mpsc::{Receiver, Sender},
    task,
};

/// Controls and configures the ground station.
trait Controller {
    async fn next_command(&mut self) -> Option<Command>;
}

/// Commands sent to the ground station.
enum Command {
    Ping,
}

/// Messages sent from the ground station.
enum Message {
    Pong,
}

/// Where the ground station sends the stuff it receives.
trait Publisher {
    async fn publish(&self, message: Message);
}

struct ChannelPublisher {
    tx: Sender<Message>,
}

impl ChannelPublisher {
    pub fn new(tx: Sender<Message>) -> Self {
        Self { tx }
    }
}

impl Publisher for ChannelPublisher {
    async fn publish(&self, message: Message) {
        let _ = self.tx.send(message).await;
    }
}

struct GroundStation<C: Controller, P: Publisher> {
    controller: C,
    publisher: P,
}

impl<C: Controller, P: Publisher> GroundStation<C, P> {
    pub fn new(controller: C, publisher: P) -> Self {
        Self {
            controller,
            publisher,
        }
    }

    pub async fn run(&mut self) {
        loop {
            let command = self.controller.next_command().await.unwrap();

            match command {
                Command::Ping => self.publisher.publish(Message::Pong).await,
            }
        }
    }
}

/// Reads controls
struct ChannelController {
    rx: Receiver<Command>,
}

impl ChannelController {
    fn new(rx: Receiver<Command>) -> Self {
        Self { rx }
    }
}

impl Controller for ChannelController {
    async fn next_command(&mut self) -> Option<Command> {
        self.rx.recv().await
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::channel;

    use super::*;

    /// Simple test to test and understand the command -> response flow.
    #[tokio::test]
    async fn ping_pong() {
        let (tx_controller, rx_controller) = channel(1);
        let controller = ChannelController::new(rx_controller);

        let (tx_publisher, mut rx_publisher) = channel(1);
        let publisher = ChannelPublisher::new(tx_publisher);

        let mut gs = GroundStation::new(controller, publisher);

        tokio::spawn(async move { gs.run().await });

        tx_controller.send(Command::Ping).await.unwrap();
        let message = rx_publisher.recv().await.unwrap();

        assert!(matches!(message, Message::Pong));
    }
}

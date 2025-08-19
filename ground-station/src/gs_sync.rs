use std::{
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};

use crate::{Command, GroundStationStateOrConfigOrWhatever, Message};

pub fn run(
    commands: impl Iterator<Item = Command>,
    publisher: Sender<Message>,
    mut passes: impl Iterator<Item = Instant>,
    state: &mut GroundStationStateOrConfigOrWhatever,
) {
    todo!()
}

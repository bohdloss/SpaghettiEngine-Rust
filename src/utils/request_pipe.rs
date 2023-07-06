use std::sync::{Mutex};
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};
use crate::utils::id_type::id_type;
use crate::utils::IsLocked;
use crate::utils::new_empty::NewEmpty;

id_type!(PipeId);

struct PipeData {
    responded: bool
}

impl PipeData {
    fn new() -> Self {
        Self {
            responded: false
        }
    }
}

pub struct RequestPipe<T: NewEmpty> {
    id: PipeId,
    lock: Mutex<()>,
    data: Mutex<PipeData>,
    request_sender: SyncSender<T>,
    request_receiver: Receiver<T>,
    response_sender: SyncSender<T>,
    response_receiver: Receiver<T>,
}

impl<T: NewEmpty> RequestPipe<T> {
    pub fn new() -> Self {
        let (request_sender, request_receiver) = sync_channel(1);
        let (response_sender, response_receiver) = sync_channel(1);

        Self {
            id: PipeId::new(),
            lock: Mutex::new(()),
            data: Mutex::new(PipeData::new()),
            request_sender,
            request_receiver,
            response_sender,
            response_receiver,
        }
    }

    pub fn request(&self, packet: T) -> T {
        self.data.lock().unwrap().responded = false;
        let _lock = self.lock.lock().unwrap();
        let _ = self.request_sender.send(packet);
        match self.response_receiver.recv() {
            Ok(packet) => packet,
            Err(_) => T::new_empty()
        }
    }

    pub fn receive(&self) -> Option<T> {
        // Locked = we're in the middle of a request
        // Not locked = no request current, you should receive nothing
        if !self.lock.is_locked() {
            return None
        }
        match self.request_receiver.try_recv() {
            Ok(packet) => Some(packet),
            Err(_) => None
        }
    }

    pub fn respond(&self, packet: T) {
        // If no request is ongoing, sending an item in the channel
        // would break the state machine
        if !self.lock.is_locked() {
            return;
        }
        let mut data_guard = self.data.lock().unwrap();

        // If we allowed responding multiple times,
        // the state machine would break
        if data_guard.responded {
            return;
        }

        let _ = self.response_sender.send(packet);
        data_guard.responded = true;
    }
}

impl<T: NewEmpty> PartialEq for RequestPipe<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: NewEmpty> Eq for RequestPipe<T> {}
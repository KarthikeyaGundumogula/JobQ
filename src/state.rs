use crate::types::{Index, Job};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};
use tokio::sync::{Mutex, Notify};
use uuid::Uuid;

pub struct AppState {
    pub jobs: Mutex<HashMap<Uuid, Job>>,
    pub index: Mutex<BinaryHeap<Reverse<Index>>>,
    pub notify: Notify,
}

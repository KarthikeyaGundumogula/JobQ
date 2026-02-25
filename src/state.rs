use crate::types::{Index, Job};
use std::collections::{BinaryHeap, HashMap};
use uuid::Uuid;

pub struct AppState {
    pub jobs: HashMap<Uuid, Job>,
    pub index: BinaryHeap<Index>,
}

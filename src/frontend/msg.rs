use serde::{Serialize, Deserialize};
use std::collections::LinkedList;
use crate::PitouFile;

#[derive(Serialize, Deserialize)]
pub enum SearchMsg {
    Active(LinkedList<PitouFile>),
    Terminated(LinkedList<PitouFile>),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum ActivityMsg {
    Active,
    Terminated,
}
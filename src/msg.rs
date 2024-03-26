use crate::PitouFile;
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;

pub enum SearchMsg {
    Active(LinkedList<PitouFile>),
    Terminated(LinkedList<PitouFile>),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum ActivityMsg {
    Active,
    Terminated,
}

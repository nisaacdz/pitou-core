use crate::PitouFile;
use serde::{Deserialize, Serialize};
use std::{collections::LinkedList, time::Duration};

pub enum SearchMsg {
    Active(LinkedList<PitouFile>),
    Terminated(LinkedList<PitouFile>),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum TransferState {
    Initializing(u64),
    Active(TransferSize),
    Terminated(TransferSize),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum TransferMsg {
    Copy {
        id: TransferSessionID,
        state: TransferState,
        time_elapsed: Duration,
    },
    Move {
        id: TransferSessionID,
        state: TransferState,
        time_elapsed: Duration,
    },
}

impl TransferMsg {
    pub fn details(self) -> (TransferState, Duration) {
        match self {
            TransferMsg::Copy {
                id: _,
                state,
                time_elapsed,
            } => (state, time_elapsed),
            TransferMsg::Move {
                id: _,
                state,
                time_elapsed,
            } => (state, time_elapsed),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct TransferSize {
    pub total: u64,
    pub current: u64,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct TransferSessionID {
    pub idx: i64,
    pub parity: i64,
}

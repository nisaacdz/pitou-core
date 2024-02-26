use std::{collections::LinkedList, sync::{Arc, Mutex}};

use crate::PitouFile;

use super::SearchOptions;

mod stream {
    use std::{collections::LinkedList, sync::OnceLock};
    use static_init::dynamic;
    use crate::PitouFile;

    #[dynamic]
    static mut STREAM: Option<LinkedList<PitouFile>> = None;

    pub fn read() -> Option<LinkedList<PitouFile>> {
        match &STREAM.write() {
            Some(v) => ,
            None => None,
        }
    }

}


pub fn search(options: SearchOptions) {
    todo!()
}




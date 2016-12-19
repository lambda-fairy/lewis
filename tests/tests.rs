#![feature(proc_macro)]
#![allow(non_camel_case_types)]

#[macro_use]
extern crate serde_derive;
extern crate tempdir;

extern crate lewis;

use std::collections::HashMap;
use lewis::{Acid, Acidic};
use tempdir::TempDir;

// I wish it could look like this instead...
/*
lewis! {
    impl Ponies {
        fn get_pony(&self, name: String) -> Option<u32> { ... }
        fn set_pony(&mut self, name: String, value: u32) { ... }
    }
}
*/

#[derive(Debug, Default, Deserialize, Serialize)]
struct Ponies {
    ponies: HashMap<String, u32>,
}

impl Acidic for Ponies {
    type QueryEvent = PoniesQueryEvent;
    type QueryOutput = PoniesQueryOutput;
    type UpdateEvent = PoniesUpdateEvent;
    type UpdateOutput = PoniesUpdateOutput;

    fn run_query(&self, event: PoniesQueryEvent) -> PoniesQueryOutput {
        match event {
            PoniesQueryEvent::get_pony(name) => PoniesQueryOutput::get_pony({
                self.ponies.get(&name).cloned()
            }),
        }
    }

    fn run_update(&mut self, event: PoniesUpdateEvent) -> PoniesUpdateOutput {
        match event {
            PoniesUpdateEvent::set_pony(name, value) => PoniesUpdateOutput::set_pony({
                self.ponies.insert(name, value);
            }),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
enum PoniesQueryEvent {
    get_pony(String),
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
enum PoniesQueryOutput {
    get_pony(Option<u32>),
}

#[derive(Debug, Deserialize, Serialize)]
enum PoniesUpdateEvent {
    set_pony(String, u32),
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
enum PoniesUpdateOutput {
    set_pony(()),
}

#[test]
fn smoke() {
    let root = TempDir::new("lewis").unwrap();
    {
        let acid = Acid::<Ponies>::open(root.path()).unwrap();
        let r = acid.update(PoniesUpdateEvent::set_pony("Pinkie Pie".into(), 21)).unwrap();
        assert_eq!(r, PoniesUpdateOutput::set_pony(()));
        let r = acid.query(PoniesQueryEvent::get_pony("Pinkie Pie".into())).unwrap();
        assert_eq!(r, PoniesQueryOutput::get_pony(Some(21)));
    }
    {
        let acid = Acid::<Ponies>::open(root.path()).unwrap();
        let r = acid.query(PoniesQueryEvent::get_pony("Pinkie Pie".into())).unwrap();
        assert_eq!(r, PoniesQueryOutput::get_pony(Some(21)));
    }
}

#![feature(proc_macro)]
#![allow(non_camel_case_types)]

#[macro_use]
extern crate serde_derive;
extern crate tempdir;

#[macro_use]
extern crate lewis;

use lewis::Acid;
use std::collections::HashMap;
use tempdir::TempDir;

#[derive(Debug, Default, Deserialize, Serialize)]
struct Ponies {
    ponies: HashMap<String, u32>,
}

lewis! {
    #[acidic(PoniesQueryEvent, PoniesQueryOutput,
             PoniesUpdateEvent, PoniesUpdateOutput,
             AcidPoniesExt)]
    impl Ponies {
        fn get_pony(&self, name: String) -> Option<u32> {
            self.ponies.get(&name).cloned()
        }
        fn set_pony(&mut self, name: String, value: u32) -> () {
            self.ponies.insert(name, value);
        }
    }
}

trait AcidPoniesExt {
    fn get_pony(&self, name: String) -> lewis::Result<Option<u32>>;
    fn set_pony(&self, name: String, value: u32) -> lewis::Result<()>;
}

impl AcidPoniesExt for Acid<Ponies> {
    fn get_pony(&self, name: String) -> lewis::Result<Option<u32>> {
        Ok(match self.query(PoniesQueryEvent::get_pony(name))? {
            PoniesQueryOutput::get_pony(r) => r,
            // _ => unreachable!()
        })
    }

    fn set_pony(&self, name: String, value: u32) -> lewis::Result<()> {
        Ok(match self.update(PoniesUpdateEvent::set_pony(name, value))? {
            PoniesUpdateOutput::set_pony(r) => r,
            // _ => unreachable!()
        })
    }
}

#[test]
fn smoke() {
    let root = TempDir::new("lewis").unwrap();
    {
        let acid = Acid::<Ponies>::open(root.path()).unwrap();
        let r = acid.set_pony("Pinkie Pie".into(), 21).unwrap();
        assert_eq!(r, ());
        let r = acid.get_pony("Pinkie Pie".into()).unwrap();
        assert_eq!(r, Some(21));
    }
    {
        let acid = Acid::<Ponies>::open(root.path()).unwrap();
        let r = acid.get_pony("Pinkie Pie".into()).unwrap();
        assert_eq!(r, Some(21));
    }
}

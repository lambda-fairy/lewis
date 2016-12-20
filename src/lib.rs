extern crate atomicwrites;
extern crate byteorder;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_cbor;

use atomicwrites::{AtomicFile, AllowOverwrite};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub use serde_cbor::{Error, Result};

mod macros;

pub trait Acidic: Send + Sync + Deserialize + Serialize {
    type QueryEvent: Deserialize + Serialize;
    type QueryOutput: Deserialize + Serialize;
    type UpdateEvent: Deserialize + Serialize;
    type UpdateOutput: Deserialize + Serialize;

    fn run_query(&self, event: Self::QueryEvent) -> Self::QueryOutput;
    fn run_update(&mut self, event: Self::UpdateEvent) -> Self::UpdateOutput;
}

struct Journal<S> {
    file: File,
    _phantom: PhantomData<S>,
}

impl<S: Acidic> Journal<S> {
    fn open<P: AsRef<Path>>(root: P, state: &mut S) -> Result<Journal<S>> {
        let path = root.as_ref().join("journal");
        info!("opening journal at {:?}", path);
        let mut file = OpenOptions::new().read(true).append(true).create(true).open(path)?;
        let mut buffer = Vec::new();
        let mut n_events = 0u64;
        loop {
            // Read the length of the next event
            let mut len_buffer = [0u8; 8];
            match file.read(&mut len_buffer)? {
                0 => break,  // EOF
                8 => {},  // OK
                _ => return Err(serde_cbor::Error::TrailingBytes),
            }
            let len = (&mut &len_buffer[..]).read_u64::<BigEndian>().unwrap();
            // Read the event
            buffer.resize(len as usize, 0);
            file.read_exact(&mut buffer)?;
            let event = serde_cbor::from_slice::<S::UpdateEvent>(&buffer)?;
            // Update the state
            let _ = state.run_update(event);
            n_events += 1
        }
        info!("read {} events successfully", n_events);
        Ok(Journal { file: file, _phantom: PhantomData })
    }

    fn record(&mut self, event: &S::UpdateEvent) -> Result<()> {
        let buffer = serde_cbor::to_vec(event)?;
        self.file.write_u64::<BigEndian>(buffer.len() as u64)?;
        self.file.write_all(&buffer)?;
        self.file.sync_data()?;
        Ok(())
    }
}

struct State<S> {
    state: S,
    path: PathBuf,
}

impl<S: Acidic + Default> State<S> {
    fn open<P: AsRef<Path>>(root: P) -> Result<State<S>> {
        let path = root.as_ref().join("state");
        info!("loading initial state from {:?}", path);
        let state = match File::open(&path) {
            Ok(file) => serde_cbor::from_reader(file)?,
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                warn!("state file not found; creating");
                Default::default()
            },
            Err(e) => return Err(e.into()),
        };
        info!("finished loading state");
        Ok(State { state: state, path: path })
    }
}

impl<S: Acidic> State<S> {
    fn checkpoint(&self) -> Result<()> {
        info!("writing state to {:?}", self.path);
        let afile = AtomicFile::new(&self.path, AllowOverwrite);
        afile.write::<_, Error, _>(|file| {
            let mut writer = BufWriter::new(file);
            serde_cbor::ser::to_writer(&mut writer, &self.state)?;
            writer.flush()?;
            Ok(())
        })?;
        info!("finished writing state");
        Ok(())
    }
}

impl<S> Deref for State<S> {
    type Target = S;
    fn deref(&self) -> &S { &self.state }
}

impl<S> DerefMut for State<S> {
    fn deref_mut(&mut self) -> &mut S { &mut self.state }
}

pub struct Acid<S> { lock: Arc<RwLock<AcidInner<S>>> }

struct AcidInner<S> {
    state: State<S>,
    journal: Journal<S>,
}

impl<S: Acidic + Default> Acid<S> {
    pub fn open<P: AsRef<Path>>(root: P) -> Result<Acid<S>> {
        fs::create_dir_all(&root)?;
        let mut state = State::open(&root)?;
        let journal = Journal::open(&root, &mut *state)?;
        Ok(Acid { lock: Arc::new(RwLock::new(AcidInner {
            state: state,
            journal: journal,
        })) })
    }
}

impl<S: Acidic> Acid<S> {
    pub fn query(&self, event: S::QueryEvent) -> Result<S::QueryOutput> {
        let inner = self.lock.read().unwrap();
        Ok(inner.state.run_query(event))
    }

    pub fn update(&self, event: S::UpdateEvent) -> Result<S::UpdateOutput> {
        let mut inner = self.lock.write().unwrap();
        inner.journal.record(&event)?;
        Ok(inner.state.run_update(event))
    }

    pub fn checkpoint(&self) -> Result<()> {
        self.lock.read().unwrap().state.checkpoint()
    }
}

impl<S> Clone for Acid<S> {
    fn clone(&self) -> Acid<S> {
        Acid { lock: self.lock.clone() }
    }
}

// This macro would have allowed for really pretty invocations, like this:
//
//     query!(Ponies, acid.get_pony("Pinkie Pie".into()))
//
// Unfortunately, it doesn't work as-is since Rust doesn't resolve enum variants
// through type aliases (https://github.com/rust-lang/rust/issues/26264). So we
// leave it commented for now.
/*
#[macro_export]
macro_rules! query {
    ($ty:ty, $acid:ident . $method:ident ( $($arg:expr),* )) => {
        match $acid.query(<$ty as $crate::Acidic>::QueryEvent::$method($($arg),*)) {
            Ok(r) => match r {
                <$ty as $crate::Acidic>::QueryOutput::$method(r) => r,
                _ => unreachable!(),
            },
            Err(e) => Err(e),
        }
    }
}
*/

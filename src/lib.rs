extern crate atomicwrites;
extern crate byteorder;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_cbor;

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

pub use serde_cbor::{Error, Result};

mod macros;

mod local;
pub use local::Local;

pub struct Acid<S> { backend: Arc<Backend<S>> }

impl<S: Acidic + Default> Acid<S> {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Acid<S>> {
        Local::open(path)
    }
}

impl<S: Acidic> Acid<S> {
    pub fn query(&self, event: S::QueryEvent) -> Result<S::QueryOutput> {
        self.backend.query(event)
    }
    pub fn update(&self, event: S::UpdateEvent) -> Result<S::UpdateOutput> {
        self.backend.update(event)
    }
    pub fn checkpoint(&self) -> Result<()> {
        self.backend.checkpoint()
    }
    pub fn from_backend<B: Backend<S>>(backend: B) -> Acid<S> {
        Acid { backend: Arc::new(backend) }
    }
}

impl<S> Clone for Acid<S> {
    fn clone(&self) -> Acid<S> {
        Acid { backend: self.backend.clone() }
    }
}

pub trait Acidic: 'static + Send + Sync + Deserialize + Serialize {
    type QueryEvent: Deserialize + Serialize;
    type QueryOutput: Deserialize + Serialize;
    type UpdateEvent: Deserialize + Serialize;
    type UpdateOutput: Deserialize + Serialize;

    fn run_query(&self, event: Self::QueryEvent) -> Self::QueryOutput;
    fn run_update(&mut self, event: Self::UpdateEvent) -> Self::UpdateOutput;
}

pub trait Backend<S: Acidic>: 'static {
    fn query(&self, event: S::QueryEvent) -> Result<S::QueryOutput>;
    fn update(&self, event: S::UpdateEvent) -> Result<S::UpdateOutput>;
    fn checkpoint(&self) -> Result<()>;
}

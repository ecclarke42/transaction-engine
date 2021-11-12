use std::sync::{Arc, RwLock};

#[cfg(feature = "async-engine")]
use async_trait::async_trait;

use crate::{
    state::{State, UpdateError},
    Action,
};

pub trait SyncEngine {
    fn process(&mut self, action: Action) -> Result<(), UpdateError>;

    fn process_all<I: IntoIterator<Item = Action>>(
        &mut self,
        actions: I,
    ) -> Result<(), UpdateError> {
        for action in actions.into_iter() {
            self.process(action)?
        }
        Ok(())
    }
}

#[cfg(feature = "async-engine")]
#[async_trait]
pub trait AsyncEngine {
    async fn process_async(&self, action: Action);
    // async fn process_stream();
}

#[derive(Debug, Default)]
pub struct SingleThreadedEngine {
    state: State,
}

impl SingleThreadedEngine {
    pub fn new() -> Self {
        Self {
            state: State::new(),
        }
    }
    pub fn state(&self) -> &State {
        &self.state
    }
}
impl SyncEngine for SingleThreadedEngine {
    fn process(&mut self, action: Action) -> Result<(), UpdateError> {
        // Per the assignment, we'll ignore pretty much all errors here, leaving the
        // account unchanged. A more sophisticated system would log the ignored actions
        // on error
        let _ = self.state.update(action);
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct MultiThreadedEngine {
    // Realistically, if we were implementing this, we'd probably use the tokio
    // primitives
    state: Arc<RwLock<State>>,
}

impl MultiThreadedEngine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(State::new())),
        }
    }
    pub fn state(&self) -> Arc<RwLock<State>> {
        self.state.clone()
    }
}

impl SyncEngine for MultiThreadedEngine {
    fn process(&mut self, action: Action) -> Result<(), UpdateError> {
        // TODO: add an error type for lock failures
        let mut state = self.state.write().expect("poisoned!");
        let _ = state.update(action);
        Ok(())
    }
}

// TODO: impl AsyncEngine for MultiThreadedEngine

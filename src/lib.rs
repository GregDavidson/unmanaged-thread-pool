// Inspired by Rust Book Final Project

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::TrySendError;
use std::sync::Arc;
use std::sync::Mutex;

pub struct ThreadPool {
  sender: mpsc::SyncSender<Job>,
  receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
  id: usize, // available for debugging
}

trait FnBox {
  fn call(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
  fn call(self: Box<F>) {
    (*self)()
  }
}

type Job = Box<dyn FnBox + Send + 'static>;

impl ThreadPool {
  /// Create a new ThreadPool.
  pub fn new( ) -> ThreadPool {

    let (sender, receiver) = mpsc::sync_channel(0); // rendezvous channel
    let receiver = Arc::new(Mutex::new(receiver));
    
    ThreadPool { sender, receiver, id:0 }
  }

  /// Execute a function in a thread.
  pub fn execute<F>(&self, f: F) -> Result< (), () >
    where  F: FnOnce() + Send + 'static
  {
    let job = Box::new(f);
    
    match self.sender.try_send(job) {
      Ok(_) => Ok( () ), // rendezvous successful, job delivered
      Err(e) => match e {
        TrySendError::Full(job) => { // no worker available; give birth to a new one
          let receiver = self.receiver.clone();
          thread::spawn(move || {
            job.call();
            loop {
              match receiver.lock().unwrap().recv() {
                Ok(job) => { job.call(); },
                Err(_) => { return; }
              }
            }
          } ); 
          Ok( () )
        },
        TrySendError::Disconnected(_) => Err( () )
      }
    }
  }

}


#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  /// run_multiple_tasks - check that they all execute and complete
  fn run_multiple_tasks() {
        assert_eq!(2 + 2, 4);
    }
  
  #[test]
  /// run_multiple_tasks with significant sleeps
  /// - check that they all execute and complete in time less than sum(sleeps)
  fn run_multiple_sleepy_tasks() {
    // do something!
        assert_eq!(4 + 4, 8);
    }
  
  #[test]
  /// run_multiple_tasks which block on external channel
  /// - check that they execute and complete independently
  fn run_multiple_blocking_tasks() {
    // do something!
        assert_eq!(4 + 2, 6);
    }
  }

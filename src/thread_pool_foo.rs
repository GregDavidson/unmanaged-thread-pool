// Inspired by Rust Book Final Project

use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

enum Message {
  NewJob(Job),
  Terminate,
}

pub struct ThreadPool {
  workers: Vec<Worker>,
  sender: mpsc::Sender<Message>,
}

trait FnBox {
  fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
  fn call_box(self: Box<F>) {
    (*self)()
  }
}

type Job = Box<dyn FnBox + Send + 'static>;

impl ThreadPool {
  /// Create a new ThreadPool.
  ///
  /// The size is the number of threads in the pool.
  ///
  /// # Panics
  ///
  /// The `new` function will panic if the size is zero.
  // should it be returning Self??
  pub fn new(size: usize) -> ThreadPool {
    assert!(size > 0);
    let (sender, receiver) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(receiver));
    let mut workers = Vec::with_capacity(size);
    
    for id in 0..size {
      workers.push(Worker::new(id, Arc::clone(&receiver)));
    }
    
    ThreadPool { workers, sender }
  }
  
  pub fn execute<F>(&self, f: F)
    where  F: FnOnce() + Send + 'static
  {
    let job = Box::new(f);
    
    self.sender.send(Message::NewJob(job)).unwrap();
  }
}

impl Drop for ThreadPool {
  fn drop(&mut self) {
    eprintln!("Sending terminate message to all workers.");
    
    for _ in &mut self.workers {
      self.sender.send(Message::Terminate).unwrap();
    }
    
    eprintln!("Shutting down all workers.");
    
    for worker in &mut self.workers {
      eprintln!("Shutting down worker {}", worker.id);
      
      if let Some(thread) = worker.thread.take() {
        thread.join().unwrap();
      }
    }
  }
}

struct Worker {
  id: usize,
  thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
  fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
    
    let thread = thread::spawn(move ||{
      loop {
        let message = receiver.lock().unwrap().recv().unwrap();
        
        match message {
          Message::NewJob(job) => {
            eprintln!("Worker {} got a job; executing.", id);
            job.call_box();
          },
          Message::Terminate => {
            eprintln!("Worker {} was told to terminate.", id);
            break;
          },
        }
      }
    });
    
    Worker { id, thread: Some(thread) }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn do_something() {
    // do something!
  }
}

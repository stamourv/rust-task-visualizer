#![crate_type = "rlib"]

extern crate time;
extern crate sync;

use sync::Arc;
use std::any::Any;
use std::mem;
use std::raw;
use std::rt::Runtime;
use std::rt::exclusive::Exclusive;
use std::rt::local::Local;
use std::rt::rtio;
use std::rt::task::{Task, TaskOpts, BlockedTask};

#[deriving(Show, Clone)]
pub struct Message {
    /// Time at which this event happened
    pub timestamp: u64,
    /// Rust task that performed this event
    pub task_id: uint,
    /// OS thread that performed this event
    pub thread_id: uint,
    /// Creator task of this thread (zero if not relevant)
    pub creator: uint,
    /// Short string description of the event that happened
    pub desc: String,
}

struct InstrumentedRuntime<R> {
    inner: Option<Box<R>>,
    messages: Arc<Exclusive<Vec<Message>>>,
}

/// Instrument all code run inside the specific block, returning a vector of all
/// messages which occurred.
pub fn instrument<R: 'static + Runtime + Send>(f: ||) -> Vec<Message> {
    install::<R>(Arc::new(Exclusive::new(Vec::new())), 0);
    f();
    let rt = uninstall::<R>();
    unsafe { rt.messages.lock().clone() }
}

/// Installs an instrumented runtime which will append to the given vector of
/// messages.
///
/// The instrumented runtime is installed into the current task.
fn install<R: 'static + Runtime + Send>(
    messages: Arc<Exclusive<Vec<Message>>>, creator: uint
) {
    let mut task = Local::borrow(None::<Task>);
    let rt = task.maybe_take_runtime::<R>().unwrap();
    let mut new_rt = box InstrumentedRuntime {
        inner: Some(rt),
        messages: messages
    };
    new_rt.log2("spawn", creator);
    task.put_runtime(new_rt);
}

/// Uninstalls the runtime from the current task, returning the instrumented
/// runtime.
fn uninstall<R: 'static + Runtime + Send>() -> InstrumentedRuntime<R> {
    let mut task = Local::borrow(None::<Task>);
    let mut rt = task.maybe_take_runtime::<InstrumentedRuntime<R>>().unwrap();
    rt.log("death");
    task.put_runtime(rt.inner.take().unwrap());
    *rt
}

impl<R: 'static + Runtime + Send>  InstrumentedRuntime<R> {
    /// Puts this runtime back into the local task
    fn put(mut ~self, msg: &str) {
        assert!(self.inner.is_none());

        let mut task: Box<Task> = Local::take();
        let rt = task.maybe_take_runtime::<R>().unwrap();
        self.inner = Some(rt);
        self.log(msg);
        task.put_runtime(self);
        Local::put(task);
    }

    /// Logs a message into this runtime
    fn log(&mut self, msg: &str) {
        self.log2(msg, 0)
    }
    /// Logs a longer message
    fn log2(&mut self, msg: &str, creator: uint) {
        let id = self.thread_id();
        let mut messages = unsafe { self.messages.lock() };
        messages.push(Message {
            timestamp: time::precise_time_ns(),
            desc: msg.to_str(),
            task_id: self.task_id(),
            thread_id: id,
            creator: creator,
        });
    }

    fn task_id(&self) -> uint { self as *_ as uint }

    fn thread_id(&mut self) -> uint {
        self.inner.get_mut_ref().local_io().map(|mut i| {
            let i: raw::TraitObject = unsafe { mem::transmute(i.get()) };
            i.data as uint
        }).unwrap_or(0)
    }
}

impl<R: 'static + Runtime + Send> Runtime for InstrumentedRuntime<R> {
    fn yield_now(mut ~self, cur_task: Box<Task>) {
        self.log("yield");
        self.inner.take().unwrap().yield_now(cur_task);
        self.put("done-yield")
    }

    fn maybe_yield(mut ~self, cur_task: Box<Task>) {
        self.log("maybe-yield");
        self.inner.take().unwrap().maybe_yield(cur_task);
        self.put("done-yield")
    }

    fn deschedule(mut ~self, times: uint, cur_task: Box<Task>,
                  f: |BlockedTask| -> Result<(), BlockedTask>) {
        self.log("deschedule");
        self.inner.take().unwrap().deschedule(times, cur_task, f);
        self.put("wakeup")
    }

    fn reawaken(mut ~self, _to_wake: Box<Task>) { fail!("unimplemented") }

    fn spawn_sibling(mut ~self,
                     cur_task: Box<Task>,
                     opts: TaskOpts,
                     f: proc():Send) {
        // Be sure to install an instrumented runtime for the spawned sibling by
        // specifying a new runtime.
        let messages = self.messages.clone();
        let me = self.task_id();
        self.log("before-spawn");
        self.inner.take().unwrap().spawn_sibling(cur_task, opts, proc() {
            install::<R>(messages, me);
            f();
            drop(uninstall::<R>());
        });
        self.put("after-spawn")
    }

    fn local_io<'a>(&'a mut self) -> Option<rtio::LocalIo<'a>> {
        self.inner.get_mut_ref().local_io()
    }
    fn stack_bounds(&self) -> (uint, uint) { self.inner.get_ref().stack_bounds() }
    fn can_block(&self) -> bool { self.inner.get_ref().can_block() }
    fn wrap(~self) -> Box<Any> { self as Box<Any> }
}

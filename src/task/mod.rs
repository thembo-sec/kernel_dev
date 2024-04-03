use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
pub mod simple_executor;

/// Task structure.
///
/// This is a wrapper around a pinned, heap-allocated, and
/// dynamically dispatched future with the empty type ()
pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    /// This function takes an arbitrary future with an output type of () and pins it in memory
    /// through the Box::pin function.
    /// Then it wraps the boxed future in the Task struct and returns it.
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future),
        }
    }
    /// Since the poll method of the Future trait expects to be called on a Pin<&mut T> type, 
    /// we use the Pin::as_mut method to convert the self.future field of type Pin<Box<T>> first. 
    /// Then we call poll on the converted self.future field and return the result. 
    /// Since the Task::poll method should only be called by the executor that we’ll create in a moment, 
    /// we keep the function private to the task module.
    fn poll(&mut self, context: &mut Context) -> Poll<()>{
        self.future.as_mut().poll(context)
    }
}

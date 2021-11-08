#![feature(generators, generator_trait)]

use std::{marker, mem, ops, pin::Pin};

pub mod task;
pub use task::*;

pub fn execute_recursive<G, R>(mut task: Task<G, R>) -> R
where
    G: ops::Generator<mem::MaybeUninit<R>, Yield = Task<G, R>, Return = R>,
{
    let mut task = unsafe { Pin::new_unchecked(&mut task) };
    let mut output = unsafe { task.as_mut().resume(mem::MaybeUninit::uninit()) };
    loop {
        match output {
            ops::GeneratorState::Yielded(g) => {
                let out = execute_recursive(g);
                output = unsafe { task.as_mut().resume(mem::MaybeUninit::new(out)) };
            }
            ops::GeneratorState::Complete(r) => break r,
        }
    }
}

pub fn execute_vec_stack<G, R>(task: Task<G, R>) -> R
where
    G: ops::Generator<mem::MaybeUninit<R>, Yield = Task<G, R>, Return = R> + marker::Unpin,
{
    let mut task_stack = vec![task];
    // task on top of the stack is fresh
    let mut output = mem::MaybeUninit::uninit();

    while let Some(task) = task_stack.last_mut() {
        let out = unsafe { Pin::new(task).resume(output) };
        match out {
            ops::GeneratorState::Yielded(t) => {
                output = mem::MaybeUninit::uninit();
                task_stack.push(t);
            }
            ops::GeneratorState::Complete(out) => {
                output = mem::MaybeUninit::new(out);
                task_stack.pop();
            }
        };
    }

    unsafe { output.assume_init() }
}

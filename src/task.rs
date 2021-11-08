use std::{marker, mem, ops, pin::Pin};

#[repr(transparent)]
pub struct Task<G, R>(G, marker::PhantomData<R>);

impl<G: Unpin, R> marker::Unpin for Task<G, R> {}

impl<G, R> Task<G, R> {
    pub fn inner(self) -> G {
        self.0
    }
}

impl<G, R> Task<G, R>
where
    G: ops::Generator<mem::MaybeUninit<R>, Yield = Self, Return = R>,
{
    /// SAFETY: `gen` may only assume the argument, passed through `Generator::resume`:
    /// - could be uninitalized on the first call
    /// - is initialized on any subsequent call
    pub unsafe fn new_unchecked(gen: G) -> Self {
        Task(gen, marker::PhantomData)
    }

    /// SAFETY: `arg`:
    /// - may be uninitalized on the first call
    /// - must be initialized on any subsequent call
    pub unsafe fn resume(
        self: Pin<&mut Self>,
        arg: mem::MaybeUninit<R>,
    ) -> ops::GeneratorState<Self, R> {
        self.map_unchecked_mut(|p| &mut p.0).resume(arg)
    }
}

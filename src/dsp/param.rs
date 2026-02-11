use fundsp::shared::Shared;

/// Lock-free ECS→audio bridge wrapping `fundsp::Shared`.
///
/// Create one per parameter when building a DSP graph. The ECS sync systems
/// call [`ParamHandle::set`] on the main thread; the audio thread reads the
/// value through the [`fundsp::prelude::var`] node wired to the same [`Shared`].
#[derive(Clone)]
pub struct ParamHandle {
    inner: Shared,
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
}

impl ParamHandle {
    pub fn new(name: &'static str, initial: f32, min: f32, max: f32) -> Self {
        Self {
            inner: Shared::new(initial),
            name,
            min,
            max,
        }
    }

    /// Write a new value from the main thread (atomic store).
    pub fn set(&self, value: f32) {
        let clamped = value.clamp(self.min, self.max);
        self.inner.set_value(clamped);
    }

    /// Get the current value.
    pub fn get(&self) -> f32 {
        self.inner.value()
    }

    /// Access the inner `Shared` for wiring into a FunDSP graph via `var(&shared)`.
    pub fn shared(&self) -> &Shared {
        &self.inner
    }
}

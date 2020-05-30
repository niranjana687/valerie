use crate::{channel, Receiver, Sender};
use super::State;

use crossbeam::atomic::AtomicCell;
use futures_intrusive::channel::StateId;
use core::fmt::Display;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};
use alloc::sync::Arc;

#[derive(Clone)]
pub struct StateAtomic<T>
where
    T: Display + Copy,
{
    value: Arc<AtomicCell<T>>,
    tx: Arc<Sender>,
    rx: Arc<Receiver>,
}

impl<T> State<T> for StateAtomic<T>
where
    T: Display + Copy,
{
    type Value = AtomicCell<T>;

    fn value(&self) -> T {
        self.value.load()
    }

    fn tx(&self) -> Arc<Sender> {
        Arc::clone(&self.tx)
    }

    fn rx(&self) -> Arc<Receiver> {
        Arc::clone(&self.rx)
    }

    fn put(&self, value: T) {
        self.value.store(value);
    }

    fn pointer(&self) -> Arc<Self::Value> {
        Arc::clone(&self.value)
    }

    fn update(&self) {
        while self.tx.send(self.value().into()).is_err() {}
    }
}

impl<T> StateAtomic<T>
where
    T: Display + Copy,
{
    pub fn new(value: T) -> Self {
        let (tx, rx) = channel();
        Self {
            value: Arc::new(AtomicCell::new(value)),
            tx: Arc::new(tx),
            rx: Arc::new(rx),
        }
    }

    pub fn from<U, V, F>(state: &U, mut func: F) -> Self
    where
        U: State<V> + 'static,
        V: Display + Clone + 'static,
        F: FnMut(V) -> T + 'static,
        T: From<V> + 'static,
    {
        let value = func(state.value());
        let new = Self::new(value);

        let new_move = new.clone();
        let state_value = state.clone();
        let rx = state.rx();
        wasm_bindgen_futures::spawn_local(async move {
            let mut old = StateId::new();
            while let Some((new, _)) = rx.receive(old).await {
                new_move.put(func(state_value.value()));
                new_move.update();

                old = new;
            }
        });

        new
    }
}

impl<T, U> Add<U> for StateAtomic<T>
where
    T: Display + Copy + Add<U> + AddAssign<U>,
{
    type Output = Self;

    fn add(mut self, other: U) -> Self::Output {
        self += other;
        self
    }
}

impl<T, U> AddAssign<U> for StateAtomic<T>
where
    T: Display + Copy + AddAssign<U>,
{
    fn add_assign(&mut self, other: U) {
        let mut value = self.value();
        value += other;
        self.put(value);
    }
}

impl<T, U> Div<U> for StateAtomic<T>
where
    T: Display + Copy + Div<U> + DivAssign<U>,
{
    type Output = Self;

    fn div(mut self, other: U) -> Self::Output {
        self /= other;
        self
    }
}

impl<T, U> DivAssign<U> for StateAtomic<T>
where
    T: Display + Copy + DivAssign<U>,
{
    fn div_assign(&mut self, other: U) {
        let mut value = self.value();
        value /= other;
        self.put(value);
    }
}

impl<T, U> Mul<U> for StateAtomic<T>
where
    T: Display + Copy + Mul<U> + MulAssign<U>,
{
    type Output = Self;

    fn mul(mut self, other: U) -> Self::Output {
        self *= other;
        self
    }
}

impl<T, U> MulAssign<U> for StateAtomic<T>
where
    T: Display + Copy + MulAssign<U>,
{
    fn mul_assign(&mut self, other: U) {
        let mut value = self.value();
        value *= other;
        self.put(value);
    }
}

impl<T, U> Rem<U> for StateAtomic<T>
where
    T: Display + Copy + Rem<U> + RemAssign<U>,
{
    type Output = Self;

    fn rem(mut self, other: U) -> Self::Output {
        self %= other;
        self
    }
}

impl<T, U> RemAssign<U> for StateAtomic<T>
where
    T: Display + Copy + RemAssign<U>,
{
    fn rem_assign(&mut self, other: U) {
        let mut value = self.value();
        value %= other;
        self.put(value);
    }
}

impl<T, U> Sub<U> for StateAtomic<T>
where
    T: Display + Copy + Sub<U> + SubAssign<U>,
{
    type Output = Self;

    fn sub(mut self, other: U) -> Self::Output {
        self -= other;
        self
    }
}

impl<T, U> SubAssign<U> for StateAtomic<T>
where
    T: Display + Copy + SubAssign<U>,
{
    fn sub_assign(&mut self, other: U) {
        let mut value = self.value();
        value -= other;
        self.put(value);
    }
}

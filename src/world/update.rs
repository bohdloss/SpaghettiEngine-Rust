use crate::utils::types::*;

pub trait Update {
    fn update(&mut self, delta: float);
}

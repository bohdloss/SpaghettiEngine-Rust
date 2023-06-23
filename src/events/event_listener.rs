use crate::events::GameEvent;

pub trait EventListener: Send {
    fn handle_event(&self, event: &mut Box<dyn GameEvent>);
}

pub struct LambdaEL<T: Fn(&mut Box<dyn GameEvent>) + Send + 'static> {
    f: T,
}

impl<T: Fn(&mut Box<dyn GameEvent>) + Send + 'static> LambdaEL<T> {
    pub fn new(f: T) -> Self {
        Self { f }
    }
}

impl<T: Fn(&mut Box<dyn GameEvent>) + Send + 'static> EventListener for LambdaEL<T> {
    fn handle_event(&self, event: &mut Box<dyn GameEvent>) {
        (self.f)(event);
    }
}

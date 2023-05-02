pub trait ThreadComponent {
    fn initialize(&mut self);
    fn post_initialize(&mut self);
    fn loop_cycle(&mut self, delta: f32);
    fn pre_terminate(&mut self);
    fn terminate(&mut self);
}

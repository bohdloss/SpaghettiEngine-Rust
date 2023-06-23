pub trait Replicate {
    // TODO ADD PARAMETERS
    fn write_data(&self, is_client: bool);
    fn read_data(&mut self, is_client: bool);

    fn needs_replication(&self) -> bool {
        true
    }

    fn is_local(&self) -> bool {
        false
    }
}

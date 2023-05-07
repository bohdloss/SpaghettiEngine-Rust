pub trait Replicate { // TODO ADD PARAMETERS
	fn write_data_server(&self);
	fn read_data_server(&self);
	fn write_data_client(&self);
	fn read_data_client(&self);

	fn needs_replication(&self) -> bool {
		true
	}

	fn is_local(&self) -> bool {
		false
	}
}
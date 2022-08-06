pub struct AsyncStdExecutor;
impl futures::task::Spawn for AsyncStdExecutor {
    fn spawn_obj(
        &self,
        future: futures::task::FutureObj<'static, ()>,
    ) -> Result<(), futures::task::SpawnError> {
        async_std::task::spawn(future);
        Ok(())
    }
}

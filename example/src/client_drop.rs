use std::time::Duration;
use tokio::sync::Semaphore;

#[norpc::service]
trait Loop {
    fn inf_loop();
    fn noop();
}
struct LoopApp {
    sem: Semaphore,
}
#[norpc::async_trait]
impl Loop for LoopApp {
    async fn inf_loop(&self) {
        let _tok = self.sem.acquire().await;
        loop {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }
    async fn noop(&self) {
        let _tok = self.sem.acquire().await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_client_drop() {
    use norpc::runtime::*;

    let app = LoopApp {
        sem: Semaphore::new(1),
    };
    let builder = ServerBuilder::new(LoopService::new(app));
    let (chan, server) = builder.build();
    ::tokio::spawn(server.serve(tokio::TokioExecutor));

    let mut cli1 = LoopClient::new(chan.clone());
    let hdl1 = ::tokio::spawn(async move {
        cli1.inf_loop().await;
    });

    ::tokio::time::sleep(Duration::from_secs(1)).await;

    let mut cli2 = LoopClient::new(chan);
    let hdl2 = ::tokio::spawn(async move {
        cli2.noop().await;
    });

    // Comment out this line ends up server looping forever.
    hdl1.abort();

    hdl2.await.unwrap();
}

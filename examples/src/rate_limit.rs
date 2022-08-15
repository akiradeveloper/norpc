use tower::util::BoxCloneService;
use tower::{Layer, ServiceBuilder};

const N: usize = 10000;

#[norpc::service]
trait RateLimit {
    fn noop();
}

struct RateLimitApp;
#[norpc::async_trait]
impl RateLimit for RateLimitApp {
    async fn noop(&self) {}
}
struct ServiceHolder {
    chan: BoxCloneService<RateLimitRequest, RateLimitResponse, tower::BoxError>,
}
#[tokio::test(flavor = "multi_thread")]
async fn test_rate_limit() {
    use norpc::runtime::*;
    let app = RateLimitApp;
    let service = RateLimitService::new(app);
    let service = ServiceBuilder::new()
        .rate_limit(5000, std::time::Duration::from_secs(1))
        .service(service);
    let builder = ServerBuilder::new(service);
    let (chan, server) = builder.build();
    ::tokio::spawn(server.serve(TokioExecutor));
    let inner = ServiceBuilder::new()
        .buffer(1)
        .rate_limit(1000, std::time::Duration::from_secs(1))
        .service(chan.unwrap());
    let chan = Channel::new(inner);
    // This move means nothing but to check if holding the boxed service in struct works.
    let cli = RateLimitClient::new(chan);
    for _ in 0..N {
        // This can be commented out but to make sure thet the client is cloneable.
        let mut cli = cli.clone();
        cli.noop().await;
    }
}

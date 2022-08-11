use tower::util::BoxCloneService;
use tower::ServiceBuilder;

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
    let chan = ServiceBuilder::new()
        .buffer(1)
        .rate_limit(1000, std::time::Duration::from_secs(1))
        .service(chan);
    let chan = BoxCloneService::new(chan);
    // This move means nothing but to check if holding the boxed service in struct works.
    let holder = ServiceHolder { chan };
    let cli = RateLimitClient::new(holder.chan);
    for _ in 0..N {
        // This can be commented out but to make sure thet the client is cloneable.
        let mut cli = cli.clone();
        cli.noop().await;
    }
}

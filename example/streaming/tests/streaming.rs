use futures::stream::StreamExt;
use futures::stream::{BoxStream, LocalBoxStream};
use tokio::sync::mpsc;

#[norpc::service]
trait BidiStreaming {
    fn double(input: Stream<u64>) -> Stream<u64>;
}
#[derive(Clone)]
struct BidiStreamingApp;
#[norpc::async_trait]
impl BidiStreaming for BidiStreamingApp {
    async fn double(self, st: BoxStream<'static, u64>) -> BoxStream<'static, u64> {
        Box::pin(st.map(|x| {
            if x == 3 {
                panic!();
            }
            2 * x
        }))
    }
}
#[should_panic]
#[tokio::test(flavor = "multi_thread")]
async fn test_bidi_streaming() {
    use norpc::runtime::send::*;
    let (tx, rx) = mpsc::channel(100);
    tokio::spawn(async move {
        let app = BidiStreamingApp;
        let service = BidiStreamingService::new(app);
        let server = Executor::new(rx, service);
        server.serve().await
    });
    let chan = ClientService::new(tx);
    let mut cli = BidiStreamingClient::new(chan);
    let inp_stream = Box::pin(futures::stream::iter([1, 2, 3]));
    let mut ret_stream = cli.double(inp_stream).await.unwrap();
    assert_eq!(ret_stream.next().await, Some(2));
    assert_eq!(ret_stream.next().await, Some(4));
    ret_stream.next().await;
}

#[norpc::service(?Send)]
trait BidiStreamingLocal {
    fn double(input: Stream<u64>) -> Stream<u64>;
}
#[derive(Clone)]
struct BidiStreamingLocalApp;
#[norpc::async_trait(?Send)]
impl BidiStreamingLocal for BidiStreamingLocalApp {
    async fn double(self, st: LocalBoxStream<'static, u64>) -> LocalBoxStream<'static, u64> {
        Box::pin(st.map(|x| {
            let y = 2 * x;
            y
        }))
    }
}
#[tokio::test(flavor = "multi_thread")]
async fn test_bidi_streaming_local() {
    use norpc::runtime::no_send::*;
    let local = tokio::task::LocalSet::new();
    let (tx, rx) = mpsc::channel(100);
    local.spawn_local(async move {
        let app = BidiStreamingLocalApp;
        let service = BidiStreamingLocalService::new(app);
        let server = Executor::new(rx, service);
        server.serve().await
    });
    local.spawn_local(async move {
        let chan = ClientService::new(tx);
        let mut cli = BidiStreamingLocalClient::new(chan);
        let inp_stream = Box::pin(futures::stream::iter([1, 2, 3]));
        let mut ret_stream = cli.double(inp_stream).await.unwrap();
        assert_eq!(ret_stream.next().await, Some(2));
        assert_eq!(ret_stream.next().await, Some(4));
        assert_eq!(ret_stream.next().await, Some(6));
    });
    local.await;
}

#[norpc::service]
trait ServerStreaming {
    fn double(input: u64) -> Stream<u64>;
}
#[derive(Clone)]
struct ServerStreamingApp;
#[norpc::async_trait]
impl ServerStreaming for ServerStreamingApp {
    async fn double(self, x: u64) -> BoxStream<'static, u64> {
        let st = async_stream::stream! {
            for i in x..x+3 {
                yield i*2;
            }
        };
        Box::pin(st)
    }
}
#[tokio::test(flavor = "multi_thread")]
async fn test_server_streaming() {
    use norpc::runtime::send::*;
    let (tx, rx) = mpsc::channel(100);
    tokio::spawn(async move {
        let app = ServerStreamingApp;
        let service = ServerStreamingService::new(app);
        let server = Executor::new(rx, service);
        server.serve().await
    });
    let chan = ClientService::new(tx);
    let mut cli = ServerStreamingClient::new(chan);
    let mut ret_stream = cli.double(3).await.unwrap();
    assert_eq!(ret_stream.next().await, Some(6));
    assert_eq!(ret_stream.next().await, Some(8));
    assert_eq!(ret_stream.next().await, Some(10));
}

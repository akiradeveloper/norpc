use futures::stream::StreamExt;
use std::pin::Pin;
use tokio::sync::mpsc;
use tower::Service;

#[norpc::service]
trait BidiStreaming {
    fn double(input: Stream<u64>) -> Stream<u64>;
}
#[derive(Clone)]
struct BidiStreamingApp;
#[norpc::async_trait]
impl BidiStreaming for BidiStreamingApp {
    async fn double(
        self,
        st: Pin<Box<(dyn futures::Stream<Item = u64> + std::marker::Send + 'static)>>,
    ) -> Pin<Box<(dyn futures::Stream<Item = u64> + std::marker::Send + 'static)>> {
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
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let app = BidiStreamingApp;
        let service = BidiStreamingService::new(app);
        let server = norpc::ServerChannel::new(rx, service);
        server.serve().await
    });
    let chan = norpc::ClientChannel::new(tx);
    let mut cli = BidiStreamingClient::new(chan);
    let inp_stream = Box::pin(futures::stream::iter([1, 2, 3]));
    let mut ret_stream = cli.double(inp_stream).await.unwrap();
    assert_eq!(ret_stream.next().await, Some(2));
    assert_eq!(ret_stream.next().await, Some(4));
    ret_stream.next().await;
}


#[norpc::service]
trait ServerStreaming {
    fn double(input: u64) -> Stream<u64>;
}
#[derive(Clone)]
struct ServerStreamingApp;
#[norpc::async_trait]
impl ServerStreaming for ServerStreamingApp {
    async fn double(
        self,
        x: u64,
    ) -> Pin<Box<(dyn futures::Stream<Item = u64> + std::marker::Send + 'static)>> {
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
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let app = ServerStreamingApp;
        let service = ServerStreamingService::new(app);
        let server = norpc::ServerChannel::new(rx, service);
        server.serve().await
    });
    let chan = norpc::ClientChannel::new(tx);
    let mut cli = ServerStreamingClient::new(chan);
    let mut ret_stream = cli.double(3).await.unwrap();
    assert_eq!(ret_stream.next().await, Some(6));
    assert_eq!(ret_stream.next().await, Some(8));
    assert_eq!(ret_stream.next().await, Some(10));
}
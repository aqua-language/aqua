use futures_util::SinkExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufWriter;

use crate::builtins::encoding::Encoding;
use crate::builtins::path::Path;
use crate::builtins::socket::SocketAddr;
// use crate::builtins::url::Url;
use crate::builtins::writer::Writer;
use crate::formats::Encode;
use crate::runner::context::Context;
use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn sink(self, ctx: &mut Context, writer: Writer, encoding: Encoding) {
        let mut this = self;
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        ctx.sink(|| async move {
            loop {
                let event = this.recv().await;
                match event {
                    Event::Data(_, data) => tx.send(data).await.unwrap(),
                    Event::Watermark(_) => continue,
                    Event::Snapshot(_) => todo!(),
                    Event::Sentinel => break,
                }
            }
        });
        Self::sink_encoding(ctx, rx, writer, encoding);
    }

    // async fn write_http(
    //     rx: tokio::sync::mpsc::Receiver<T>,
    //     url: Url,
    //     encoder: impl Encode + 'static,
    // ) {
    //     todo!()
    // let uri: Uri = url.0.to_string().parse().unwrap();
    // let client = hyper::Client::new();
    // let (mut tx1, rx1) = futures::channel::mpsc::channel(100);
    // let req = Request::builder()
    //     .method(Method::POST)
    //     .uri(uri)
    //     .header("content-type", encoder.content_type())
    //     .body(Body::wrap_stream(rx1))
    //     .unwrap();
    // client.request(req).await.unwrap();
    // let mut buf = vec![0; 1024];
    // loop {
    //     match rx.recv().await {
    //         Some(data) => match encoder.encode(&data, &mut buf) {
    //             Ok(n) => {
    //                 tracing::info!("Encoded: {:?}", data);
    //                 let bytes: Result<_, std::io::Error> =
    //                     Ok(hyper::body::Bytes::from(buf[0..n].to_vec()));
    //                 tx1.send(bytes).await.unwrap();
    //             }
    //             Err(e) => tracing::info!("Failed to encode: {}", e),
    //         },
    //         None => break,
    //     }
    // }
    // }

    async fn write_pipe(
        mut rx: tokio::sync::mpsc::Receiver<T>,
        mut encoder: impl Encode + Send + 'static,
        tx: impl AsyncWriteExt + Unpin,
    ) {
        let mut tx = BufWriter::new(tx);
        let mut buf = vec![0; 1024];
        loop {
            match rx.recv().await {
                Some(data) => match encoder.encode(&data, &mut buf) {
                    Ok(n) => {
                        tracing::info!("Encoded: {:?}", data);
                        tx.write_all(&buf[0..n]).await.unwrap();
                    }
                    Err(e) => tracing::info!("Failed to encode: {}", e),
                },
                None => {
                    tx.flush().await.unwrap();
                    break;
                }
            }
        }
    }

    async fn write_file(
        rx: tokio::sync::mpsc::Receiver<T>,
        path: Path,
        encoder: impl Encode + Send + 'static,
    ) {
        match tokio::fs::File::create(&path.0).await {
            Ok(tx) => Self::write_pipe(rx, encoder, tx).await,
            Err(e) => panic!("Failed to open file `{}`: {}", path.0.display(), e),
        }
    }

    async fn write_socket(
        mut rx: tokio::sync::mpsc::Receiver<T>,
        addr: SocketAddr,
        mut encoder: impl Encode + 'static,
    ) {
        tracing::info!("Connecting to {}", addr.0);
        let socket = tokio::net::TcpStream::connect(addr.0).await.unwrap();
        tracing::info!("Connected to {}", addr.0);
        let mut tx = tokio_util::codec::Framed::new(socket, tokio_util::codec::LinesCodec::new());
        let mut buf = vec![0; 1024];
        while let Some(data) = rx.recv().await {
            match encoder.encode(&data, &mut buf) {
                Ok(n) => {
                    tracing::info!("Encoded: {:?}", data);
                    let s = std::str::from_utf8(&buf[0..n - 1]).unwrap(); // -1 to remove trailing newline
                    tracing::info!("Sending: [{}]", s);
                    tx.send(s).await.unwrap();
                }
                Err(e) => tracing::info!("Failed to encode: {}", e),
            }
        }
    }

    fn sink_encoding(
        ctx: &mut Context,
        rx: tokio::sync::mpsc::Receiver<T>,
        writer: Writer,
        encoding: Encoding,
    ) {
        match encoding {
            Encoding::Csv { sep } => {
                let encoder = crate::formats::csv::ser::Writer::new(sep);
                Self::sink_writer(ctx, rx, writer, encoder);
            }
            Encoding::Json => {
                let encoder = crate::formats::json::ser::Writer::new();
                Self::sink_writer(ctx, rx, writer, encoder);
            }
        }
    }

    fn sink_writer(
        ctx: &mut Context,
        rx: tokio::sync::mpsc::Receiver<T>,
        writer: Writer,
        encoder: impl Encode + Send + 'static,
    ) {
        ctx.spawn(async move {
            match writer {
                Writer::Stdout => Self::write_pipe(rx, encoder, tokio::io::stdout()).await,
                Writer::File { path } => Self::write_file(rx, path, encoder).await,
                // Writer::Http { url } => Self::write_http(rx, url, encoder).await,
                Writer::Tcp { addr } => Self::write_socket(rx, addr, encoder).await,
                Writer::Kafka { addr: _, topic: _ } => todo!(),
            }
        });
    }
}

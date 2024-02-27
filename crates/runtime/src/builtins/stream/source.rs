use crate::formats::Decode;
use crate::runner::context::Context;

use time::OffsetDateTime;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;
use tokio_stream::StreamExt;

use crate::builtins::duration::Duration;
use crate::builtins::encoding::Encoding;
use crate::builtins::path::Path;
use crate::builtins::reader::Reader;
use crate::builtins::socket::SocketAddr;
use crate::builtins::stream::Event;
use crate::builtins::time::Time;
use crate::builtins::time_source::TimeSource;
// use crate::builtins::url::Url;
use crate::traits::Data;

use super::Stream;

impl<T: Data> Stream<T> {
    pub fn source(
        ctx: &mut Context,
        reader: Reader,
        encoding: Encoding,
        time_source: TimeSource<impl Fn(&T) -> Time + Send + 'static>,
    ) -> Stream<T> {
        Self::source_encoding(ctx, reader, encoding, time_source)
    }

    fn source_encoding(
        ctx: &mut Context,
        reader: Reader,
        encoding: Encoding,
        time_source: TimeSource<impl Fn(&T) -> Time + Send + 'static>,
    ) -> Stream<T> {
        match encoding {
            Encoding::Csv { sep } => {
                let decoder = crate::formats::csv::de::Reader::<1024>::new(sep);
                Self::sink_reader(ctx, reader, decoder, time_source)
            }
            Encoding::Json => {
                let decoder = crate::formats::json::de::Reader::new();
                Self::sink_reader(ctx, reader, decoder, time_source)
            }
        }
    }

    async fn read_pipe(
        rx: impl AsyncReadExt + Unpin,
        mut decoder: impl Decode + 'static,
        watch: bool,
        tx: tokio::sync::mpsc::Sender<T>,
    ) {
        let mut rx = BufReader::with_capacity(1024 * 30, rx);
        let mut buf = Vec::with_capacity(1024 * 30);
        loop {
            match rx.read_until(b'\n', &mut buf).await {
                Ok(0) => {
                    tracing::info!("EOF");
                    if watch {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    } else {
                        break;
                    }
                }
                Ok(n) => match decoder.decode(&buf[0..n]) {
                    Ok(data) => {
                        tracing::info!("Decoded: {:?}", data);
                        tx.send(data).await.unwrap();
                        buf.clear();
                    }
                    Err(e) => tracing::info!("Failed to decode: {}", e),
                },
                Err(e) => panic!("Failed to read from stdin: {}", e),
            }
        }
    }

    async fn read_file(
        path: Path,
        decoder: impl Decode + 'static,
        watch: bool,
        tx2: tokio::sync::mpsc::Sender<T>,
    ) {
        match tokio::fs::File::open(&path.0).await {
            Ok(rx) => Self::read_pipe(rx, decoder, watch, tx2).await,
            Err(e) => panic!("Failed to open file `{}`: {}", path.0.display(), e),
        }
    }

    async fn read_socket(
        addr: SocketAddr,
        mut decoder: impl Decode + 'static,
        tx: tokio::sync::mpsc::Sender<T>,
    ) {
        tracing::info!("Trying to listen on {}", addr.0);
        let socket = tokio::net::TcpListener::bind(addr.0).await.unwrap();
        tracing::info!("Listening on {}", addr.0);
        let (socket, _) = socket.accept().await.unwrap();
        tracing::info!("Accepted connection from {}", addr.0);
        let mut rx = tokio_util::codec::Framed::new(socket, tokio_util::codec::LinesCodec::new());
        loop {
            match rx.next().await {
                Some(Ok(line)) => match decoder.decode(line.as_bytes()) {
                    Ok(data) => {
                        tracing::info!("Decoded: {:?}", data);
                        tx.send(data).await.unwrap()
                    }
                    Err(e) => tracing::info!("Failed to decode: {}", e),
                },
                Some(Err(e)) => tracing::info!("Failed to read: {}", e),
                None => break,
            }
        }
    }

    // #[allow(unused)]
    // async fn read_http(url: Url, decoder: impl Decode + 'static, tx: tokio::sync::mpsc::Sender<T>) {
    //     todo!()
    // let uri: Uri = url.0.to_string().parse().unwrap();
    // let client = hyper::Client::new();
    // let mut resp = client.get(uri).await.unwrap();
    // loop {
    //     match resp.body_mut().data().await {
    //         Some(Ok(chunk)) => match decoder.decode(&chunk) {
    //             Ok(data) => {
    //                 tracing::info!("Decoded: {:?}", data);
    //                 tx.send(data).await.unwrap();
    //             }
    //             Err(e) => tracing::info!("Failed to decode: {}", e),
    //         },
    //         Some(Err(e)) => tracing::info!("Failed to read: {}", e),
    //         None => break,
    //     }
    // }
    // }

    fn sink_reader(
        ctx: &mut Context,
        reader: Reader,
        decoder: impl Decode + Send + 'static,
        time_source: TimeSource<impl Fn(&T) -> Time + Send + 'static>,
    ) -> Stream<T> {
        let (tx2, rx2) = tokio::sync::mpsc::channel(10);
        ctx.spawn(async move {
            match reader {
                Reader::Stdin => Self::read_pipe(tokio::io::stdin(), decoder, false, tx2).await,
                Reader::File { path, watch } => Self::read_file(path, decoder, watch, tx2).await,
                // Reader::Http { url } => Self::read_http(url, decoder, tx2).await,
                Reader::Tcp { addr } => Self::read_socket(addr, decoder, tx2).await,
                Reader::Kafka { addr: _, topic: _ } => todo!(),
            }
        });
        Self::source_event_time(ctx, rx2, time_source)
    }

    fn source_event_time(
        ctx: &mut Context,
        rx: tokio::sync::mpsc::Receiver<T>,
        time_source: TimeSource<impl Fn(&T) -> Time + Send + 'static>,
    ) -> Stream<T> {
        match time_source {
            TimeSource::Ingestion { watermark_interval } => {
                Self::source_ingestion_time(ctx, rx, watermark_interval)
            }
            TimeSource::Event {
                extractor,
                watermark_interval,
                slack,
            } => Self::_source_event_time(ctx, rx, extractor, watermark_interval, slack),
        }
    }

    fn source_ingestion_time(
        ctx: &mut Context,
        mut rx: tokio::sync::mpsc::Receiver<T>,
        watermark_interval: Duration,
    ) -> Stream<T> {
        ctx.operator(|tx1| async move {
            let mut watermark_interval = tokio::time::interval(watermark_interval.to_std());
            loop {
                tokio::select! {
                    _ = watermark_interval.tick() => {
                        tx1.send(Event::Watermark(Time::now())).await;
                    },
                    data = rx.recv() => {
                        match data {
                            Some(data) => tx1.send(Event::Data(Time::now(), data)).await,
                            None => {
                                tx1.send(Event::Sentinel).await;
                                break;
                            },
                        }
                    }
                }
            }
        })
    }

    fn _source_event_time(
        ctx: &mut Context,
        mut rx: tokio::sync::mpsc::Receiver<T>,
        extractor: impl Fn(&T) -> Time + Send + 'static,
        watermark_interval: Duration,
        slack: Duration,
    ) -> Stream<T> {
        ctx.operator(|tx| async move {
            let mut latest_time = OffsetDateTime::UNIX_EPOCH;
            let slack = slack.to_std();
            let mut watermark_interval = tokio::time::interval(watermark_interval.to_std());
            let mut watermark = OffsetDateTime::UNIX_EPOCH;
            loop {
                tokio::select! {
                    _ = watermark_interval.tick() => {
                        if latest_time > OffsetDateTime::UNIX_EPOCH {
                            watermark = latest_time - slack;
                            tx.send(Event::Watermark(Time(watermark))).await;
                        }
                    },
                    data = rx.recv() => {
                        match data {
                            Some(data) => {
                                let time = extractor(&data);
                                if time.0 < watermark {
                                    continue;
                                }
                                if time.0 > latest_time {
                                    latest_time = time.0;
                                }
                                tx.send(Event::Data(time, data)).await;
                            }
                            None => {
                                tx.send(Event::Sentinel).await;
                                break;
                            },
                        }
                    }
                }
            }
        })
    }
}

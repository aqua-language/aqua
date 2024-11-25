use crate::formats::Decode;
use crate::runner::context::Context;

use time::OffsetDateTime;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;
use tokio_stream::StreamExt;

use crate::builtins::duration::Duration;
use crate::builtins::format::Format;
use crate::builtins::path::Path;
use crate::builtins::reader::Reader;
use crate::builtins::socket::SocketAddr;
use crate::builtins::stream::Event;
use crate::builtins::time::Time;
// use crate::builtins::url::Url;
use crate::traits::Data;

use super::Stream;

impl<T: Data> Stream<T> {
    pub fn source(
        ctx: &mut Context,
        reader: Reader,
        encoding: Format,
        extractor: impl FnMut(T, Time) -> Time + Send + 'static,
        slack: Duration,
        watermark_interval: Duration,
    ) -> Stream<T> {
        match encoding {
            Format::Csv { sep } => {
                let mut decoder = crate::formats::csv::de::Reader::<1024>::new(sep);
                Self::_source1(
                    ctx,
                    reader,
                    move |s| decoder.decode(s),
                    extractor,
                    slack,
                    watermark_interval,
                )
            }
            Format::Json => {
                let mut decoder = crate::formats::json::de::Reader::new();
                Self::_source1(
                    ctx,
                    reader,
                    move |s| decoder.decode(s),
                    extractor,
                    slack,
                    watermark_interval,
                )
            }
        }
    }

    pub fn dyn_source<Seed>(
        ctx: &mut Context,
        reader: Reader,
        encoding: Format,
        extractor: impl FnMut(T, Time) -> Time + Send + 'static,
        slack: Duration,
        watermark_interval: Duration,
        type_tag: Seed,
    ) -> Stream<T>
    where
        Seed: Clone + Send + Sync + for<'a> serde::de::DeserializeSeed<'a, Value = T> + 'static,
    {
        match encoding {
            Format::Csv { sep } => {
                let mut decoder = crate::formats::csv::de::Reader::<1024>::new(sep);
                Self::_source1(
                    ctx,
                    reader,
                    move |s| decoder.decode_dyn(s, type_tag.clone()),
                    extractor,
                    slack,
                    watermark_interval,
                )
            }
            Format::Json => {
                let mut decoder = crate::formats::json::de::Reader::new();
                Self::_source1(
                    ctx,
                    reader,
                    move |s| decoder.decode_dyn(s, type_tag.clone()),
                    extractor,
                    slack,
                    watermark_interval,
                )
            }
        }
    }

    async fn read_pipe<E: std::error::Error>(
        rx: impl AsyncReadExt + Unpin,
        mut decoder: impl for<'a> FnMut(&'a [u8]) -> Result<T, E> + Send + 'static,
        watch: bool,
        tx: tokio::sync::mpsc::Sender<T>,
    ) {
        let mut rx = BufReader::with_capacity(1024 * 30, rx);
        let mut buf = Vec::with_capacity(1024 * 30);
        loop {
            match rx.read_until(b'\n', &mut buf).await {
                Ok(0) => {
                    tracing::info!("EOF");
                    println!("EOF");
                    if watch {
                        println!("Waiting for more data...");
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    } else {
                        break;
                    }
                }
                Ok(n) => match decoder(&buf[0..n]) {
                    Ok(data) => {
                        tracing::info!("Decoded: {:?}", data);
                        if tx.send(data).await.is_err() {
                            break;
                        }
                        buf.clear();
                    }
                    Err(e) => tracing::info!("Failed to decode: {}", e),
                },
                Err(e) => panic!("Failed to read from stdin: {}", e),
            }
        }
    }

    async fn read_file<E: std::error::Error>(
        path: Path,
        decoder: impl for<'a> FnMut(&'a [u8]) -> Result<T, E> + Send + 'static,
        watch: bool,
        tx2: tokio::sync::mpsc::Sender<T>,
    ) {
        match tokio::fs::File::open(&path.0).await {
            Ok(rx) => Self::read_pipe(rx, decoder, watch, tx2).await,
            Err(e) => panic!("Failed to open file `{}`: {}", path.0.display(), e),
        }
    }

    async fn read_socket<E: std::error::Error>(
        addr: SocketAddr,
        mut decoder: impl for<'a> FnMut(&'a [u8]) -> Result<T, E> + Send + 'static,
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
                Some(Ok(line)) => match decoder(line.as_bytes()) {
                    Ok(data) => {
                        tracing::info!("Decoded: {:?}", data);
                        if tx.send(data).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => tracing::info!("Failed to decode: {}", e),
                },
                Some(Err(e)) => tracing::info!("Failed to read: {}", e),
                None => break,
            }
        }
    }

    async fn read_http<E: std::error::Error>(
        _addr: SocketAddr,
        _decoder: impl for<'a> FnMut(&'a [u8]) -> Result<T, E> + Send + 'static,
        _tx: tokio::sync::mpsc::Sender<T>,
    ) {
        todo!()
    }

    fn _source1<E: std::error::Error + Send>(
        ctx: &mut Context,
        reader: Reader,
        decoder: impl for<'a> FnMut(&'a [u8]) -> Result<T, E> + Send + 'static,
        extractor: impl FnMut(T, Time) -> Time + Send + 'static,
        slack: Duration,
        watermark_interval: Duration,
    ) -> Stream<T> {
        let (tx2, rx2) = tokio::sync::mpsc::channel(10);
        ctx.spawn(async move {
            match reader {
                Reader::Stdin => Self::read_pipe(tokio::io::stdin(), decoder, false, tx2).await,
                Reader::File { path, watch } => Self::read_file(path, decoder, watch, tx2).await,
                Reader::Http { addr } => Self::read_http(addr, decoder, tx2).await,
                Reader::Tcp { addr } => Self::read_socket(addr, decoder, tx2).await,
                Reader::Kafka { addr: _, topic: _ } => todo!(),
            }
            Ok(())
        });
        Self::_source4(ctx, rx2, extractor, watermark_interval, slack)
    }

    fn _source3(
        ctx: &mut Context,
        mut rx: tokio::sync::mpsc::Receiver<T>,
        watermark_interval: Duration,
    ) -> Stream<T> {
        ctx.operator(move |tx1| async move {
            let mut watermark_interval = tokio::time::interval(watermark_interval.to_std());
            loop {
                tokio::select! {
                    _ = watermark_interval.tick() => {
                        tx1.send(Event::Watermark(Time::now())).await?;
                    },
                    data = rx.recv() => {
                        match data {
                            Some(data) => tx1.send(Event::Data(Time::now(), data)).await?,
                            None => {
                                tx1.send(Event::Sentinel).await?;
                                break;
                            },
                        }
                    }
                }
            }
            Ok(())
        })
    }

    fn _source4(
        ctx: &mut Context,
        mut rx: tokio::sync::mpsc::Receiver<T>,
        mut extractor: impl FnMut(T, Time) -> Time + Send + 'static,
        watermark_interval: Duration,
        slack: Duration,
    ) -> Stream<T> {
        ctx.operator(move |tx| async move {
            let mut latest_time = OffsetDateTime::UNIX_EPOCH;
            let slack = slack.to_std();
            let mut watermark_interval = tokio::time::interval(watermark_interval.to_std());
            let mut watermark = OffsetDateTime::UNIX_EPOCH;
            loop {
                tokio::select! {
                    _ = watermark_interval.tick() => {
                        if latest_time > OffsetDateTime::UNIX_EPOCH {
                            watermark = latest_time - slack;
                            tx.send(Event::Watermark(Time(watermark))).await?;
                        }
                    },
                    data = rx.recv() => {
                        match data {
                            Some(data) => {
                                let time = extractor(data.clone(), Time::now());
                                if time.0 < watermark {
                                    continue;
                                }
                                if time.0 > latest_time {
                                    latest_time = time.0;
                                }
                                tx.send(Event::Data(time, data)).await?;
                            }
                            None => {
                                tx.send(Event::Sentinel).await?;
                                break;
                            },
                        }
                    }
                }
            }
            Ok(())
        })
    }
}

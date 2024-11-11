pub mod package;

use std::fmt::Display;

use runtime::builtins::assigner::Assigner;
use runtime::builtins::duration::Duration;
use runtime::builtins::encoding::Encoding;
use runtime::builtins::path::Path;
use runtime::builtins::reader::Reader;
use runtime::builtins::writer::Writer;

use crate::analysis::declare;
use crate::builtins::types::stream::Operator;
use crate::builtins::value::Dataflow;
use crate::builtins::value::Fun;
use crate::builtins::value::Stream;
use crate::print::Print;

struct Wrapper<T>(T, usize);

impl Dataflow {
    pub fn flink<'a>(&'a self, decls: &'a declare::Context) -> impl Display + 'a {
        Wrapper((self, decls), 0)
    }
}

impl<'a> std::fmt::Display for Wrapper<(&'a Dataflow, &'a declare::Context)> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (dataflow, decls) = self.0;
        indoc::writedoc! {f,
            "import org.apache.flink.streaming.api.environment.StreamExecutionEnvironment;
             import org.apache.flink.streaming.api.datastream.DataStream;
             import org.apache.flink.api.common.typeinfo.TypeHint;
             import org.apache.flink.api.common.typeinfo.TypeInformation;

             {decls}

             public class Main {{
                 public static void main(String[] args) throws Exception {{
                     StreamExecutionEnvironment env = StreamExecutionEnvironment.getExecutionEnvironment();
                     env.setParallelism(1);
                     env.getConfig().enableObjectReuse();
                     env.getConfig().setAutoWatermarkInterval(1000);
                     env.setStreamTimeCharacteristic(TimeCharacteristic.EventTime);
                     env.getConfig().setLatencyTrackingInterval(1000);
                     {dataflow}
                 }}
            }}",
            decls = decls.rust(),
            dataflow = Wrapper(dataflow, 2),
        }
    }
}

impl<'a> std::fmt::Display for Wrapper<&'a Dataflow> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut p = Printer::new(f);
        p.indent_level = self.1;
        p.dataflow_stmt(self.0)
    }
}

struct Printer<'a, 'b> {
    f: &'a mut std::fmt::Formatter<'b>,
    indent_level: usize,
}

impl<'a, 'b> Print<'b> for Printer<'a, 'b> {
    fn fmt(&mut self) -> &mut std::fmt::Formatter<'b> {
        self.f
    }

    fn indent_mut(&mut self) -> &mut usize {
        &mut self.indent_level
    }
}

impl<'a, 'b> Printer<'a, 'b> {
    fn new(f: &'a mut std::fmt::Formatter<'b>) -> Self {
        Self { f, indent_level: 0 }
    }

    fn dataflow_stmt(&mut self, d: &Dataflow) -> std::fmt::Result {
        match d {
            Dataflow::Collocate(_, _) => todo!(),
            Dataflow::Sink(s, w, e) => {
                self.stream_stmt(s)?;
                self.kw("let")?;
                self.space()?;
                self.lit("_")?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.lit("Stream")?;
                self.lit(".")?;
                self.lit("sink")?;
                self.paren(|this| {
                    this.stream_id(s)?;
                    this.punct(",")?;
                    this.space()?;
                    this.lit("env")?;
                    this.punct(",")?;
                    this.space()?;
                    this.writer(w)?;
                    this.punct(",")?;
                    this.space()?;
                    this.encoding(e)
                })?;
                self.punct(";")?;
                self.newline()?;
            }
        }
        Ok(())
    }

    fn writer(&mut self, w: &Writer) -> std::fmt::Result {
        self.lit("Writer")?;
        self.punct(".")?;
        match w {
            Writer::Stdout => {
                self.lit("stdout")?;
                self.paren(|_| Ok(()))
            }
            Writer::File { path } => {
                self.lit("file")?;
                self.paren(|this| this.path(path))
            }
            Writer::Tcp { addr: _ } => todo!(),
            Writer::Kafka { addr: _, topic: _ } => todo!(),
        }
    }

    fn reader(&mut self, r: &Reader) -> std::fmt::Result {
        self.lit("Reader")?;
        self.punct(".")?;
        match r {
            Reader::Stdin => todo!(),
            Reader::File { path, watch } => {
                self.lit("file")?;
                self.paren(|this| {
                    this.path(path)?;
                    this.punct(",")?;
                    this.space()?;
                    this.lit(watch)
                })
            }
            Reader::Tcp { addr: _ } => todo!(),
            Reader::Kafka { addr: _, topic: _ } => todo!(),
            Reader::Http { addr: _ } => todo!(),
        }
    }

    fn path(&mut self, p: &Path) -> std::fmt::Result {
        self.lit("Path")?;
        self.punct(".")?;
        self.lit("new")?;
        self.paren(|this| {
            this.punct("\"")?;
            this.lit(p.0.as_os_str().to_str().unwrap())?;
            this.punct("\"")
        })
    }

    fn encoding(&mut self, e: &Encoding) -> std::fmt::Result {
        match e {
            Encoding::Csv { sep } => {
                self.lit("Encoding")?;
                self.punct(".")?;
                self.lit("csv")?;
                self.paren(|this| {
                    this.lit("'")?;
                    this.lit(sep)?;
                    this.lit("'")
                })
            }
            Encoding::Json => todo!(),
        }
    }

    fn duration(&mut self, d: &Duration) -> std::fmt::Result {
        self.lit("Duration")?;
        self.punct(".")?;
        self.lit("from_milliseconds")?;
        self.paren(|this| this.lit(d.milliseconds()))
    }

    fn stream_stmt(&mut self, s0: &Stream) -> std::fmt::Result {
        match s0.kind() {
            Operator::Source(r, e, f, slack, winterval) => {
                self.kw("let")?;
                self.space()?;
                self.stream_id(s0)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.lit("Stream")?;
                self.punct(".")?;
                self.lit("source")?;
                self.paren(|this| {
                    this.lit("env")?;
                    this.punct(",")?;
                    this.space()?;
                    this.reader(r)?;
                    this.punct(",")?;
                    this.space()?;
                    this.encoding(e)?;
                    this.punct(",")?;
                    this.space()?;
                    this.fun(f)?;
                    this.punct(",")?;
                    this.space()?;
                    this.duration(slack)?;
                    this.punct(",")?;
                    this.space()?;
                    this.duration(winterval)
                })?;
            }
            Operator::Map(s1, f) => {
                self.stream_stmt(s1)?;
                self.kw("let")?;
                self.space()?;
                self.stream_id(s0)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.lit("Stream")?;
                self.punct(".")?;
                self.lit("map")?;
                self.paren(|this| {
                    this.stream_id(s1)?;
                    this.punct(",")?;
                    this.space()?;
                    this.lit("env")?;
                    this.punct(",")?;
                    this.space()?;
                    this.fun(f)
                })?;
            }
            Operator::Filter(s1, f) => {
                self.stream_stmt(s1)?;
                self.kw("let")?;
                self.space()?;
                self.stream_id(s0)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.lit("Stream")?;
                self.punct(".")?;
                self.lit("filter")?;
                self.paren(|this| {
                    this.stream_id(s1)?;
                    this.punct(",")?;
                    this.space()?;
                    this.lit("env")?;
                    this.punct(",")?;
                    this.space()?;
                    this.fun(f)
                })?;
            }
            Operator::Flatten(_) => todo!(),
            Operator::FlatMap(_, _) => todo!(),
            Operator::Keyby(_, _) => todo!(),
            Operator::Window(s1, a, f) => {
                self.stream_stmt(s1)?;
                self.kw("let")?;
                self.space()?;
                self.stream_id(s0)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.lit("Stream")?;
                self.punct(".")?;
                self.lit("window")?;
                self.paren(|this| {
                    this.stream_id(s1)?;
                    this.punct(",")?;
                    this.space()?;
                    this.lit("env")?;
                    this.punct(",")?;
                    this.space()?;
                    this.assigner(a)?;
                    this.punct(",")?;
                    this.space()?;
                    this.fun(f)
                })?;
            }
            Operator::Merge(s1, s2) => {
                self.stream_stmt(s1)?;
                self.stream_stmt(s2)?;
                self.kw("let")?;
                self.space()?;
                self.stream_id(s0)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.lit("Stream")?;
                self.punct(".")?;
                self.lit("merge")?;
                self.paren(|this| {
                    this.stream_id(s1)?;
                    this.punct(",")?;
                    this.space()?;
                    this.stream_id(s2)
                })?;
            }
            Operator::IncrWindow(_, _, _, _, _) => todo!(),
            Operator::Take(s1, i) => {
                self.stream_stmt(s1)?;
                self.kw("let")?;
                self.space()?;
                self.stream_id(s0)?;
                self.space()?;
                self.punct("=")?;
                self.space()?;
                self.lit("Stream")?;
                self.punct(".")?;
                self.lit("take")?;
                self.paren(|this| {
                    this.stream_id(s1)?;
                    this.punct(",")?;
                    this.space()?;
                    this.lit("env")?;
                    this.punct(",")?;
                    this.space()?;
                    this.lit(i)
                })?;
            }
        }
        self.punct(";")?;
        self.newline()
    }

    fn assigner(&mut self, a: &Assigner) -> std::fmt::Result {
        self.lit("Assigner")?;
        self.punct(".")?;
        match a {
            Assigner::Tumbling { length } => {
                self.lit("tumbling")?;
                self.paren(|this| this.duration(length))
            }
            Assigner::Sliding { duration, step } => {
                self.lit("sliding")?;
                self.paren(|this| {
                    this.duration(duration)?;
                    this.punct(",")?;
                    this.space()?;
                    this.duration(step)
                })
            }
            Assigner::Session { .. } => todo!(),
            Assigner::Counting { .. } => todo!(),
            Assigner::Moving { .. } => todo!(),
        }
    }

    fn stream_id(&mut self, s: &Stream) -> std::fmt::Result {
        self.lit(&format!("_{}", s.id()))
    }

    fn fun(&mut self, f: &Fun) -> std::fmt::Result {
        f.java(self.indent_level).fmt(self.f)
    }
}

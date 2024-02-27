package org.example.queries;

import org.apache.flink.api.java.tuple.Tuple2;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.functions.windowing.AllWindowFunction;
import org.apache.flink.streaming.api.functions.windowing.WindowFunction;
import org.apache.flink.streaming.api.windowing.assigners.SlidingEventTimeWindows;
import org.apache.flink.streaming.api.windowing.assigners.TumblingEventTimeWindows;
import org.apache.flink.streaming.api.windowing.time.Time;
import org.apache.flink.streaming.api.windowing.windows.TimeWindow;
import org.apache.flink.util.Collector;
import org.example.data.Bid;
import org.example.io.DataSink;

import java.util.Optional;

public class Q5 {
    @JsonPropertyOrder({"auction"})
    public static class Output {
        public long auction;

        public Output(long auction) {
            this.auction = auction;
        }

        public Output() {
        }
    }

    public static class PrunedBid {
        public long auction;
        public long bidder;

        public PrunedBid(long auction, long bidder) {
            this.auction = auction;
            this.bidder = bidder;
        }

        public PrunedBid() {
        }
    }

    static Time size = Time.minutes(5);
    static Time slide = Time.minutes(1);

    public static void run(DataStream<Bid> bids) {
        bids.keyBy(b -> b.auction)
                .window(SlidingEventTimeWindows.of(size, slide))
                .apply(new WindowFunction<Bid, Tuple2<Long, Long>, Long, TimeWindow>() {
                    @Override
                    public void apply(
                            Long auction,
                            TimeWindow timeWindow,
                            Iterable<Bid> iterable,
                            Collector<Tuple2<Long, Long>> collector
                    ) throws Exception {
                        long count = 0;
                        for (Bid b : iterable) {
                            count++;
                        }
                        collector.collect(new Tuple2<Long, Long>(auction, count));
                    }
                })
                .windowAll(SlidingEventTimeWindows.of(size, slide))
                .apply(new AllWindowFunction<Tuple2<Long, Long>, Output, TimeWindow>() {
                    @Override
                    public void apply(
                            TimeWindow timeWindow,
                            Iterable<Tuple2<Long, Long>> iterable,
                            Collector<Output> collector
                    ) throws Exception {
                        long max = Long.MIN_VALUE;
                        Optional<Long> auction = Optional.empty();
                        for (Tuple2<Long, Long> t : iterable) {
                            if (t.f1 > max) {
                                max = t.f1;
                                auction = Optional.of(t.f0);
                            }
                        }
                        assert auction.isPresent();
                        collector.collect(new Output(auction.get()));
                    }
                })
                .returns(Output.class)
                .addSink(new DataSink<Output>());
    }

    public static void runOpt(DataStream<Bid> bids) {
        DataStream<PrunedBid> prunedBids = bids.map(b -> new PrunedBid(b.auction, b.bidder));
        prunedBids.keyBy(b -> b.auction)
                .window(SlidingEventTimeWindows.of(size, slide))
                .apply(new WindowFunction<PrunedBid, Tuple2<Long, Long>, Long, TimeWindow>() {
                    @Override
                    public void apply(
                            Long auction,
                            TimeWindow timeWindow,
                            Iterable<PrunedBid> iterable,
                            Collector<Tuple2<Long, Long>> collector
                    ) throws Exception {
                        long count = 0;
                        for (PrunedBid b : iterable) {
                            count++;
                        }
                        collector.collect(new Tuple2<Long, Long>(auction, count));
                    }
                })
                .windowAll(SlidingEventTimeWindows.of(size, slide))
                .apply(new AllWindowFunction<Tuple2<Long, Long>, Output, TimeWindow>() {
                    @Override
                    public void apply(
                            TimeWindow timeWindow,
                            Iterable<Tuple2<Long, Long>> iterable,
                            Collector<Output> collector
                    ) throws Exception {
                        long max = Long.MIN_VALUE;
                        Optional<Long> auction = Optional.empty();
                        for (Tuple2<Long, Long> t : iterable) {
                            if (t.f1 > max) {
                                max = t.f1;
                                auction = Optional.of(t.f0);
                            }
                        }
                        assert auction.isPresent();
                        collector.collect(new Output(auction.get()));
                    }
                })
                .returns(Output.class)
                .addSink(new DataSink<Output>());
    }

}

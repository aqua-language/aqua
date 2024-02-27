package org.example.queries;

import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.functions.windowing.WindowFunction;
import org.apache.flink.streaming.api.windowing.assigners.TumblingEventTimeWindows;
import org.apache.flink.streaming.api.windowing.time.Time;
import org.apache.flink.streaming.api.windowing.windows.TimeWindow;
import org.apache.flink.util.Collector;
import org.example.data.Bid;
import org.example.io.DataSink;

import java.util.Optional;

public class Q7 {
    @JsonPropertyOrder({"auction", "price", "bidder"})
    public static class Output {
        public long auction;
        public long price;
        public long bidder;

        public Output(long auction, long price, long bidder) {
            this.auction = auction;
            this.price = price;
            this.bidder = bidder;
        }

        public Output() {
        }
    }

    public static class PrunedBid {
        public long auction;
        public long price;
        public long bidder;

        public PrunedBid(long auction, long price, long bidder) {
            this.auction = auction;
            this.price = price;
            this.bidder = bidder;
        }

        public PrunedBid() {
        }
    }

    static Time size = Time.seconds(10);

    public static void run(DataStream<Bid> bids) {
        bids.keyBy(b -> b.auction)
                .window(TumblingEventTimeWindows.of(size))
                .apply(new WindowFunction<Bid, Output, Long, TimeWindow>() {
                    @Override
                    public void apply(
                            Long auction,
                            TimeWindow timeWindow,
                            Iterable<Bid> iterable,
                            Collector<Output> collector
                    ) throws Exception {
                        long max = Long.MIN_VALUE;
                        Optional<Bid> maxBid = Optional.empty();
                        for (Bid b : iterable) {
                            if (b.price > max) {
                                max = b.price;
                                maxBid = Optional.of(b);
                            }
                        }
                        assert maxBid.isPresent();
                        Bid bid = maxBid.get();
                        collector.collect(new Output(auction, bid.price, bid.bidder));
                    }
                })
                .addSink(new DataSink<Output>());
    }

    public static void runOpt(DataStream<Bid> bids) {
        DataStream<PrunedBid> prunedBids = bids.map(b -> new PrunedBid(b.auction, b.price, b.bidder));
        prunedBids.keyBy(b -> b.auction)
                .window(TumblingEventTimeWindows.of(size))
                .apply(new WindowFunction<PrunedBid, Output, Long, TimeWindow>() {
                    @Override
                    public void apply(
                            Long auction,
                            TimeWindow timeWindow,
                            Iterable<PrunedBid> iterable,
                            Collector<Output> collector
                    ) throws Exception {
                        long max = Long.MIN_VALUE;
                        Optional<PrunedBid> maxBid = Optional.empty();
                        for (PrunedBid b : iterable) {
                            if (b.price > max) {
                                max = b.price;
                                maxBid = Optional.of(b);
                            }
                        }
                        assert maxBid.isPresent();
                        PrunedBid bid = maxBid.get();
                        collector.collect(new Output(auction, bid.price, bid.bidder));
                    }
                })
                .addSink(new DataSink<Output>());
    }
}

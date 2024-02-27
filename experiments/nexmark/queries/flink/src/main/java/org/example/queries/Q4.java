package org.example.queries;

import org.apache.flink.api.common.functions.JoinFunction;
import org.apache.flink.api.java.functions.KeySelector;
import org.apache.flink.api.java.tuple.Tuple2;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.functions.windowing.WindowFunction;
import org.apache.flink.streaming.api.windowing.assigners.TumblingEventTimeWindows;
import org.apache.flink.streaming.api.windowing.time.Time;
import org.apache.flink.streaming.api.windowing.windows.TimeWindow;
import org.apache.flink.util.Collector;
import org.example.data.Auction;
import org.example.data.Bid;
import org.example.io.DataSink;

public class Q4 {
    @JsonPropertyOrder({"auction", "price"})
    public static class Output {
        public long auction;
        public long price;

        public Output(long auction, long price) {
            this.auction = auction;
            this.price = price;
        }

        public Output() {
        }
    }

    public static class PrunedAuction {
        public long id;
        public long category;
        public long expires;
        public long dateTime;

        public PrunedAuction(long id, long category, long expires, long dateTime) {
            this.id = id;
            this.category = category;
            this.expires = expires;
            this.dateTime = dateTime;
        }

        public PrunedAuction() {
        }
    }

    public static class PrunedBid {
        public long auction;
        public long price;
        public long dateTime;

        public PrunedBid(long auction, long price, long dateTime) {
            this.auction = auction;
            this.price = price;
            this.dateTime = dateTime;
        }

        public PrunedBid() {
        }
    }

    static Time size = Time.seconds(10);

    public static void run(DataStream<Auction> auctions, DataStream<Bid> bids) {
        auctions.join(bids)
                .where(a -> a.id)
                .equalTo(b -> b.auction)
                .window(TumblingEventTimeWindows.of(size))
                .apply(new JoinFunction<Auction, Bid, Tuple2<Auction, Bid>>() {
                    @Override
                    public Tuple2<Auction, Bid> join(Auction auction, Bid bid) {
                        return new Tuple2<Auction, Bid>(auction, bid);
                    }
                })
                .filter(t -> t.f0.dateTime <= t.f1.dateTime && t.f1.dateTime <= t.f0.expires)
                .keyBy(new KeySelector<Tuple2<Auction, Bid>, Tuple2<Long, Long>>() {

                    @Override
                    public Tuple2<Long, Long> getKey(Tuple2<Auction, Bid> t) throws Exception {
                        return new Tuple2<Long, Long>(t.f0.id, t.f0.category);
                    }
                })
                .window(TumblingEventTimeWindows.of(size))
                .apply(new WindowFunction<Tuple2<Auction, Bid>, Tuple2<Long, Long>, Tuple2<Long, Long>, TimeWindow>() {
                    @Override
                    public void apply(
                            Tuple2<Long, Long> key,
                            TimeWindow timeWindow,
                            Iterable<Tuple2<Auction, Bid>> iterable,
                            Collector<Tuple2<Long, Long>> collector
                    ) throws Exception {
                        long category = key.f1;
                        long max = Long.MIN_VALUE;
                        for (Tuple2<Auction, Bid> t : iterable) {
                            if (t.f1.price > max) {
                                max = t.f1.price;
                            }
                        }
                        collector.collect(new Tuple2<Long, Long>(category, max));
                    }
                })
                .keyBy(t -> t.f0)
                .window(TumblingEventTimeWindows.of(size))
                .apply(new WindowFunction<Tuple2<Long, Long>, Output, Long, TimeWindow>() {
                    @Override
                    public void apply(
                            Long category,
                            TimeWindow timeWindow,
                            Iterable<Tuple2<Long, Long>> iterable,
                            Collector<Output> collector
                    ) throws Exception {
                        long sum = 0;
                        for (Tuple2<Long, Long> t : iterable) {
                            sum += t.f1;
                        }
                        collector.collect(new Output(category, sum));
                    }
                })
                .addSink(new DataSink<Output>());
    }

    // Optimisations:
    // * Data pruning
    public static void runOpt(DataStream<Auction> auctions, DataStream<Bid> bids) {
        DataStream<PrunedAuction> prunedAuctions = auctions.map(a -> new PrunedAuction(a.id, a.category, a.expires, a.dateTime));
        DataStream<PrunedBid> prunedBids = bids.map(b -> new PrunedBid(b.auction, b.price, b.dateTime));
        prunedAuctions.join(prunedBids)
                .where(a -> a.id)
                .equalTo(b -> b.auction)
                .window(TumblingEventTimeWindows.of(size))
                .apply(new JoinFunction<PrunedAuction, PrunedBid, Tuple2<PrunedAuction, PrunedBid>>() {
                    @Override
                    public Tuple2<PrunedAuction, PrunedBid> join(PrunedAuction auction, PrunedBid bid) {
                        return new Tuple2<PrunedAuction, PrunedBid>(auction, bid);
                    }
                })
                .filter(t -> t.f0.dateTime <= t.f1.dateTime && t.f1.dateTime <= t.f0.expires)
                .keyBy(new KeySelector<Tuple2<PrunedAuction, PrunedBid>, Tuple2<Long, Long>>() {

                    @Override
                    public Tuple2<Long, Long> getKey(Tuple2<PrunedAuction, PrunedBid> t) throws Exception {
                        return new Tuple2<Long, Long>(t.f0.id, t.f0.category);
                    }
                })
                .window(TumblingEventTimeWindows.of(size))
                .apply(new WindowFunction<Tuple2<PrunedAuction, PrunedBid>, Tuple2<Long, Long>, Tuple2<Long, Long>, TimeWindow>() {
                    @Override
                    public void apply(
                            Tuple2<Long, Long> key,
                            TimeWindow timeWindow,
                            Iterable<Tuple2<PrunedAuction, PrunedBid>> iterable,
                            Collector<Tuple2<Long, Long>> collector
                    ) throws Exception {
                        long category = key.f1;
                        long max = Long.MIN_VALUE;
                        for (Tuple2<PrunedAuction, PrunedBid> t : iterable) {
                            if (t.f1.price > max) {
                                max = t.f1.price;
                            }
                        }
                        collector.collect(new Tuple2<Long, Long>(category, max));
                    }
                })
                .keyBy(t -> t.f0)
                .window(TumblingEventTimeWindows.of(size))
                .apply(new WindowFunction<Tuple2<Long, Long>, Output, Long, TimeWindow>() {
                    @Override
                    public void apply(
                            Long category,
                            TimeWindow timeWindow,
                            Iterable<Tuple2<Long, Long>> iterable,
                            Collector<Output> collector
                    ) throws Exception {
                        long sum = 0;
                        for (Tuple2<Long, Long> t : iterable) {
                            sum += t.f1;
                        }
                        collector.collect(new Output(category, sum));
                    }
                })
                .addSink(new DataSink<Output>());
    }
}

package org.example.queries;

import org.apache.flink.api.common.functions.JoinFunction;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.functions.windowing.WindowFunction;
import org.apache.flink.streaming.api.windowing.assigners.TumblingEventTimeWindows;
import org.apache.flink.streaming.api.windowing.time.Time;
import org.apache.flink.streaming.api.windowing.windows.GlobalWindow;
import org.apache.flink.util.Collector;
import org.example.data.Auction;
import org.example.data.Bid;
import org.example.io.DataSink;

public class Q6 {
    @JsonPropertyOrder({"seller", "avgBidPrice"})
    public static class Output {
        public long seller;
        public long avgBidPrice;

        public Output(long seller, long avgBidPrice) {
            this.seller = seller;
            this.avgBidPrice = avgBidPrice;
        }

        public Output() {
        }
    }

    public static class JoinOutput {
        public long auctionSeller;
        public long auctionExpires;
        public long auctionDateTime;
        public long bidPrice;
        public long bidDateTime;

        public JoinOutput(long auctionSeller,
                          long auctionExpires,
                          long auctionDateTime,
                          long bidPrice,
                          long bidDateTime) {
            this.auctionSeller = auctionSeller;
            this.auctionExpires = auctionExpires;
            this.auctionDateTime = auctionDateTime;
            this.bidPrice = bidPrice;
            this.bidDateTime = bidDateTime;
        }

        public JoinOutput() {
        }
    }

    public static class PrunedAuction {
        public long id;
        public long seller;
        public long expires;
        public long dateTime;

        public PrunedAuction(long id, long seller, long expires, long dateTime) {
            this.id = id;
            this.seller = seller;
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

    static Time timeSize = Time.seconds(10);
    static long countSize = 10;
    static long countSlide = 1;

    public static void run(DataStream<Auction> auctions, DataStream<Bid> bids) {
        auctions.join(bids)
                .where(a -> a.id)
                .equalTo(b -> b.auction)
                .window(TumblingEventTimeWindows.of(timeSize))
                .apply(new JoinFunction<Auction, Bid, JoinOutput>() {
                    @Override
                    public JoinOutput join(Auction auction, Bid bid) {
                        return new JoinOutput(auction.seller, auction.expires, auction.dateTime, bid.price, bid.dateTime);
                    }
                })
                .filter(i -> i.auctionDateTime < i.bidDateTime && i.bidDateTime < i.auctionExpires)
                .keyBy(i -> i.auctionSeller)
                .countWindow(countSize, countSlide)
                .apply(new WindowFunction<JoinOutput, Output, Long, GlobalWindow>() {

                    @Override
                    public void apply(Long seller, GlobalWindow globalWindow, Iterable<JoinOutput> iterable, Collector<Output> collector) throws Exception {
                        long sum = 0;
                        long count = 0;
                        for (JoinOutput i : iterable) {
                            sum += i.bidPrice;
                            count++;
                        }
                        collector.collect(new Output(seller, sum / count));
                    }
                })
                .addSink(new DataSink<Output>()) ;
    }

    public static void runOpt(DataStream<Auction> auctions, DataStream<Bid> bids) {
        DataStream<PrunedAuction> prunedAuctions = auctions.map(a -> new PrunedAuction(a.id, a.seller, a.expires, a.dateTime));
        DataStream<PrunedBid> prunedBids = bids.map(b -> new PrunedBid(b.auction, b.price, b.dateTime));
        prunedAuctions.join(prunedBids)
                .where(a -> a.id)
                .equalTo(b -> b.auction)
                .window(TumblingEventTimeWindows.of(timeSize))
                .apply(new JoinFunction<PrunedAuction, PrunedBid, JoinOutput>() {
                    @Override
                    public JoinOutput join(PrunedAuction auction, PrunedBid bid) {
                        return new JoinOutput(auction.seller, auction.expires, auction.dateTime, bid.price, bid.dateTime);
                    }
                })
                .filter(i -> i.auctionDateTime < i.bidDateTime && i.bidDateTime < i.auctionExpires)
                .keyBy(i -> i.auctionSeller)
                .countWindow(countSize, countSlide)
                .apply(new WindowFunction<JoinOutput, Output, Long, GlobalWindow>() {
                    @Override
                    public void apply(Long seller, GlobalWindow globalWindow, Iterable<JoinOutput> iterable, Collector<Output> collector) throws Exception {
                        long sum = 0;
                        long count = 0;
                        for (JoinOutput i : iterable) {
                            sum += i.bidPrice;
                            count++;
                        }
                        collector.collect(new Output(seller, sum / count));
                    }
                })
                .addSink(new DataSink<Output>()) ;
    }
}

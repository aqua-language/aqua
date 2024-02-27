package org.example.queries;

import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.example.data.Bid;
import org.example.io.DataSink;

public class Q1 {

    @JsonPropertyOrder({"auction", "price", "bidder", "dateTime"})
    public static class Output {
        public long auction;
        public long price;
        public long bidder;
        public long dateTime;

        public Output(long auction, long price, long bidder, long dateTime) {
            this.auction = auction;
            this.price = price;
            this.bidder = bidder;
            this.dateTime = dateTime;
        }

        public Output() {
        }
    }

    public static void run(DataStream<Bid> bids) {
        bids
                .map(b -> new Output(b.auction, (long) Math.floor(b.price * 0.85), b.bidder, b.dateTime))
                .returns(Output.class)
                .addSink(new DataSink<Output>());
    }

    public static void runOpt(DataStream<Bid> bids) {
        bids
                .map(b -> new Output(b.auction, (long) Math.floor(b.price * 0.85), b.bidder, b.dateTime))
                .returns(Output.class)
                .addSink(new DataSink<Output>());
    }
}
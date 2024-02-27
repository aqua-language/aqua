package org.example.queries;

import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.example.data.Bid;
import org.example.io.DataSink;
import org.example.operators.FilterMap;
import java.util.Optional;

public class Q2 {
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

    public static void run(DataStream<Bid> bids) {
        bids
                .filter(b -> b.auction == 1007
                        || b.auction == 1020
                        || b.auction == 2001
                        || b.auction == 2019
                        || b.auction == 2087)
                .map(b -> new Output(b.auction, b.price))
                .returns(Q2.Output.class)
                .addSink(new DataSink<Output>());
    }

    // Optimisations:
    // - Fuse filter and map
    public static void runOpt(DataStream<Bid> bids) {
        bids
                .process(new FilterMap<Bid, Q2.Output>(b -> {
                    if (b.auction == 1007
                            || b.auction == 1020
                            || b.auction == 2001
                            || b.auction == 2019
                            || b.auction == 2087) {
                        return Optional.of(new Q2.Output(b.auction, b.price));
                    } else {
                        return Optional.empty();
                    }
                }))
                .returns(Q2.Output.class)
                .addSink(new DataSink<Output>());
    }


}

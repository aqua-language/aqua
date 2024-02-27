package org.example.queries;

import org.apache.flink.api.common.functions.AggregateFunction;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.functions.windowing.AllWindowFunction;
import org.apache.flink.streaming.api.windowing.assigners.GlobalWindows;
import org.apache.flink.streaming.api.windowing.evictors.CountEvictor;
import org.apache.flink.streaming.api.windowing.triggers.CountTrigger;
import org.apache.flink.streaming.api.windowing.windows.GlobalWindow;
import org.apache.flink.util.Collector;
import org.example.data.Bid;
import org.example.io.DataSink;

public class QW {
    @JsonPropertyOrder({"person", "name", "reserve"})
    public static class Output {
        public float mean;
        public long min;
        public long max;
        public float stddev;

        public Output(float mean, long min, long max, float stddev) {
            this.mean = mean;
            this.min = min;
            this.max = max;
            this.stddev = stddev;
        }

        public Output() {
        }
    }

    public static void run(DataStream<Bid> bids, long size, long slide) {
        bids.windowAll(GlobalWindows.create())
                .evictor(CountEvictor.of(size))
                .trigger(CountTrigger.of(slide))
                .apply(new AllWindowFunction<Bid, Output, GlobalWindow>() {
                    @Override
                    public void apply(
                            GlobalWindow window,
                            Iterable<Bid> iterable,
                            Collector<Output> collector
                    ) throws Exception {
                        long sum = 0;
                        long count = 0;
                        long min = Long.MAX_VALUE;
                        long max = Long.MIN_VALUE;
                        for (Bid b : iterable) {
                            sum += b.price;
                            count++;
                            if (b.price < min) {
                                min = b.price;
                            }
                            if (b.price > max) {
                                max = b.price;
                            }
                        }
                        float mean = (float) sum / count;
                        float sum_squared_diffs = 0;
                        for (Bid b : iterable) {
                            sum_squared_diffs += (b.price - mean) * (b.price - mean);
                        }
                        float stddev = (float) Math.sqrt(sum_squared_diffs / count);
                        collector.collect(new Output(mean, min, max, stddev));
                    }
                })
                .addSink(new DataSink<Output>());
    }

    public static void runOpt(DataStream<Bid> bids, long size, long slide) {
        bids.windowAll(GlobalWindows.create())
                .evictor(CountEvictor.of(size))
                .trigger(CountTrigger.of(slide))
                .aggregate(new Aggregator())
                .addSink(new DataSink<Output>());
    }

    public static class Aggregator implements AggregateFunction<Bid, Aggregator, Output> {

        public long sum;
        public long count;
        public long min;
        public long max;
        public long sumSquares;

        public Aggregator(long sum, long count, long min, long max, long sumSquares) {
            this.sum = sum;
            this.count = count;
            this.min = min;
            this.max = max;
            this.sumSquares = sumSquares;
        }

        public Aggregator() {
        }

        @Override
        public Aggregator createAccumulator() {
            return new Aggregator(0, 0, Long.MAX_VALUE, Long.MIN_VALUE, 0);
        }

        @Override
        public Aggregator add(Bid bid, Aggregator partial) {
            return new Aggregator(
                    partial.sum + bid.price,
                    partial.count + 1,
                    Math.min(partial.min, bid.price),
                    Math.max(partial.max, bid.price),
                    partial.sumSquares + bid.price * bid.price
            );
        }

        @Override
        public Output getResult(Aggregator partial) {
            float mean = (float) partial.sum / partial.count;
            float variance = (float) partial.sumSquares / partial.count - mean * mean;
            return new Output(mean, partial.min, partial.max, (float) Math.sqrt(variance));
        }

        @Override
        public Aggregator merge(Aggregator partial, Aggregator acc1) {
            return new Aggregator(
                    partial.sum + acc1.sum,
                    partial.count + acc1.count,
                    Math.min(partial.min, acc1.min),
                    Math.max(partial.max, acc1.max),
                    partial.sumSquares + acc1.sumSquares
            );
        }
    }

}

package org.example.io;

import org.apache.flink.streaming.api.functions.source.RichSourceFunction;
import org.apache.flink.streaming.api.watermark.Watermark;

import java.io.Serializable;
import java.util.Iterator;

public class DataSource<T extends DataSource.TimestampExtractor<T>> extends RichSourceFunction<T> {

    private volatile boolean isRunning = true;
    private final long frequency;
    private final long slack;
    private final IteratorInit<T> uninitializedIterator;

    public DataSource(IteratorInit<T> uninitializedIterator, long slack, long frequency) {
        this.frequency = frequency;
        this.slack = slack;
        this.uninitializedIterator = uninitializedIterator;
    }

    @Override
    public void run(SourceContext<T> ctx) throws Exception {
        Iterator<T> iter = uninitializedIterator.init();
        long watermark = 0;
        long latestTimestamp = 0;
        long i = 0;
        while (isRunning && iter.hasNext()) {
            T event = iter.next();
            long timestamp = event.extractTimestamp();

            if (timestamp < watermark) {
                continue;
            }
            if (timestamp > latestTimestamp) {
                latestTimestamp = timestamp;
            }
            if (i % frequency == 0) {
                watermark = timestamp - slack;
                ctx.emitWatermark(new Watermark(watermark));
            }

            try {
                ctx.collectWithTimestamp(event, timestamp);
                i++;
            } catch (Exception e) {
                e.printStackTrace();
            }
        }
    }

    @Override
    public void cancel() {
        isRunning = false;
    }

    @FunctionalInterface
    public interface TimestampExtractor<T> extends Serializable {
        long extractTimestamp();
    }

    public interface IteratorInit<T> extends Serializable {
        Iterator<T> init();
    }
}
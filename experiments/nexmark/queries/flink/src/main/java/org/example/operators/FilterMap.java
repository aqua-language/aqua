package org.example.operators;

import org.apache.flink.streaming.api.functions.ProcessFunction;
import org.apache.flink.util.Collector;

import java.io.Serializable;
import java.util.Optional;

public class FilterMap<I, O> extends ProcessFunction<I, O> {
    final UDF<I, O> udf;

    public FilterMap(UDF<I, O> udf) {
        this.udf = udf;
    }

    @Override
    public void processElement(I i, ProcessFunction<I, O>.Context context, Collector<O> collector) {
        udf.udf(i).ifPresent(collector::collect);
    }

    @FunctionalInterface
    public interface UDF<I, O> extends Serializable {
        Optional<O> udf(I i);
    }
}

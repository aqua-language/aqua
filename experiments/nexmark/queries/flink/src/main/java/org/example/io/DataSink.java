package org.example.io;

import org.apache.flink.streaming.api.functions.sink.RichSinkFunction;

public class DataSink<T> extends RichSinkFunction<T> {

        public DataSink() {
        }

        @Override
        public void invoke(T value, Context context) throws Exception {
        }

}

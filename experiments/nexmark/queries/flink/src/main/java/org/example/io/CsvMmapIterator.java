package org.example.io;

import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.databind.MappingIterator;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.dataformat.csv.CsvMapper;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.dataformat.csv.CsvSchema;

import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.RandomAccessFile;
import java.io.Serializable;
import java.nio.MappedByteBuffer;
import java.nio.channels.FileChannel;
import java.nio.charset.StandardCharsets;
import java.util.Iterator;

public class CsvMmapIterator<T> implements Serializable, Iterator<T> {
    private final Iterator<T> iter;
    public CsvMmapIterator(String filePath, Class<T> targetType) {
        try (RandomAccessFile randomAccessFile = new RandomAccessFile(filePath, "r");
             FileChannel fileChannel = randomAccessFile.getChannel()) {

            MappedByteBuffer buffer = fileChannel.map(FileChannel.MapMode.READ_ONLY, 0, fileChannel.size());
            byte[] byteArray = new byte[buffer.remaining()];
            buffer.get(byteArray);

            CsvMapper mapper = new CsvMapper();
            CsvSchema schema = mapper
                    .schemaFor(targetType)
                    .withHeader();

            this.iter = mapper
                    .readerFor(targetType)
                    .with(schema)
                    .readValues(new String(byteArray, StandardCharsets.UTF_8));
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }

    @Override
    public boolean hasNext() {
        return iter.hasNext();
    }

    @Override
    public T next() {
        return iter.next();
    }
}

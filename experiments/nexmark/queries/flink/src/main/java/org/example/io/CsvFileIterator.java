package org.example.io;

import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.databind.MappingIterator;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.dataformat.csv.CsvMapper;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.dataformat.csv.CsvSchema;

import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;
import java.io.Serializable;
import java.util.Iterator;

public class CsvFileIterator<T> implements Serializable, Iterator<T> {
    private final MappingIterator<T> iter;

    public CsvFileIterator(String filePath, Class<T> targetType) {
        try {
            BufferedReader br = new BufferedReader(new FileReader(filePath));

            CsvMapper mapper = new CsvMapper();
            CsvSchema schema = mapper
                    .schemaFor(targetType)
                    .withHeader();

            this.iter = mapper
                    .readerFor(targetType)
                    .with(schema)
                    .readValues(br);
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
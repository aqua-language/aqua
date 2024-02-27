package org.example;

import org.apache.flink.api.common.JobExecutionResult;
import org.apache.flink.api.common.typeinfo.TypeHint;
import org.apache.flink.api.common.typeinfo.TypeInformation;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.environment.StreamExecutionEnvironment;
import org.example.data.Auction;
import org.example.data.Bid;
import org.example.data.Person;
import org.example.queries.*;
import org.example.io.*;

import java.util.Optional;

public class Main {
    public static void main(String[] args) throws Exception {

        if (args.length == 0) {
            System.out.println("No query specified");
            return;
        }
        String dir = args[0];
        String query = args[1];

        StreamExecutionEnvironment env = StreamExecutionEnvironment.getExecutionEnvironment();
        env.setParallelism(1);
        env.setMaxParallelism(1);

        Optional<DataStream<Auction>> auctions = read(env, dir + "/auctions.csv", Auction.class, TypeInformation.of(new TypeHint<Auction>() {
        }));
        Optional<DataStream<Person>> persons = read(env, dir + "/persons.csv", Person.class, TypeInformation.of(new TypeHint<Person>() {
        }));
        Optional<DataStream<Bid>> bids = read(env, dir + "/bids.csv", Bid.class, TypeInformation.of(new TypeHint<Bid>() {
        }));

        switch (query) {
            case "q1": {
                assert bids.isPresent();
                Q1.run(bids.get());
                break;
            }
            case "q2": {
                assert bids.isPresent();
                Q2.run(bids.get());
                break;
            }
            case "q3": {
                assert persons.isPresent() && auctions.isPresent();
                Q3.run(persons.get(), auctions.get());
                break;
            }
            case "q4": {
                assert auctions.isPresent() && bids.isPresent();
                Q4.run(auctions.get(), bids.get());
                break;
            }
            case "q5": {
                assert bids.isPresent();
                Q5.run(bids.get());
                break;
            }
            case "q6": {
                assert auctions.isPresent() && bids.isPresent();
                Q6.run(auctions.get(), bids.get());
                break;
            }
            case "q7": {
                assert bids.isPresent();
                Q7.run(bids.get());
                break;
            }
            case "q8": {
                assert auctions.isPresent() && persons.isPresent();
                Q8.run(auctions.get(), persons.get());
                break;
            }
            case "qw": {
                assert bids.isPresent();
                long size = Long.parseLong(args[2]);
                long slide = Long.parseLong(args[3]);
                QW.run(bids.get(), size, slide);
                break;
            }
            case "q1-opt": {
                assert bids.isPresent();
                Q1.runOpt(bids.get());
                break;
            }
            case "q2-opt": {
                assert bids.isPresent();
                Q2.runOpt(bids.get());
                break;
            }
            case "q3-opt": {
                assert persons.isPresent() && auctions.isPresent();
                Q3.runOpt(persons.get(), auctions.get());
                break;
            }
            case "q4-opt": {
                assert auctions.isPresent() && bids.isPresent();
                Q4.runOpt(auctions.get(), bids.get());
                break;
            }
            case "q5-opt": {
                assert bids.isPresent();
                Q5.runOpt(bids.get());
                break;
            }
            case "q6-opt": {
                assert auctions.isPresent() && bids.isPresent();
                Q6.runOpt(auctions.get(), bids.get());
                break;
            }
            case "q7-opt": {
                assert bids.isPresent();
                Q7.runOpt(bids.get());
                break;
            }
            case "q8-opt": {
                assert auctions.isPresent() && persons.isPresent();
                Q8.runOpt(auctions.get(), persons.get());
                break;
            }
            case "qw-opt": {
                assert bids.isPresent();
                long size = Long.parseLong(args[2]);
                long slide = Long.parseLong(args[3]);
                QW.runOpt(bids.get(), size, slide);
                break;
            }
            case "io": {
                auctions.ifPresent(auctionDataStream -> auctionDataStream.addSink(new DataSink<>()));
                persons.ifPresent(personDataStream -> personDataStream.addSink(new DataSink<>()));
                bids.ifPresent(bidDataStream -> bidDataStream.addSink(new DataSink<>()));
                break;
            }
            default:
                System.out.println("Unknown query: " + query);
                throw new IllegalArgumentException("Unknown query: " + query);
        }

        JobExecutionResult result = env.execute();
        System.err.println(result.getNetRuntime());
    }

    public static Long WatermarkFrequency = 1000L;
    public static Long Slack = 100L;
    public static <T extends DataSource.TimestampExtractor<T>> Optional<DataStream<T>> read(StreamExecutionEnvironment env, String path, Class<T> c, TypeInformation<T> ti) {
        if (new java.io.File(path).exists()) {
            return Optional.of(env
                    .addSource(new DataSource<T>(() -> new CsvFileIterator<T>(path, c), WatermarkFrequency, Slack))
                    .returns(ti));
        } else {
            return Optional.empty();
        }
    }
}

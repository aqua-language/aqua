package org.example.queries;

import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.functions.co.ProcessJoinFunction;
import org.apache.flink.streaming.api.windowing.time.Time;
import org.apache.flink.util.Collector;
import org.example.data.Auction;
import org.example.data.Person;
import org.example.io.DataSink;

public class Q8 {
    @JsonPropertyOrder({"person", "name", "reserve"})
    public static class Output {
        public long person;
        public String name;
        public long reserve;

        public Output(long person, String name, long reserve) {
            this.person = person;
            this.name = name;
            this.reserve = reserve;
        }

        public Output() {
        }
    }

    public static class PrunedAuction {
        public long seller;
        public long reserve;

        public PrunedAuction(long seller, long reserve) {
            this.seller = seller;
            this.reserve = reserve;
        }

        public PrunedAuction() {
        }
    }

    public static class PrunedPerson {
        public long id;
        public String name;

        public PrunedPerson(long id, String name) {
            this.id = id;
            this.name = name;
        }

        public PrunedPerson() {
        }
    }

    static Time lowerBound = Time.minutes(0);
    static Time upperBound = Time.minutes(1);

    public static void run(DataStream<Auction> auctions, DataStream<Person> persons) {
        persons.keyBy(p -> p.id)
                .intervalJoin(auctions.keyBy(a -> a.seller))
                .between(lowerBound, upperBound)
                .process(new ProcessJoinFunction<Person, Auction, Output>() {
                    @Override
                    public void processElement(Person person, Auction auction, ProcessJoinFunction<Person, Auction, Output>.Context context, Collector<Output> collector) throws Exception {
                        collector.collect(new Output(person.id, person.name, auction.reserve));
                    }
                })
                .returns(Output.class)
                .addSink(new DataSink<Output>());
    }

    public static void runOpt(DataStream<Auction> auctions, DataStream<Person> persons) {
        DataStream<PrunedPerson> prunedPersons = persons.map(p -> new PrunedPerson(p.id, p.name));
        DataStream<PrunedAuction> prunedAuctions = auctions.map(a -> new PrunedAuction(a.seller, a.reserve));
        prunedPersons.keyBy(p -> p.id)
                .intervalJoin(prunedAuctions.keyBy(a -> a.seller))
                .between(lowerBound, upperBound)
                .upperBoundExclusive()
                .process(new ProcessJoinFunction<PrunedPerson, PrunedAuction, Output>() {
                    @Override
                    public void processElement(PrunedPerson person, PrunedAuction auction, ProcessJoinFunction<PrunedPerson, PrunedAuction, Output>.Context context, Collector<Output> collector) throws Exception {
                        collector.collect(new Output(person.id, person.name, auction.reserve));
                    }
                })
                .returns(Output.class)
                .addSink(new DataSink<Output>());
    }
}

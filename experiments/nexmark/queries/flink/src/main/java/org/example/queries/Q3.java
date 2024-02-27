package org.example.queries;

import org.apache.flink.api.common.functions.JoinFunction;
import org.apache.flink.api.java.tuple.Tuple2;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.windowing.assigners.TumblingEventTimeWindows;
import org.apache.flink.streaming.api.windowing.time.Time;
import org.example.data.Auction;
import org.example.data.Person;
import org.example.io.DataSink;
import org.example.operators.FilterMap;

import java.util.Optional;

// Who is selling in OR, ID or CA in category 10, and for what auction ids?
public class Q3 {

    @JsonPropertyOrder({"name", "city", "state", "id"})
    public static class Output {
        public String name;
        public String city;
        public String state;
        public long id;

        public Output(String name, String city, String state, long id) {
            this.name = name;
            this.city = city;
            this.state = state;
            this.id = id;
        }

        public Output() {
        }
    }

    public static class PrunedPerson {
        public long id;
        public String name;
        public String city;
        public String state;

        public PrunedPerson(long id, String name, String city, String state) {
            this.id = id;
            this.name = name;
            this.city = city;
            this.state = state;
        }

        public PrunedPerson() {
        }
    }

    public static class PrunedAuction {
        public long seller;
        public long id;

        public PrunedAuction(long seller, long id) {
            this.seller = seller;
            this.id = id;
        }

        public PrunedAuction() {
        }
    }

    static Time size = Time.seconds(10);

    public static void run(DataStream<Person> persons, DataStream<Auction> auctions) {
        auctions.join(persons)
                .where(auction -> auction.seller)
                .equalTo(person -> person.id)
                .window(TumblingEventTimeWindows.of(size))
                .apply(new JoinFunction<Auction, Person, Tuple2<Auction, Person>>() {
                    @Override
                    public Tuple2<Auction, Person> join(Auction auction, Person person) {
                        return new Tuple2<Auction, Person>(auction, person);
                    }
                })
                .filter(tuple ->
                        (tuple.f1.state.equals("or") || tuple.f1.state.equals("id") || tuple.f1.state.equals("ca")) && tuple.f0.category == 10
                )
                .map(tuple -> new Output(tuple.f1.name, tuple.f1.city, tuple.f1.state, tuple.f0.id))
                .returns(Output.class)
                .addSink(new DataSink<Output>());
    }

    // Optimisations:
    // - Data pruning
    // - Predicate pushdown
    // - Fuse filter and map
    public static void runOpt(DataStream<Person> persons, DataStream<Auction> auctions) {
        DataStream<PrunedPerson> persons2 = persons
                .process(new FilterMap<Person, PrunedPerson>(p -> {
                    if (p.state.equals("or") || p.state.equals("id") || p.state.equals("ca")) {
                        return Optional.of(new PrunedPerson(p.id, p.name, p.city, p.state));
                    } else {
                        return Optional.empty();
                    }
                }))
                .returns(PrunedPerson.class);
        DataStream<PrunedAuction> auctions2 = auctions
                .process(new FilterMap<Auction, PrunedAuction>(a -> {
                    if (a.category == 10) {
                        return Optional.of(new PrunedAuction(a.seller, a.id));
                    } else {
                        return Optional.empty();
                    }
                }))
                .returns(PrunedAuction.class);
        auctions2
                .join(persons2)
                .where(a -> a.seller)
                .equalTo(p -> p.id)
                .window(TumblingEventTimeWindows.of(size))
                .apply((a, p) -> new Output(p.name, p.city, p.state, a.id))
                .addSink(new DataSink<Output>());
    }
}

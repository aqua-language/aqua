package org.example.data;

import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.example.io.DataSource;

import java.io.Serializable;
import java.time.Instant;

@JsonPropertyOrder({"id", "name", "emailAddress", "creditCard", "city", "state", "dateTime", "extra"})
public class Person implements Serializable, DataSource.TimestampExtractor<Person> {
    public long id;
    public String name;
    public String emailAddress;
    public String creditCard;
    public String city;
    public String state;
    public long dateTime;
    public String extra;

    public Person(long id, String name, String emailAddress, String creditCard, String city, String state, long dateTime, String extra) {
        this.id = id;
        this.name = name;
        this.emailAddress = emailAddress;
        this.creditCard = creditCard;
        this.city = city;
        this.state = state;
        this.dateTime = dateTime;
        this.extra = extra;
    }

    public Person() {
    }

    @Override
    public long extractTimestamp() {
        return this.dateTime;
    }
}

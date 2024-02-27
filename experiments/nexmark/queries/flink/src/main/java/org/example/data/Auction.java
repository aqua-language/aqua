package org.example.data;

import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.example.io.DataSource;

import java.io.Serializable;
import java.time.Instant;

@JsonPropertyOrder({"id", "itemName", "description", "initialBid", "reserve", "dateTime", "expires", "seller", "category", "extra"})
public class Auction implements Serializable, DataSource.TimestampExtractor<Auction> {
    public long id;
    public String itemName;
    public String description;
    public long initialBid;
    public long reserve;
    public long dateTime;
    public long expires;
    public long seller;
    public long category;
    public String extra;

    public Auction(long id, String itemName, String description, long initialBid, long reserve, long dateTime, long expires, long seller, long category, String extra) {
        this.id = id;
        this.itemName = itemName;
        this.description = description;
        this.initialBid = initialBid;
        this.reserve = reserve;
        this.dateTime = dateTime;
        this.expires = expires;
        this.seller = seller;
        this.category = category;
        this.extra = extra;
    }

    public Auction() {
    }

    @Override
    public long extractTimestamp() {
        return this.dateTime;
    }
}


use maxminddb::geoip2;
use std::net::SocketAddr;

pub fn ipaddr_lookup(addr: SocketAddr) -> String {
    let reader = maxminddb::Reader::open_readfile("./config/ip.mmdb").unwrap();
    let record = reader.lookup::<geoip2::City>(addr.ip());
    // println!("{:?}", record);
    match record {
        Ok(record) => {
            let mut location: String = String::new();
            if let Some(city) = record.city {
                if let Some(city_name_map) = city.names {
                    if let Some(en) = city_name_map.get("en") {
                        location.push_str(en);
                    }
                }
            }
            if let Some(subdivisions) = record.subdivisions {
                subdivisions.iter().for_each(|subdivision| {
                    if let Some(subdivision_name_map) = &subdivision.names {
                        if let Some(en) = subdivision_name_map.get("en") {
                            if !location.is_empty() {
                                location.push_str(", ");
                            }
                            location.push_str(en);
                        }
                    }
                });
            }
            if let Some(country) = record.country {
                if let Some(country_name_map) = country.names {
                    if let Some(en) = country_name_map.get("en") {
                        if location.is_empty() {
                            location.push_str(en);
                        } else {
                            location.push_str(", ");
                            location.push_str(en);
                        }
                    }
                }
            }
            if location.is_empty() {
                location.push_str("Unknown")
            }
            location
        },
        Err(_) => "Unknown".to_string(),
    }
}
use maxminddb::geoip2;
use std::net::SocketAddr;

pub fn ipaddr_lookup(addr: SocketAddr) -> String {
    let reader = maxminddb::Reader::open_readfile("./config/ip.mmdb").unwrap();
    let record = reader.lookup::<geoip2::City>(addr.ip());
    match record {
        Ok(record) => {
            if let Some(city) = record.city {
                if let Some(city_name_map) = city.names {
                    if let Some(en) = city_name_map.get("en") {
                        return en.to_string();
                    }
                }
            }
            "Unknown".to_string()
        },
        Err(_) => "Unknown".to_string(),
    }
}
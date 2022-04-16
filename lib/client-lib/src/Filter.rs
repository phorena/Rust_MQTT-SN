use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::Connection::{ConnId, Connection};
use std::net::SocketAddr;
use uuid::v1::{Context, Timestamp};
use uuid::Uuid;

/// Checks if a topic or topic filter has wildcards
#[inline(always)]
pub fn has_wildcards(filter: &str) -> bool {
    filter.contains('+') || filter.contains('#')
}

// https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718106
// A subscription topic filter can contain # or + to allow the client to
// subscribe to multiple topics at once.
#[inline(always)]
pub fn valid_filter(filter: &str) -> bool {
    if !filter.is_empty() {
        if has_wildcards(filter) {
            // Verify multi level wildcards.
            if filter.find('#') == Some(filter.len() - 1)
                && filter.ends_with("/#")
            {
                return true;
            }
        // TODO verify single level wildcards.
        } else {
            return true;
        }
    }
    false
}

// XXX copy from rumqtt
/// Checks if topic matches a filter. topic and filter validation isn't done here.
///
/// **NOTE**: 'topic' is a misnomer in the arg. this can also be used to match 2 wild subscriptions
/// **NOTE**: make sure a topic is validated during a publish and filter is validated
/// during a subscribe
#[inline(always)]
pub fn match_topic(topic: &str, filter: &str) -> bool {
    if !topic.is_empty() && topic[..1].contains('$') {
        return false;
    }

    let mut topics = topic.split('/');
    let mut filters = filter.split('/');

    for f in filters.by_ref() {
        // "#" being the last element is validated by the broker with 'valid_filter'
        if f == "#" {
            return true;
        }

        // filter still has remaining elements
        // filter = a/b/c/# should match topci = a/b/c
        // filter = a/b/c/d should not match topic = a/b/c
        let top = topics.next();
        match top {
            Some(t) if t == "#" => return false,
            Some(_) if f == "+" => continue,
            Some(t) if f != t => return false,
            Some(_) => continue,
            None => return false,
        }
    }

    // topic has remaining elements and filter's last element isn't "#"
    if topics.next().is_some() {
        return false;
    }

    true
}

#[derive(Debug, Clone)]
pub struct Filter {
    wildcard_topics: HashMap<String, Arc<Mutex<HashSet<ConnId>>>>,
    wildcard_filters: HashMap<String, Arc<Mutex<HashSet<ConnId>>>>,
    concrete_topics: HashMap<String, Arc<Mutex<HashSet<ConnId>>>>,
}

impl Filter {
    pub fn new() -> Self {
        Filter {
            wildcard_topics: HashMap::new(),
            wildcard_filters: HashMap::new(),
            concrete_topics: HashMap::new(),
        }
    }
    // TODO return better error
    /// Insert a new filter/subscription from a connection.
    #[inline(always)]
    pub fn insert(&mut self, filter: &str, id: ConnId) -> bool {
        if valid_filter(filter) {
            if has_wildcards(filter) {
                let conn_set = self
                    .wildcard_filters
                    .entry(filter.to_string())
                    .or_insert(Arc::new(Mutex::new(HashSet::new())));
                let mut conn_set = conn_set.lock().unwrap();
                conn_set.insert(id);
            } else {
                let conn_set = self
                    .concrete_topics
                    .entry(filter.to_string())
                    .or_insert(Arc::new(Mutex::new(HashSet::new())));
                let mut conn_set = conn_set.lock().unwrap();
                conn_set.insert(id);
            }
            return true;
        }
        false
    }

    #[inline(always)]
    pub fn match_topic_concrete(
        &mut self,
        topic: &str,
    ) -> Option<HashSet<ConnId>> {
        let id = Uuid::new_v4();
        if let Some(id_set) = self.concrete_topics.get(topic) {
            return Some(id_set.lock().unwrap().clone());
        }
        None
    }

    #[inline(always)]
    pub fn match_topic_wildcard(
        &mut self,
        topic: &str,
    ) -> Option<HashSet<ConnId>> {
        // Topic is in the wildcard_topics map.
        if let Some(id_set) = self.wildcard_topics.get(topic) {
            return Some(id_set.lock().unwrap().clone());
        } else {
            // Publish topic shouldn't have wildcards.
            if has_wildcards(topic) {
                return None;
            }
            // Match the topic against all wildcard filters.
            // Insert the topic into wildcard_topics if matched.
            for (filter, id_set) in &self.wildcard_filters {
                // dbg!((filter, id_set));
                if match_topic(topic, filter) {
                    // dbg!((filter, id_set));
                    self.wildcard_topics
                        .insert(topic.to_string(), id_set.clone());
                }
            }
            // Return the topic's wildcard_topics set.
            if let Some(id_set) = self.wildcard_topics.get(topic) {
                return Some(id_set.lock().unwrap().clone());
            }
        }
        None
    }

    // Doesn't work correctly.
    pub fn match_topic(&mut self, topic: &str) -> Option<HashSet<ConnId>> {
        // Publish topic shouldn't have wildcards.
        if has_wildcards(topic) {
            return None;
        }

        let mut new_set: HashSet<ConnId> = HashSet::new();
        if let Some(socket_set) = self.wildcard_topics.get(topic) {
            // return Some(socket_set.lock().unwrap().clone());
            let wildcard_set = socket_set.lock().unwrap().clone();
            new_set.extend(&wildcard_set);
        } else {
            for (filter, socket_set) in &self.wildcard_filters {
                dbg!((filter, socket_set));
                if match_topic(topic, filter) {
                    dbg!((filter, socket_set));
                    self.wildcard_topics
                        .insert(topic.to_string(), socket_set.clone());
                }
            }
        }
        if let Some(socket_set) = self.concrete_topics.get(topic) {
            // return Some(socket_set.lock().unwrap().clone());
            let concrete_set = socket_set.lock().unwrap().clone();
            new_set.extend(&concrete_set);
        }
        if !new_set.is_empty() {
            return Some(new_set);
        }
        None
    }
}
#[repr(C)]
pub union MyUnion {
    sock_addr: SocketAddr,
    bytes: [u8; 6],
}

#[cfg(test)]
mod test {
    use std::net::SocketAddr;
    use uuid::Uuid;

    use uuid::v1::{Context, Timestamp};

    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_insert() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        let socket = "127.0.0.1:1200".parse::<SocketAddr>().unwrap();
        let socket_str = socket.to_string();
        dbg!(socket_str);
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        println!("{:?}", since_the_epoch);
        let in_ns = since_the_epoch.as_nanos() as u32;
        let in_s = since_the_epoch.as_secs();
        println!("{:?}", in_ns);
        println!("{:?}", in_s);
        let context = Context::new(42);
        let ts = Timestamp::from_unix(&context, in_s, in_ns);
        let mut ip4bytes: [u8; 4] = [0; 4];
        let port_bytes: [u8; 2] = socket.port().to_be_bytes();

        match socket.ip() {
            IpAddr::V4(ip4) => ip4bytes = ip4.octets(),
            IpAddr::V6(ip6) => {
                println!("ipv6: {}, segments: {:?}", ip6, ip6.segments())
            }
        }
        dbg!(ip4bytes);
        dbg!(port_bytes);
        let zz: [u8; 6] = [
            ip4bytes[0],
            ip4bytes[1],
            ip4bytes[2],
            ip4bytes[3],
            port_bytes[0],
            port_bytes[1],
        ];
        dbg!(zz);

        let uuid = Uuid::new_v1(ts, &zz).expect("failed to generate UUID");
        dbg!((&context, ts, uuid));
        let uuid =
            Uuid::new_v1(ts, b"123456").expect("failed to generate UUID");
        dbg!((&context, ts, uuid));
        let context = Context::new(42);
        let ts = Timestamp::from_unix(&context, in_s, in_ns);
        let uuid = Uuid::new_v1(ts, &[192, 168, 0, 4, 8, 7])
            .expect("failed to generate UUID");
        dbg!((&context, ts, uuid));
        let context = Context::new(45);
        let ts = Timestamp::from_unix(&context, in_s, in_ns);
        let uuid = Uuid::new_v1(ts, &[1, 2, 3, 4, 5, 6])
            .expect("failed to generate UUID");
        dbg!((context, ts, uuid));

        let mut filter = super::Filter::new();
        let id = uuid::Uuid::new_v4();
        filter.insert("aa/bb", id);
        filter.insert("aa/cc", id);
        let id = uuid::Uuid::new_v4();
        filter.insert("aa/bb", id);
        let mut r = filter.match_topic("aa/bb").unwrap();
        dbg!(&r);

        // Test for r is a pointer to the same set as the filter's set.
        // Answer is no. r is a copy of the set, not pointer to the set.
        let id = uuid::Uuid::new_v4();
        r.insert(id);
        dbg!(&filter);

        let id = uuid::Uuid::new_v4();
        filter.insert("aa/#", id);
        let id = uuid::Uuid::new_v4();
        filter.insert("aa/#", id);
        let id = uuid::Uuid::new_v4();
        filter.insert("bb/#", id);
        let r = filter.match_topic_concrete("aa/bb");
        dbg!(&r);
        let r = filter.match_topic_wildcard("aa/dd");
        dbg!(&r);
        let r = filter.match_topic_wildcard("zz/dd");
        dbg!(&r);
        dbg!(&filter);
    }

    /*
    #[test]
    fn filer_add() {
        let mut filter = super::Filter::new();
        assert!(filter.add("a/b/c"));
        assert!(filter.add("a/b/#"));
        dbg!(filter);
    }

    #[test]
    fn filer_match() {
        let mut filter = super::Filter::new();
        assert!(filter.add("a/b/c"));
        assert!(filter.add("a/b/#"));
        // TODO implement + wildcard
        assert!(!filter.add("a/+/e"));
        assert!(!filter.match_topic("a/b/#"));
        assert!(filter.match_topic("a/b/c"));
        assert!(filter.match_topic("a/b/d"));
        assert!(filter.match_topic("a/b/e"));
        dbg!(filter);
    }

    #[test]
    fn wildcards_are_detected_correctly() {
        assert!(!super::has_wildcards("a/b/c"));
        assert!(super::has_wildcards("a/+/c"));
        assert!(super::has_wildcards("a/b/#"));
    }

    #[test]
    fn filters_are_validated_correctly() {
        assert!(!super::valid_filter("wrong/#/filter"));
        assert!(!super::valid_filter("wrong/wr#ng/filter"));
        assert!(!super::valid_filter("wrong/filter#"));
        assert!(super::valid_filter("correct/filter/#"));
        assert!(super::valid_filter("correct/filter/"));
        assert!(super::valid_filter("correct/filter"));
        assert!(!super::valid_filter(""));
    }

    #[test] // TODO learn more about this from rumqtt
    fn dollar_subscriptions_doesnt_match_dollar_topic() {
        assert!(super::match_topic("sy$tem/metrics", "sy$tem/+"));
        assert!(!super::match_topic("$system/metrics", "$system/+"));
        assert!(!super::match_topic("$system/metrics", "+/+"));
    }

    #[test]
    fn topics_match_with_filters_as_expected() {
        let topic = "a/b/c";
        let filter = "a/b/c";
        assert!(super::match_topic(topic, filter));

        let topic = "a/b/c";
        let filter = "d/b/c";
        assert!(!super::match_topic(topic, filter));

        let topic = "a/b/c";
        let filter = "a/b/e";
        assert!(!super::match_topic(topic, filter));

        let topic = "a/b/c";
        let filter = "a/b/c/d";
        assert!(!super::match_topic(topic, filter));

        let topic = "a/b/c";
        let filter = "#";
        assert!(super::match_topic(topic, filter));

        let topic = "a/b/c";
        let filter = "a/b/c/#";
        assert!(super::match_topic(topic, filter));

        let topic = "a/b/c/d";
        let filter = "a/b/c";
        assert!(!super::match_topic(topic, filter));

        let topic = "a/b/c/d";
        let filter = "a/b/c/#";
        assert!(super::match_topic(topic, filter));

        let topic = "a/b/c/d/e/f";
        let filter = "a/b/c/#";
        assert!(super::match_topic(topic, filter));

        let topic = "a/b/c";
        let filter = "a/+/c";
        assert!(super::match_topic(topic, filter));
        let topic = "a/b/c/d/e";
        let filter = "a/+/c/+/e";
        assert!(super::match_topic(topic, filter));

        let topic = "a/b";
        let filter = "a/b/+";
        assert!(!super::match_topic(topic, filter));

        let filter1 = "a/b/+";
        let filter2 = "a/b/#";
        assert!(super::match_topic(filter1, filter2));
        assert!(!super::match_topic(filter2, filter1));

        let filter1 = "a/b/+";
        let filter2 = "#";
        assert!(super::match_topic(filter1, filter2));

        let filter1 = "a/+/c/d";
        let filter2 = "a/+/+/d";
        assert!(super::match_topic(filter1, filter2));
        assert!(!super::match_topic(filter2, filter1));
    }
    */
}

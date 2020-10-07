use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fmt::Debug,
    string::*,
};

#[derive(Debug)]
pub struct TopicTree {
    sub_topics: HashMap<String, Box<TopicTree>>,
    topic_subscribers_id: HashSet<String>,
}

impl TopicTree {
    pub fn new_root() -> TopicTree {
        let mut topic = TopicTree {
            sub_topics: HashMap::new(),
            topic_subscribers_id: HashSet::new(),
        };
        topic
    }

    fn new<S1: AsRef<str>, S2: AsRef<str>>(topic_str: S1, topic_subscriber_id: S2) -> TopicTree {
        let mut topic = TopicTree {
            sub_topics: HashMap::new(),
            topic_subscribers_id: HashSet::new(),
        };
        let splitted_topic: Vec<&str> = topic_str.as_ref().splitn(2, "/").collect();
        let topic_str = splitted_topic[0].to_string();
        if !topic_str.is_empty() {
            if splitted_topic.len() > 1 {
                topic
                    .sub_topics
                    .insert(topic_str, Box::new(TopicTree::new(splitted_topic[1].to_string(), topic_subscriber_id)));
            } else {
                topic.sub_topics.insert(topic_str, Box::new(TopicTree::new("", topic_subscriber_id)));
            }
        } else {
            topic.topic_subscribers_id.insert(String::from(topic_subscriber_id.as_ref()));
        }
        topic
    }

    pub fn subscribe<S1: AsRef<str>, S2: AsRef<str>>(&mut self, topic_str: S1, topic_subscriber_id: S2) {
        let splitted_topic: Vec<&str> = topic_str.as_ref().splitn(2, "/").collect();
        let topic_str = splitted_topic[0].to_string();
        if !topic_str.is_empty() {
            if splitted_topic.len() > 1 {
                match self.sub_topics.entry(topic_str) {
                    Entry::Occupied(o) => o.into_mut().subscribe(splitted_topic[1].to_string(), topic_subscriber_id),
                    Entry::Vacant(v) => {
                        v.insert(Box::new(TopicTree::new(splitted_topic[1].to_string(), topic_subscriber_id)));
                    }
                };
            } else {
                match self.sub_topics.entry(topic_str) {
                    Entry::Occupied(o) => o.into_mut().subscribe("", topic_subscriber_id),
                    Entry::Vacant(v) => {
                        v.insert(Box::new(TopicTree::new("", topic_subscriber_id)));
                    }
                }
            }
        } else {
            self.topic_subscribers_id.insert(String::from(topic_subscriber_id.as_ref()));
        }
    }

    fn get_subscribers_id<S: AsRef<str>>(&mut self, topic_str: S) -> Option<Vec<String>> {
        if topic_str.as_ref().is_empty() {
            return Some(self.topic_subscribers_id.clone().into_iter().collect());
        } else {
            let splitted_topic: Vec<&str> = topic_str.as_ref().splitn(2, "/").collect();
            if splitted_topic.len() > 1 {
                return self.sub_topics.get_mut(splitted_topic[0]).unwrap().get_subscribers_id(splitted_topic[1]);
            } else {
                return match self.sub_topics.entry(String::from(splitted_topic[0])) {
                    Entry::Occupied(o) => o.into_mut().get_subscribers_id(""),
                    Entry::Vacant(v) => None,
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn topic_test() {
        let mut root_topic = TopicTree::new_root();
        root_topic.subscribe("hello/beto", "&mut observer1");
        root_topic.subscribe("a/b/c", "&mut observer2");
        root_topic.subscribe("a/g/d", "&mut observer3");
        root_topic.subscribe("a/g/d", "&mut observer4");
        root_topic.subscribe("a/g/d", "&mut observer3");
        println!("{:#?}", root_topic.get_subscribers_id("a/b/c"));
        println!("{:#?}", root_topic.get_subscribers_id("a/g"));
        println!("{:#?}", root_topic.get_subscribers_id("a/g/d"));
    }
    #[test]
    fn splitn_test() {
        let datas: Vec<&str> = "hello".splitn(2, "/").collect();
        println!("{:#?}", datas);
    }
}

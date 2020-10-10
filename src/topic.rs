use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fmt::Debug,
    string::*,
};

#[derive(Debug)]
pub struct TopicTree {
    sub_topics: HashMap<String, Box<TopicTree>>,
    topic_subscribers_id: HashSet<String>,
    multi_level_topic_subscribers_id: HashSet<String>,
    single_level_topic_subscribers_id: HashSet<String>,
}

impl TopicTree {
    pub fn new_root() -> TopicTree {
        let topic = TopicTree {
            sub_topics: HashMap::new(),
            topic_subscribers_id: HashSet::new(),
            multi_level_topic_subscribers_id: HashSet::new(),
            single_level_topic_subscribers_id: HashSet::new(),
        };
        topic
    }

    fn new<S1: AsRef<str>, S2: AsRef<str>>(topic_str: S1, topic_subscriber_id: S2) -> TopicTree {
        let mut topic = TopicTree {
            sub_topics: HashMap::new(),
            topic_subscribers_id: HashSet::new(),
            multi_level_topic_subscribers_id: HashSet::new(),
            single_level_topic_subscribers_id: HashSet::new(),
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
            if topic_str == "#" {
                self.multi_level_topic_subscribers_id.insert(String::from(topic_subscriber_id.as_ref()));
            } else if splitted_topic.len() > 1 {
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
            if self.multi_level_topic_subscribers_id.len() > 0 {
                for (_, elem) in self.sub_topics.iter_mut() {
                    for multi_ids in self.multi_level_topic_subscribers_id.iter() {
                        elem.subscribe("#", multi_ids);
                    }
                }
            }
        } else {
            self.topic_subscribers_id.insert(String::from(topic_subscriber_id.as_ref()));
        }
    }

    fn get_subscribers_id<S: AsRef<str>>(&mut self, topic_str: S) -> Option<Vec<String>> {
        if topic_str.as_ref().is_empty() {
            return match self.topic_subscribers_id.len() + self.multi_level_topic_subscribers_id.len() {
                0 => None,
                _ => Some(
                    self.topic_subscribers_id
                        .clone()
                        .into_iter()
                        .chain(self.multi_level_topic_subscribers_id.clone())
                        .collect(),
                ),
            };
        } else {
            let splitted_topic: Vec<&str> = topic_str.as_ref().splitn(2, "/").collect();
            if splitted_topic.len() > 1 {
                return self.sub_topics.get_mut(splitted_topic[0]).unwrap().get_subscribers_id(splitted_topic[1]);
            } else {
                return match self.sub_topics.entry(String::from(splitted_topic[0])) {
                    Entry::Occupied(o) => o.into_mut().get_subscribers_id(""),
                    Entry::Vacant(v) => match self.multi_level_topic_subscribers_id.len() {
                        0 => None,
                        _ => Some(self.multi_level_topic_subscribers_id.clone().into_iter().collect()),
                    },
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    #[test]
    fn topic_test() {
        let start = Instant::now();
        let mut root_topic = TopicTree::new_root();
        root_topic.subscribe("hello/beto", "observer1");
        root_topic.subscribe("a/b/c", "observer2");
        root_topic.subscribe("a/g/d", "observer3");
        root_topic.subscribe("a/g/e", "observer4");
        root_topic.subscribe("a/g/z", "observer3");
        root_topic.subscribe("a/g/#", "observer#");
        root_topic.subscribe("d/g/a", "observer5");
        root_topic.subscribe("b/a/a", "observer6");
        root_topic.subscribe("#", "observer root");
        //println!("{:#?}", root_topic);
        println!("{:#?}", root_topic.get_subscribers_id("a/b/c"));
        println!("{:#?}", root_topic.get_subscribers_id("a/g"));
        println!("{:#?}", root_topic.get_subscribers_id("a"));
        println!("{:#?}", root_topic.get_subscribers_id("a/g/d"));
        println!("{:#?}", root_topic.get_subscribers_id("a/g/e"));
        println!("{:#?}", root_topic.get_subscribers_id("a/g/m"));
        println!("{:#?}", root_topic.get_subscribers_id("hello/beto"));
        let duration = start.elapsed();
        println!("Time elapsed in topic_test() is: {:?}", duration);
    }
    #[test]
    fn splitn_test() {
        let datas: Vec<&str> = "hello".splitn(2, "/").collect();
        println!("{:#?}", datas);
    }
}

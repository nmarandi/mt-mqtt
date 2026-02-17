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
        TopicTree {
            sub_topics: HashMap::new(),
            topic_subscribers_id: HashSet::new(),
            multi_level_topic_subscribers_id: HashSet::new(),
            single_level_topic_subscribers_id: HashSet::new(),
        }
    }

    #[allow(dead_code)]
    fn new<S1: AsRef<str>, S2: AsRef<str>>(topic_str: S1, topic_subscriber_id: S2) -> TopicTree {
        let mut topic = TopicTree {
            sub_topics: HashMap::new(),
            topic_subscribers_id: HashSet::new(),
            multi_level_topic_subscribers_id: HashSet::new(),
            single_level_topic_subscribers_id: HashSet::new(),
        };
        let splitted_topic: Vec<&str> = topic_str.as_ref().splitn(2, '/').collect();
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
        let splitted_topic: Vec<&str> = topic_str.as_ref().splitn(2, '/').collect();
        let topic_str = splitted_topic[0].to_string();
        if !topic_str.is_empty() {
            if topic_str == "#" {
                self.multi_level_topic_subscribers_id.insert(String::from(topic_subscriber_id.as_ref()));
            } else if topic_str == "+" {
                // Handle single-level wildcard
                if splitted_topic.len() > 1 {
                    // Pattern like "+/data" or "+/data/#" - need to create + subtopic
                    let remaining = splitted_topic[1].to_string();
                    match self.sub_topics.entry("+".to_string()) {
                        Entry::Occupied(o) => o.into_mut().subscribe(remaining, topic_subscriber_id),
                        Entry::Vacant(v) => {
                            let mut new_node = TopicTree::new_root();
                            new_node.subscribe(remaining, topic_subscriber_id);
                            v.insert(Box::new(new_node));
                        }
                    }
                } else {
                    // Just "+" - add to single-level subscribers
                    self.single_level_topic_subscribers_id.insert(String::from(topic_subscriber_id.as_ref()));
                }
            } else if splitted_topic.len() > 1 {
                // Handle multi-level topics like "sensor/temp"
                let remaining = splitted_topic[1].to_string();
                // Check if the remaining part is a wildcard
                if remaining == "#" {
                    // Add to multi-level subscribers of the current topic
                    match self.sub_topics.entry(topic_str.clone()) {
                        Entry::Occupied(o) => {
                            o.into_mut()
                                .multi_level_topic_subscribers_id
                                .insert(String::from(topic_subscriber_id.as_ref()));
                        }
                        Entry::Vacant(v) => {
                            let mut new_node = TopicTree::new_root();
                            new_node
                                .multi_level_topic_subscribers_id
                                .insert(String::from(topic_subscriber_id.as_ref()));
                            v.insert(Box::new(new_node));
                        }
                    }
                } else if remaining.starts_with("+/") || remaining == "+" {
                    // Single-level wildcard
                    match self.sub_topics.entry(topic_str.clone()) {
                        Entry::Occupied(o) => o.into_mut().subscribe(remaining, topic_subscriber_id),
                        Entry::Vacant(v) => {
                            let mut new_node = TopicTree::new_root();
                            new_node.subscribe(remaining, topic_subscriber_id);
                            v.insert(Box::new(new_node));
                        }
                    }
                } else {
                    // Regular topic path
                    match self.sub_topics.entry(topic_str.clone()) {
                        Entry::Occupied(o) => o.into_mut().subscribe(remaining, topic_subscriber_id),
                        Entry::Vacant(v) => {
                            let mut new_node = TopicTree::new_root();
                            new_node.subscribe(remaining, topic_subscriber_id);
                            v.insert(Box::new(new_node));
                        }
                    }
                }
            } else {
                match self.sub_topics.entry(topic_str) {
                    Entry::Occupied(o) => o.into_mut().subscribe("", topic_subscriber_id),
                    Entry::Vacant(v) => {
                        let mut new_node = TopicTree::new_root();
                        new_node.subscribe("", topic_subscriber_id);
                        v.insert(Box::new(new_node));
                    }
                }
            }
            if !self.multi_level_topic_subscribers_id.is_empty() {
                for (_, elem) in self.sub_topics.iter_mut() {
                    for multi_ids in self.multi_level_topic_subscribers_id.iter() {
                        elem.subscribe("#", multi_ids);
                    }
                }
            }
            if !self.single_level_topic_subscribers_id.is_empty() {
                for (_, elem) in self.sub_topics.iter_mut() {
                    for multi_ids in self.single_level_topic_subscribers_id.iter() {
                        if splitted_topic.len() > 1 {
                            elem.subscribe(splitted_topic[1].to_string(), multi_ids);
                        } else {
                            elem.subscribe("", multi_ids);
                        }
                    }
                }
            }
        } else {
            self.topic_subscribers_id.insert(String::from(topic_subscriber_id.as_ref()));
        }
    }

    pub fn unsubscribe<S: AsRef<str>, T: AsRef<str>>(&mut self, topic_str: S, topic_subscriber_id: T) {
        let splitted_topic: Vec<&str> = topic_str.as_ref().split('/').filter(|x| !x.is_empty()).collect();

        if splitted_topic.is_empty() {
            self.topic_subscribers_id.remove(topic_subscriber_id.as_ref());
            return;
        }

        let topic = splitted_topic[0];

        // Handle wildcards at the current level
        if topic == "#" {
            self.multi_level_topic_subscribers_id.remove(topic_subscriber_id.as_ref());
            // Also remove from all subtopics since # was propagated
            for (_, child) in self.sub_topics.iter_mut() {
                child.unsubscribe_recursive("#", topic_subscriber_id.as_ref());
            }
            return;
        }

        if topic == "+" {
            self.single_level_topic_subscribers_id.remove(topic_subscriber_id.as_ref());
            return;
        }

        // Check if this is a multi-level wildcard subscription like "sensor/#"
        if splitted_topic.len() > 1 && splitted_topic[1] == "#" {
            if let Some(node) = self.sub_topics.get_mut(topic) {
                node.multi_level_topic_subscribers_id.remove(topic_subscriber_id.as_ref());
                // Also remove from all child subtopics
                for (_, child) in node.sub_topics.iter_mut() {
                    child.unsubscribe_recursive("#", topic_subscriber_id.as_ref());
                }
            }
            return;
        }

        // Check if this is a single-level wildcard subscription like "sensor/+"
        if splitted_topic.len() > 1 && splitted_topic[1] == "+" {
            if let Some(node) = self.sub_topics.get_mut(topic) {
                node.single_level_topic_subscribers_id.remove(topic_subscriber_id.as_ref());
                // Also remove from all direct children since + was propagated
                for (_, child) in node.sub_topics.iter_mut() {
                    child.topic_subscribers_id.remove(topic_subscriber_id.as_ref());
                }
            }
            return;
        }

        // For deeper paths, recursively unsubscribe
        if splitted_topic.len() > 1 {
            let remaining_parts: Vec<&str> = splitted_topic[1..].to_vec();
            let remaining = remaining_parts.join("/");

            if let Some(node) = self.sub_topics.get_mut(topic) {
                node.unsubscribe(remaining, topic_subscriber_id);
            }
        } else {
            // This is the final level
            if let Some(node) = self.sub_topics.get_mut(topic) {
                node.topic_subscribers_id.remove(topic_subscriber_id.as_ref());
            }
        }
    }

    // Helper to recursively remove a subscriber
    fn unsubscribe_recursive(&mut self, wildcard: &str, subscriber_id: &str) {
        if wildcard == "#" {
            self.multi_level_topic_subscribers_id.remove(subscriber_id);
            for (_, child) in self.sub_topics.iter_mut() {
                child.unsubscribe_recursive("#", subscriber_id);
            }
        } else if wildcard == "+" {
            self.single_level_topic_subscribers_id.remove(subscriber_id);
        }
    }

    pub fn get_subscribers<S: AsRef<str>>(&self, topic_str: S) -> Vec<String> {
        self.get_subscribers_internal(topic_str).unwrap_or_default()
    }

    fn get_subscribers_internal<S: AsRef<str>>(&self, topic_str: S) -> Option<Vec<String>> {
        if topic_str.as_ref().is_empty() {
            match self.topic_subscribers_id.len() + self.multi_level_topic_subscribers_id.len() {
                0 => None,
                _ => {
                    let mut all_subscriber = self.topic_subscribers_id.clone();
                    all_subscriber.extend(self.multi_level_topic_subscribers_id.clone());
                    Some(all_subscriber.into_iter().collect())
                }
            }
        } else {
            let splitted_topic: Vec<&str> = topic_str.as_ref().splitn(2, '/').collect();

            // Collect subscribers from multi-level wildcard at this level
            let mut result: Vec<String> = Vec::new();
            result.extend(self.multi_level_topic_subscribers_id.iter().cloned());

            // Continue traversing the tree
            if splitted_topic.len() > 1 {
                if let Some(child_subs) = self
                    .sub_topics
                    .get(splitted_topic[0])
                    .and_then(|tree| tree.get_subscribers_internal(splitted_topic[1]))
                {
                    result.extend(child_subs);
                }

                // Also check if this node has single-level wildcard subscribers
                // This handles patterns like "sensor/+" matching "sensor/temp/..."
                if let Some(parent_node) = self.sub_topics.get(splitted_topic[0]) {
                    result.extend(parent_node.single_level_topic_subscribers_id.iter().cloned());
                }

                // Also check single-level wildcard as a subtopic
                if let Some(wildcard_tree) = self.sub_topics.get("+") {
                    if let Some(wild_subs) = wildcard_tree.get_subscribers_internal(splitted_topic[1]) {
                        result.extend(wild_subs);
                    }
                }
            } else {
                // Last level of topic
                if let Some(tree) = self.sub_topics.get(splitted_topic[0]) {
                    if let Some(exact_subs) = tree.get_subscribers_internal("") {
                        result.extend(exact_subs);
                    }
                    // Also include single-level wildcard subscribers from the matched node
                    result.extend(tree.single_level_topic_subscribers_id.iter().cloned());
                }

                // Also check single-level wildcard as a subtopic
                if let Some(wildcard_tree) = self.sub_topics.get("+") {
                    if let Some(wild_subs) = wildcard_tree.get_subscribers_internal("") {
                        result.extend(wild_subs);
                    }
                }
            }

            if result.is_empty() {
                None
            } else {
                // Remove duplicates
                result.sort();
                result.dedup();
                Some(result)
            }
        }
    }

    // Deprecated - kept for compatibility
    #[allow(dead_code)]
    fn get_subscribers_id<S: AsRef<str>>(&mut self, topic_str: S) -> Option<Vec<String>> {
        if topic_str.as_ref().is_empty() {
            match self.topic_subscribers_id.len() + self.multi_level_topic_subscribers_id.len() {
                0 => None,
                _ => {
                    let mut all_subscriber = self.topic_subscribers_id.clone();
                    all_subscriber.extend(self.multi_level_topic_subscribers_id.clone());
                    Some(all_subscriber.into_iter().collect())
                }
            }
        } else {
            let splitted_topic: Vec<&str> = topic_str.as_ref().splitn(2, '/').collect();
            if splitted_topic.len() > 1 {
                return self.sub_topics.get_mut(splitted_topic[0]).unwrap().get_subscribers_id(splitted_topic[1]);
            } else {
                return match self.sub_topics.entry(String::from(splitted_topic[0])) {
                    Entry::Occupied(o) => o.into_mut().get_subscribers_id(""),
                    Entry::Vacant(_) => match self.multi_level_topic_subscribers_id.len() {
                        0 => None,
                        _ => Some(self.multi_level_topic_subscribers_id.clone().into_iter().collect()),
                    },
                };
            }
        }
    }
}

#[cfg(test)]
#[path = "topic/tests.rs"]
mod topic_tests;

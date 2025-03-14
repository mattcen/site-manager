use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use p2panda_core::{Hash, PublicKey};
use p2panda_net::TopicId;
use p2panda_sync::log_sync::TopicLogMap;
use p2panda_sync::TopicQuery;
use rocket::tokio;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Every site_manaer peer writes to one single log per topic which is identified by the node's public
/// key and the topic id.
pub type LogId = [u8; 32];

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ChatTopic(String, [u8; 32]);

impl ChatTopic {
    pub fn new(name: &str) -> Self {
        Self(name.to_owned(), *Hash::new(name).as_bytes())
    }
}

impl TopicQuery for ChatTopic {}

impl TopicId for ChatTopic {
    fn id(&self) -> [u8; 32] {
        self.1
    }
}

#[derive(Clone, Debug)]
pub struct AuthorStore(Arc<RwLock<HashMap<ChatTopic, HashSet<PublicKey>>>>);

impl AuthorStore {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    // pub async fn add_author(&mut self, topic: ChatTopic, public_key: PublicKey) {
    //     let mut authors = self.0.write().await;
    //     authors
    //         .entry(topic)
    //         .and_modify(|public_keys| {
    //             public_keys.insert(public_key);
    //         })
    //         .or_insert({
    //             let mut public_keys = HashSet::new();
    //             public_keys.insert(public_key);
    //             public_keys
    //         });
    // }

    pub async fn authors(&self, topic: &ChatTopic) -> Option<HashSet<PublicKey>> {
        let authors = self.0.read().await;
        authors.get(topic).cloned()
    }
}

#[async_trait]
impl TopicLogMap<ChatTopic, LogId> for AuthorStore {
    /// During sync other peers are interested in all our append-only logs for a certain topic.
    /// This method tells the sync protocol which logs we have available from which author for that
    /// given topic.
    async fn get(&self, topic: &ChatTopic) -> Option<HashMap<PublicKey, Vec<LogId>>> {
        let authors = self.authors(topic).await;
        let map = match authors {
            Some(authors) => {
                let mut map = HashMap::with_capacity(authors.len());
                for author in authors {
                    // We write all data of one author into one log for now.
                    map.insert(author, vec![topic.id()]);
                }
                map
            }
            None => HashMap::new(),
        };
        Some(map)
    }
}

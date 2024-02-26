// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Clone)]
pub struct Post {
    id: u128,
    title: String,
    description: String,
    tags: Vec<String>,
    media: String,
    users_who_liked: Vec<AccountId>,
    owner_id: AccountId,
}

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct SocialNetworking {
    posts: UnorderedMap<u128, Post>,
    number_of_posts: u128,
    likes_by_user_id: UnorderedMap<AccountId, Vec<Post>>,
    posts_by_tag: UnorderedMap<String, Vec<Post>>,
}

impl Default for SocialNetworking {
    fn default() -> Self {
        Self {
            posts: UnorderedMap::new(b'm'),
            number_of_posts: 0,
            likes_by_user_id: UnorderedMap::new(b'n'),
            posts_by_tag: UnorderedMap::new(b'o'),
        }
    }
}

#[near_bindgen]
impl SocialNetworking {
    pub fn add_post(
        &mut self,
        title: String,
        description: String,
        tags: String,
        media: String,
    ) -> Post {
        let tags_iterator = tags.split(",");
        let mut tags = Vec::<String>::new();
        for tag in tags_iterator {
            tags.push(tag.to_string());
        }

        let post = Post {
            id: self.number_of_posts,
            title,
            description,
            tags: tags.clone(),
            media,
            users_who_liked: Vec::<AccountId>::new(),
            owner_id: env::signer_account_id(),
        };

        self.number_of_posts += 1;
        self.posts.insert(&post.id, &post);

        self.add_posts_by_tag(post.clone(), tags);
        post
    }

    #[private]
    fn add_posts_by_tag(&mut self, post: Post, tags: Vec<String>) {
        let mut posts_for_tag: Vec<Post>;

        for tag in tags {
            if let None = self.posts_by_tag.get(&tag) {
                posts_for_tag = Vec::<Post>::new();
            } else {
                posts_for_tag = self
                    .posts_by_tag
                    .get(&tag)
                    .unwrap_or_else(|| env::panic_str("NO_POSTS_FOUND"));
            }

            posts_for_tag.push(post.clone());
            self.posts_by_tag.insert(&tag, &posts_for_tag);
        }
    }

    pub fn get_all_posts(&self) -> Vec<(u128, Post)> {
        self.posts.to_vec()
    }

    pub fn like_a_post(&mut self, post_id: u128) -> Post {
        let post = self.posts.get(&post_id);

        if let None = post {
            return Post {
                id: post_id,
                title: "No post found at that ID".to_string(),
                description: "No post found at that ID".to_string(),
                tags: Vec::<String>::new(),
                media: "No post found at that ID".to_string(),
                users_who_liked: Vec::<AccountId>::new(),
                owner_id: env::signer_account_id(),
            };
        }

        // Copy and update post
        let mut post_copy = post.unwrap_or_else(|| env::panic_str("POST_NOT_FOUND"));

        // Update the post copy
        post_copy.users_who_liked.push(env::signer_account_id());

        // Update the posts state
        self.posts.insert(&post_id, &post_copy.clone());

        self.add_post_to_my_liked(env::signer_account_id(), &post_copy);

        post_copy
    }

    #[private]
    pub fn add_post_to_my_liked(&mut self, sender_id: AccountId, post: &Post) {
        // Find the users liked posts
        let users_likes = self.likes_by_user_id.get(&sender_id);

        // Add post to users likes
        if let None = users_likes {
            // Create users likes
            self.likes_by_user_id
                .insert(&sender_id, &vec![post.clone()]);
        } else {
            // Update users likes
            let mut checked_users_likes =
                users_likes.unwrap_or_else(|| env::panic_str("UNABLE_TO_FIND_USERS_LIKES"));

            checked_users_likes.push(post.clone());

            self.likes_by_user_id
                .insert(&sender_id, &checked_users_likes);
        }
    }

    pub fn get_liked_posts(&self) -> Vec<Post> {
        self.likes_by_user_id
            .get(&env::signer_account_id())
            .unwrap_or_else(|| env::panic_str("UNABLE_TO_FIND_USERS_LIKED_POSTS"))
    }

    pub fn get_posts_by_tag(&self, tag: String) -> Vec<Post> {
        self.posts_by_tag
            .get(&tag)
            .unwrap_or_else(|| env::panic_str("UNABLE_TO_FIND_POSTS"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_post() {
        let mut contract = SocialNetworking::default();

        contract.add_post(
            "Test".to_string(),
            "Test Descritpion".to_string(),
            "tag1,tag2,tag3".to_string(),
            "post".to_string(),
        );

        let new_post = contract.posts.get(&0).expect("Issue getting post in test");

        assert_eq!(contract.number_of_posts, 1);
        assert_eq!(new_post.title, "Test".to_string());
        assert_eq!(new_post.tags, vec!["tag1", "tag2", "tag3"]);

        assert_eq!(
            contract
                .posts_by_tag
                .get(&"tag1".to_string())
                .expect("Error finding posts by tag in test")
                .get(0)
                .expect("Error getting first post in test")
                .title,
            "Test".to_string()
        )
    }

    #[test]
    fn get_all_posts() {
        let mut contract = SocialNetworking::default();

        contract.add_post(
            "Test".to_string(),
            "Test Descritpion".to_string(),
            "tag1,tag2,tag3".to_string(),
            "post".to_string(),
        );
        contract.add_post(
            "Test2".to_string(),
            "Test Descritpion2".to_string(),
            "tag4,tag5,tag6".to_string(),
            "video".to_string(),
        );
        contract.add_post(
            "Test3".to_string(),
            "Test Descritpion3".to_string(),
            "tag1,tag5,tag7".to_string(),
            "pic".to_string(),
        );

        let all_posts = contract.get_all_posts();

        assert_eq!(all_posts.len(), 3);
        assert_eq!(
            all_posts
                .get(2)
                .expect("Trouble getting all posts in test")
                .1
                .title,
            "Test3"
        );
    }

    #[test]
    fn like_a_post() {
        let mut contract = SocialNetworking::default();

        contract.add_post(
            "Test".to_string(),
            "Test Descritpion".to_string(),
            "tag1,tag2,tag3".to_string(),
            "post".to_string(),
        );

        contract.like_a_post(0);

        let liked_post = contract.posts.get(&0).expect("Post not liked");

        assert_eq!(liked_post.users_who_liked.len(), 1);
        assert_eq!(
            liked_post
                .users_who_liked
                .get(0)
                .expect("Error finding uses who liked the post"),
            &env::signer_account_id()
        );
    }

    #[test]
    fn get_liked_posts() {
        let mut contract = SocialNetworking::default();

        contract.add_post(
            "Test".to_string(),
            "Test Descritpion".to_string(),
            "tag1,tag2,tag3".to_string(),
            "post".to_string(),
        );
        contract.add_post(
            "Test2".to_string(),
            "Test Descritpion2".to_string(),
            "tag4,tag5,tag6".to_string(),
            "video".to_string(),
        );
        contract.add_post(
            "Test3".to_string(),
            "Test Descritpion3".to_string(),
            "tag1,tag5,tag7".to_string(),
            "pic".to_string(),
        );

        contract.like_a_post(0);
        contract.like_a_post(1);

        assert_eq!(
            contract
                .get_liked_posts()
                .get(0)
                .unwrap_or_else(|| env::panic_str("ERROR FINDING LIKED POSTS"))
                .title,
            "Test".to_string()
        );
        assert_eq!(contract.get_liked_posts().len(), 2);
    }

    #[test]
    fn get_posts_by_tag() {
        let mut contract = SocialNetworking::default();

        contract.add_post(
            "Test".to_string(),
            "Test Descritpion".to_string(),
            "tag1,tag2,tag3".to_string(),
            "post".to_string(),
        );
        contract.add_post(
            "Test2".to_string(),
            "Test Descritpion2".to_string(),
            "tag4,tag5,tag6".to_string(),
            "video".to_string(),
        );
        contract.add_post(
            "Test3".to_string(),
            "Test Descritpion3".to_string(),
            "tag1,tag5,tag7".to_string(),
            "pic".to_string(),
        );

        let posts = contract.get_posts_by_tag("tag5".to_string());

        assert_eq!(posts.len(), 2);
        assert_eq!(posts.get(0).unwrap().title, "Test2".to_string());
        assert_eq!(posts.get(1).unwrap().title, "Test3".to_string());
    }
}

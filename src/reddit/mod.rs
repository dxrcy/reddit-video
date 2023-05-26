mod json;

use self::json::{post, subreddit};
use crate::{config, ToTextFrames};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; WOW64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.5666.197 Safari/537.36";

#[derive(Debug)]
pub struct Post {
    pub title: String,
    pub body: String,
    pub link: String,
}

impl ToTextFrames for Post {
    fn to_text_frames(self) -> Vec<String> {
        vec![self.title, self.body]
    }
}

#[derive(Debug)]
pub struct Comment {
    pub body: String,
}

impl ToTextFrames for Comment {
    fn to_text_frames(self) -> Vec<String> {
        vec![self.body]
    }
}

fn build_client() -> reqwest::blocking::Client {
    reqwest::blocking::ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build()
        .expect("Error building simple reqwest client")
}

pub fn fetch_posts(config: &config::Reddit) -> Result<Vec<Post>, reqwest::Error> {
    let config::Reddit {
        subreddit,
        sort,
        time,
        limit,
        ..
    } = config;

    let url = format!("https://reddit.com/r/{subreddit}/{sort}.json?t={time}&count={limit}");

    let client = build_client();

    let text = client.get(&url).send()?.text()?;

    let subreddit: subreddit::Response =
        serde_json::from_str(&text).expect("Failed to parse json!");

    let mut posts = Vec::new();
    for child in subreddit.data.children {
        let subreddit::ChildData {
            title,
            selftext,
            permalink,
        } = child.data;

        posts.push(Post {
            title,
            body: selftext,
            link: permalink,
        });
    }

    Ok(posts)
}

pub fn fetch_comments(
    config: &config::Reddit,
    parent_link: &str,
) -> Result<Vec<Comment>, reqwest::Error> {
    let config::Reddit { limit, .. } = config;

    let url = format!("https://reddit.com/{parent_link}.json?limit={limit}");

    let client = build_client();

    let text = client.get(&url).send()?.text()?;

    let post: post::Response = serde_json::from_str(&text).expect("Failed to parse json!");

    let mut comments = Vec::new();
    for child in post.1.data.children {
        let body = child.data.body;

        let Some(body) = body else {
            println!("  [info] comment missing body");
            continue;
        };

        comments.push(Comment { body });
    }

    Ok(comments)
}

pub fn sort_and_time(config: &config::Reddit) -> String {
    let config::Reddit { time, sort, .. } = config;
    if sort == "top" {
        if time == "all" {
            format!("{sort} of all time")
        } else {
            format!("{sort} of the {time}")
        }
    } else {
        sort.to_string()
    }
}
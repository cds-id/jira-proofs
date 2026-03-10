use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::JiraConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssue {
    pub key: String,
    pub summary: String,
}

pub fn build_auth_header(email: &str, api_token: &str) -> String {
    let credentials = format!("{}:{}", email, api_token);
    format!("Basic {}", BASE64.encode(credentials.as_bytes()))
}

pub fn build_adf_image_comment(description: &str, image_url: &str) -> Value {
    json!({
        "version": 1, "type": "doc",
        "content": [
            {"type": "paragraph", "content": [{"type": "text", "text": description}]},
            {"type": "mediaSingle", "attrs": {"layout": "center"},
             "content": [{"type": "media", "attrs": {"type": "external", "url": image_url}}]}
        ]
    })
}

pub fn build_adf_link_comment(description: &str, link_url: &str) -> Value {
    json!({
        "version": 1, "type": "doc",
        "content": [
            {"type": "paragraph", "content": [{"type": "text", "text": description}]},
            {"type": "paragraph", "content": [
                {"type": "text", "text": link_url, "marks": [{"type": "link", "attrs": {"href": link_url}}]}
            ]}
        ]
    })
}

pub fn build_adf_comment(preset_title: &str, description: &str, url: &str, is_image: bool) -> Value {
    let mut content = vec![
        json!({"type": "heading", "attrs": {"level": 3}, "content": [{"type": "text", "text": preset_title}]}),
        json!({"type": "paragraph", "content": [{"type": "text", "text": description}]}),
    ];
    if is_image {
        content.push(json!({"type": "mediaSingle", "attrs": {"layout": "center"},
            "content": [{"type": "media", "attrs": {"type": "external", "url": url}}]}));
    } else {
        content.push(json!({"type": "paragraph", "content": [
            {"type": "text", "text": url, "marks": [{"type": "link", "attrs": {"href": url}}]}
        ]}));
    }
    json!({"version": 1, "type": "doc", "content": content})
}

pub async fn search_issues(config: &JiraConfig, query: &str) -> Result<Vec<JiraIssue>, String> {
    let client = Client::new();
    let auth = build_auth_header(&config.email, &config.api_token);
    let jql = if query.is_empty() {
        format!("project = {} AND status != Done ORDER BY updated DESC", config.default_project)
    } else {
        format!("project = {} AND summary ~ \"{}\" AND status != Done ORDER BY updated DESC", config.default_project, query)
    };
    let url = format!("{}/rest/api/3/search", config.base_url);
    let response = client.get(&url).header("Authorization", &auth).header("Accept", "application/json")
        .query(&[("jql", &jql), ("maxResults", &"20".to_string()), ("fields", &"summary".to_string())])
        .send().await.map_err(|e| format!("Jira search failed: {}", e))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Jira API error {}: {}", status, body));
    }
    let body: Value = response.json().await.map_err(|e| format!("Failed to parse: {}", e))?;
    let issues = body["issues"].as_array().unwrap_or(&vec![]).iter()
        .filter_map(|issue| {
            let key = issue["key"].as_str()?.to_string();
            let summary = issue["fields"]["summary"].as_str()?.to_string();
            Some(JiraIssue { key, summary })
        }).collect();
    Ok(issues)
}

pub async fn post_comment(config: &JiraConfig, issue_key: &str, preset_title: &str, description: &str, url: &str, is_image: bool) -> Result<(), String> {
    let client = Client::new();
    let auth = build_auth_header(&config.email, &config.api_token);
    let adf = build_adf_comment(preset_title, description, url, is_image);
    let api_url = format!("{}/rest/api/3/issue/{}/comment", config.base_url, issue_key);
    let response = client.post(&api_url).header("Authorization", &auth)
        .header("Content-Type", "application/json")
        .json(&json!({"body": adf}))
        .send().await.map_err(|e| format!("Failed to post comment: {}", e))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Jira comment failed {}: {}", status, body));
    }
    Ok(())
}

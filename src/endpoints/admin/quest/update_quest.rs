use crate::middleware::auth::auth_middleware;
use crate::models::{QuestDocument, Banner};
use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;

use mongodb::options::FindOneAndUpdateOptions;

use mongodb::bson::{doc, Bson, Document};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

pub_struct!(Deserialize; UpdateQuestQuery {
    id: i32,
    name: Option<String>,
    desc: Option<String>,
    start_time: Option<i64>,
    expiry: Option<i64>,
    disabled: Option<bool>,
    mandatory_domain: Option<String>,
    category: Option<String>,
    logo: Option<String>,
    rewards_img: Option<String>,
    rewards_title: Option<String>,
    img_card: Option<String>,
    title_card: Option<String>,
    issuer: Option<String>,
    banner: Option<Banner>,
});

#[route(post, "/admin/quest/update", auth_middleware)]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Extension(sub): Extension<String>,
    Json(body): Json<UpdateQuestQuery>,
) -> impl IntoResponse {
    let collection = state.db.collection::<QuestDocument>("quests");

    // filter to get existing quest
    let mut filter = doc! {
        "id": &body.id,
    };

    // check if user is super_user
    if sub != "super_user" {
        filter.insert("issuer", sub);
    }

    let existing_quest = &collection.find_one(filter.clone(), None).await.unwrap();
    if existing_quest.is_none() {
        return get_error("quest does not exist".to_string());
    }

    let mut update_doc = Document::new();
    if let Some(name) = &body.name {
        update_doc.insert("name", name);
    }
    if let Some(desc) = &body.desc {
        update_doc.insert("desc", desc);
    }
    if let Some(expiry) = &body.expiry {
        update_doc.insert("expiry", expiry);
    }
    if let Some(start_time) = &body.start_time {
        update_doc.insert("start_time", start_time);
    }
    if let Some(disabled) = &body.disabled {
        update_doc.insert("disabled", disabled);
    }
    update_doc.insert(
        "mandatory_domain",
        body.mandatory_domain
            .clone()
            .map_or(Bson::Null, |v| Bson::String(v)),
    );

    if let Some(category) = &body.category {
        update_doc.insert("category", category);
    }
    if let Some(logo) = &body.logo {
        update_doc.insert("logo", logo);
    }
    if let Some(logo) = &body.issuer {
        update_doc.insert("issuer", logo);
    }
    if let Some(rewards_img) = &body.rewards_img {
        update_doc.insert("rewards_img", rewards_img);
        let nft_reward = doc! {
            "img": &body.rewards_img.clone(),
            "level": 1,
        };
        update_doc.insert("rewards_nfts", vec![nft_reward]);
    }
    if let Some(rewards_title) = &body.rewards_title {
        update_doc.insert("rewards_title", rewards_title);
    }
    if let Some(img_card) = &body.img_card {
        update_doc.insert("img_card", img_card);
    }
    if let Some(title_card) = &body.title_card {
        update_doc.insert("title_card", title_card);
    }
    if let Some(banner) = &body.banner {
        update_doc.insert("banner", to_bson(&banner).unwrap());
    }

    // update quest query
    let update = doc! {
        "$set": update_doc.clone()
    };

    // Perform quest update
    let quest_update_result = collection
        .find_one_and_update(filter.clone(), update, None)
        .await;

    let nft_uri_collection = state.db.collection::<Document>("nft_uri");
    let nft_uri_filter = doc! { "id": &body.id };

    let mut nft_update_doc = Document::new();
    if let Some(rewards_img) = &body.rewards_img {
        nft_update_doc.insert("image", rewards_img);
    }
    if let Some(rewards_title) = &body.rewards_title {
        nft_update_doc.insert("name", rewards_title);
    }
    if let Some(desc) = &body.desc {
        nft_update_doc.insert("description", desc);
    }

    if !nft_update_doc.is_empty() {
        let nft_update = doc! { "$set": nft_update_doc };
        let nft_uri_update_result = nft_uri_collection
            .find_one_and_update(
                nft_uri_filter,
                nft_update,
                FindOneAndUpdateOptions::default(),
            )
            .await;

        return match (quest_update_result, nft_uri_update_result) {
            (Ok(_), Ok(_)) => (
                StatusCode::OK,
                Json(json!({"message": "updated successfully"})),
            )
                .into_response(),
            _ => get_error("error updating quest or nft_uri".to_string()),
        };
    }

    return match quest_update_result {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "updated successfully"})),
        )
            .into_response(),
        Err(_e) => get_error("error updating quest".to_string()),
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::bson::Bson;

    #[test]
    fn test_banner_serialization() {
        let banner = Banner {
            tag: "Test Tag".to_string(),
            title: "Test Title".to_string(),
            description: "Test Description".to_string(),
            cta: "Test CTA".to_string(),
            href: "https://test.com".to_string(),
            image: "https://test.com/image.png".to_string(),
        };
        
        let serialized = to_bson(&banner).unwrap();
        
        // Verify it serializes to a BSON document
        assert!(matches!(serialized, Bson::Document(_)));
        
        if let Bson::Document(doc) = serialized {
            assert_eq!(doc.get_str("tag").unwrap(), "Test Tag");
            assert_eq!(doc.get_str("title").unwrap(), "Test Title");
            assert_eq!(doc.get_str("description").unwrap(), "Test Description");
            assert_eq!(doc.get_str("cta").unwrap(), "Test CTA");
            assert_eq!(doc.get_str("href").unwrap(), "https://test.com");
            assert_eq!(doc.get_str("image").unwrap(), "https://test.com/image.png");
        }
    }
}

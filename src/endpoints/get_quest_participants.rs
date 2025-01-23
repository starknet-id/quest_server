use crate::{models::AppState, utils::get_error};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};

use axum::http::StatusCode;
use axum_auto_routes::route;
use futures::StreamExt;
use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]

pub struct GetQuestParticipantsQuery {
    quest_id: u32,
}

#[route(get, "/get_quest_participants")]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GetQuestParticipantsQuery>,
) -> impl IntoResponse {
    // Convert to int
    let quest_id = query.quest_id.to_string().parse::<i64>().unwrap();
    let tasks_collection = state.db.collection::<Document>("tasks");
    let tasks_ids = tasks_collection
        .find(doc! { "quest_id": quest_id }, None)
        .await
        .unwrap()
        .map(|task_doc| {
            task_doc
                .unwrap()
                .get("id")
                .unwrap()
                .to_string()
                .parse::<i64>()
                .unwrap()
        })
        .collect::<Vec<i64>>()
        .await;

    let tasks_count = tasks_ids.len();

    let pipeline = vec![
        doc! {
            "$match": {
                "task_id": {
                    "$in": tasks_ids
                }
            }
        },
        doc! {
            "$group": {
                "_id": "$address",
                "count" : { "$sum": 1 }
            }
        },
        doc! {
            "$match": {
                "count": tasks_count as i64
            }
        },
        doc! {
            "$facet": {
                "count": [
                    {
                        "$count": "count"
                    }
                ],
                "firstParticipants": [
                    {
                        "$limit": 3
                    }
                ]
            }
        },
        doc! {
            "$project": {
                "count": {
                    "$arrayElemAt": [
                        "$count.count",
                        0
                    ]
                },
                "firstParticipants": "$firstParticipants._id"
            }
        },
    ];

    let completed_tasks_collection = state.db.collection::<Document>("completed_tasks");
    let mut cursor = completed_tasks_collection
        .aggregate(pipeline, None)
        .await
        .unwrap();

    let mut res: Document = Document::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                res = document;
            }
            Err(_) => return get_error("Error querying quest participants".to_string()),
        }
    }

    return (StatusCode::OK, Json(res)).into_response();
}
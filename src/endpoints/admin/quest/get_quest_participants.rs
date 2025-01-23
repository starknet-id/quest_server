use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_auto_routes::route;
use futures::StreamExt;
use mongodb::bson::doc;
use serde::Deserialize;
use serde_json::json;
use starknet::core::types::FieldElement;

use crate::{middleware::auth::auth_middleware, utils::to_hex};
use crate::{
    models::{AppState, CompletedTaskDocument, QuestTaskDocument},
    utils::get_error,
};

pub_struct!(Deserialize; GetQuestParticipantsParams {
    quest_id: i64,
});

#[route(get, "/admin/quests/get_quest_participants", auth_middleware)]
pub async fn get_quest_participants_handler(
    State(state): State<Arc<AppState>>,
    Extension(_sub): Extension<String>,
    Query(params): Query<GetQuestParticipantsParams>,
) -> impl IntoResponse {
    let tasks_collection = state.db.collection::<QuestTaskDocument>("tasks");
    let completed_tasks_collection = state
        .db
        .collection::<CompletedTaskDocument>("completed_tasks");

    // Fetch all task IDs for the given quest_id
    let task_filter = doc! { "quest_id": params.quest_id };
    let task_ids: Vec<i32> = match tasks_collection.find(task_filter, None).await {
        Ok(mut cursor) => {
            let mut ids = Vec::new();
            while let Some(doc) = cursor.next().await {
                match doc {
                    Ok(task) => ids.push(task.id),
                    Err(e) => return get_error(format!("Error processing tasks: {}", e)),
                }
            }
            ids
        }
        Err(e) => return get_error(format!("Error fetching tasks: {}", e)),
    };

    if task_ids.is_empty() {
        return get_error(format!("No tasks found for quest_id {}", params.quest_id));
    }

    // Use aggregation pipeline to fetch completed tasks and group by address
    let pipeline = vec![
        doc! { "$match": { "task_id": { "$in": &task_ids } } },
        doc! { "$group": {
            "_id": "$address",
            "task_ids": { "$addToSet": "$task_id" },
            "max_timestamp": { "$max": "$timestamp.$numberLong" }
        }},
        doc! { "$project": {
            "address": "$_id",
            "tasks_completed_count": { "$size": "$task_ids" },
            "quest_completion_timestamp": "$max_timestamp"
        }},
    ];

    let mut cursor = match completed_tasks_collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(e) => return get_error(format!("Error aggregating completed tasks: {}", e)),
    };

    let total_tasks = task_ids.len();
    let mut participants = Vec::new();

    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(doc) => {
                // Get the decimal address and convert it to a hex string
                let address: String = match doc.get_str("address") {
                    Ok(addr) => to_hex(FieldElement::from_dec_str(addr).unwrap()),
                    Err(_) => continue, // Skip invalid documents
                };

                let tasks_completed_count: usize = match doc.get_i32("tasks_completed_count") {
                    Ok(count) => count as usize,
                    Err(_) => continue, // Skip invalid documents
                };

                if tasks_completed_count == total_tasks {
                    participants.push(address);
                }
            }
            Err(e) => return get_error(format!("Error processing aggregation results: {}", e)),
        }
    }

    let participants_json = json!({ "participants": participants });
    (StatusCode::OK, Json(participants_json)).into_response()
}

// #[cfg(test)]
// mod tests {
//     use crate::{
//         config::{self, Config},
//         logger,
//     };

//     use super::*;
//     use axum::body::HttpBody;
//     use axum::{http::StatusCode};
//     use mongodb::{bson::{doc, Document}, Client, Database};
//     use reqwest::Url;
//     use serde_json::Value;
//     use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient};
//     use std::sync::Arc;
//     use tokio::sync::Mutex;

//     async fn setup_test_db() -> Database {
//         let client = Client::with_uri_str("mongodb://localhost:27017")
//             .await
//             .expect("Failed to create MongoDB client");
//         let db = client.database("test_db");

//         // Clear collections before each test
//         db.collection::<Document>("tasks").drop(None).await.ok();
//         db.collection::<Document>("completed_tasks")
//             .drop(None)
//             .await
//             .ok();

//         db
//     }

//     async fn insert_test_data(db: Database, quest_id: i64, num_tasks: i64, num_participants: i64) {
//         let tasks_collection = db.collection::<Document>("tasks");
//         let completed_tasks_collection = db.collection::<Document>("completed_tasks");

//         // Insert tasks
//         for task_id in 1..=num_tasks {
//             tasks_collection
//                 .insert_one(
//                     doc! {
//                         "id": task_id,
//                         "quest_id": quest_id,
//                     },
//                     None,
//                 )
//                 .await
//                 .unwrap();
//         }

//         // Insert completed tasks for participants
//         // Each participant will have a different timestamp for each task
//         // timestamp will be 1000 - (participant * 10) + task_id
//         // This way, the last task for each participant will have the highest timestamp
//         // and the last participant will be the one who completed the quest first

//         // 2..=num_participants: skip the first participant
//         // The first participant haven't completed all the tasks
//         for participant in 1..=num_participants {
//             let address = format!("participant_{}", participant);
//             let base_timestamp = 1000 - (participant * 10); 

//             // First participant only do one task => not completed the quest yet
//             if participant == 1 {
//                 completed_tasks_collection.insert_one(
//                     doc! {
//                         "address": address.clone(),
//                         "task_id": 1,
//                         "timestamp": base_timestamp + 1
//                     },
//                     None,
//                 ).await.unwrap();
//             } else {
//                 for task_id in 1..=num_tasks {
//                     completed_tasks_collection
//                         .insert_one(
//                             doc! {
//                                 "address": address.clone(),
//                                 "task_id": task_id,
//                                 // Last task for each participant will have the highest timestamp
//                                 "timestamp": base_timestamp + task_id
//                             },
//                             None,
//                         )
//                         .await
//                         .unwrap();
//                 }
//             }
//         }
//     }

//     #[tokio::test]
//     async fn test_get_quest_participants() {
//         // Setup
//         let db = setup_test_db().await;
//         let conf = config::load();
//         let logger = logger::Logger::new(&conf.watchtower);
//         let provider = JsonRpcClient::new(HttpTransport::new(
//             Url::parse(&conf.variables.rpc_url).unwrap(),
//         ));

//         let app_state = Arc::new(AppState {
//             db: db.clone(),
//             last_task_id: Mutex::new(0),
//             last_question_id: Mutex::new(0),
//             conf,
//             logger,
//             provider,
//         });
//         let extension = "".to_string();

//         // Test data
//         let quest_id = 1;
//         let num_tasks = 3;
//         let num_participants = 5;

//         insert_test_data(db.clone(), quest_id, num_tasks, num_participants).await;

//         // Create request
//         let query = GetQuestParticipantsParams {
//             quest_id: quest_id as i64,
//         };

//         // Execute request
//         let response = get_quest_participants_handler(State(app_state), Extension(extension), Query(query))
//             .await
//             .into_response();

//         // Verify response
//         assert_eq!(response.status(), StatusCode::OK);

//         // Get the response body as bytes
//         let body_bytes = match response.into_body().data().await {
//             Some(Ok(bytes)) => bytes,
//             _ => panic!("Failed to get response body"),
//         };

//         // Parse the body
//         let body: Value = serde_json::from_slice(&body_bytes).unwrap();

//         // We has excluded the first participant from the test data
//         assert_eq!(body["total"], num_participants);

//         // Verify first participants
//         let first_participants = body["first_participants"].as_array().unwrap();

//         // Verify quest completion timestamp
//         let quest_completion_timestamp = body["first_participants"][1]["quest_completion_time"]
//             .as_i64()
//             .unwrap();
//         assert_eq!(quest_completion_timestamp, 953);

//         // Verify participant not completed the quest
//         let participant_not_completed = first_participants.iter().find(|participant| {
//             participant["address"].as_str().unwrap() == "participant_1"
//         }).unwrap();

//         assert_eq!(participant_not_completed["tasks_completed"].as_i64().unwrap(), 1);
//         assert_eq!(participant_not_completed["quest_completion_time"].as_i64(), None); // Not completed

//     }
// }


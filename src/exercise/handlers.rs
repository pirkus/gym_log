use bson::doc;
use mongodb::{Client, Collection};
use nvo_servers::http::{response::Response, AsyncRequest};
use serde_json::json;

use super::Exercise;

pub async fn get_excs(req: AsyncRequest) -> Result<Response, String> {
    let excs_coll: Collection<Exercise> = req.deps.get::<Client>().unwrap().database("gym-log").collection("exercises");
    let excs_id = req.path_params.get("id").unwrap();
    let obj_id = bson::oid::ObjectId::parse_str(excs_id).unwrap();
    match excs_coll.find_one(Some(doc! {"_id": obj_id}), None).await {
        Ok(excs) => match excs {
            Some(excs) => Ok(Response::create(200, serde_json::to_string(&excs).unwrap())),
            None => Ok(Response::create(404, json!({"err": "excs not found"}).to_string())),
        },
        Err(e) => {
            let e = e.to_string();
            Ok(Response::create(500, json!({"err": e}).to_string()))
        }
    }
}

pub async fn post_excs(req: AsyncRequest) -> Result<Response, String> {
    let excs_coll: Collection<Exercise> = req.deps.get::<Client>().unwrap().database("gym-log").collection("exercises");
    let body = req.body().await.unwrap();
    let excs: Exercise = serde_json::from_str(&body).unwrap();
    let id = excs_coll.insert_one(&excs, None).await.unwrap().inserted_id.as_object_id().unwrap().to_hex();
    Ok(Response::create(200, json!({"_id": id}).to_string()))
}

fn main() {
    excs_example::main();
}

pub mod excs_example {
    use std::{collections::HashSet, env};

    use bson::doc;
    use bson::serde_helpers::serialize_hex_string_as_object_id;
    use mongodb::{Client, Collection};
    use nvo_servers::{
        futures::workers::Workers,
        http::{
            async_handler::AsyncHandler,
            async_http_server::{AsyncHttpServer, AsyncHttpServerTrt},
            response::Response,
            AsyncRequest,
        },
    };
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Value};

    pub fn main() {
        env_logger::init();

        let async_worker = Workers::new(1);
        let mongo_client = async_worker
            .queue_with_result(async {
                let uri = env::var("MONGO_URI").unwrap();
                Client::with_uri_str(uri).await.unwrap()
            })
            .unwrap()
            .get();

        //async fn mongo(_: AsyncRequest) -> Result<Response, String> {}

        let handlers = HashSet::from([
            AsyncHandler::new("GET", "/status", status),
            AsyncHandler::new("GET", "/excs/:id", get_excs),
            AsyncHandler::new("GET", "/excs/:id", post_excs),
        ]);

        AsyncHttpServer::builder()
            .with_port(8090)
            .with_handlers(handlers)
            .with_dep(mongo_client)
            .build()
            .start_blocking();
    }

    #[derive(Serialize, Deserialize)]
    struct Exercise {
        #[serde(serialize_with = "serialize_hex_string_as_object_id")]
        _id: String,
        name: String,
        desc: String,
    }

    async fn status(_: AsyncRequest) -> Result<Response, String> {
        Ok(Response::create(200, "{\"status\": \"ok\"}".to_string()))
    }

    async fn get_excs(req: AsyncRequest) -> Result<Response, String> {
        let mongo = req.deps.get::<Client>().unwrap();
        let excs_coll: Collection<Exercise> = mongo.database("gym-log").collection("exercises");
        let excs_id = req.path_params.get("id").unwrap();
        match excs_coll.find_one(doc! { "_id": excs_id }, None).await {
            Ok(excs) => match excs {
                Some(excs) => {
                    let name = excs.name;
                    Ok(Response::create(200, json!({"name": name}).to_string()))
                }
                None => Ok(Response::create(
                    404,
                    json!({"err": "excs not found"}).to_string(),
                )),
            },
            Err(e) => {
                let e = e.to_string();
                Ok(Response::create(500, json!({"err": e}).to_string()))
            }
        }
    }

    async fn post_excs(req: AsyncRequest) -> Result<Response, String> {
        let mongo = req.deps.get::<Client>().unwrap();
        let excs_coll: Collection<Exercise> = mongo.database("gym-log").collection("exercises");
        let buf = req.body().await;
        let doc: Value = serde_json::from_str(&buf).unwrap();
        let excs = Exercise {
            _id: bson::oid::ObjectId::new().to_hex(),
            name: doc.get("name").unwrap().to_string(),
            desc: doc.get("desc").unwrap().to_string(),
        };

        excs_coll.insert_one(&excs, None).await.unwrap();
        Ok(Response::create(200, json!({"name": buf}).to_string()))
    }
}

pub mod exercise;

fn main() {
    excs_example::main("<mongo_conn_string>");
}

pub mod excs_example {
    use std::collections::HashSet;

    use env_logger::Env;
    use mongodb::Client;
    use nvo_servers::{
        futures::workers::Workers,
        http::{
            async_handler::AsyncHandler,
            async_http_server::{AsyncHttpServer, AsyncHttpServerTrt},
            response::Response,
            AsyncRequest,
        },
    };

    use crate::exercise;

    pub fn main(mongo_url: &str) {
        env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

        let async_worker = Workers::new(1);
        let mongo_url_clj = mongo_url.to_string();
        let mongo_client = async_worker.queue_with_result(async { Client::with_uri_str(mongo_url_clj).await.unwrap() }).unwrap().get();

        let handlers = HashSet::from([
            AsyncHandler::new("GET", "/status", status),
            AsyncHandler::new("GET", "/excs/:id", exercise::handlers::get_excs),
            AsyncHandler::new("POST", "/excs", exercise::handlers::post_excs),
        ]);

        AsyncHttpServer::builder().with_port(8090).with_handlers(handlers).with_dep(mongo_client).build().start_blocking();
    }

    async fn status(_: AsyncRequest) -> Result<Response, String> {
        Ok(Response::create(200, "{\"status\": \"ok\"}".to_string()))
    }
}

#[cfg(test)]
mod acceptance_tests {
    use std::{thread, time::Duration};

    use log::debug;
    use serde_json::Value;
    use testcontainers::{
        core::{ContainerPort, WaitFor},
        runners::SyncRunner,
        Container, GenericImage, ImageExt,
    };

    use crate::{excs_example, exercise::Exercise};

    #[test]
    fn can_create_and_retrieve_exercise() {
        let mongo_container = create_mongo_container();
        let mongo_host = mongo_container.get_host().unwrap();
        let mongo_port = mongo_container.get_host_port_ipv4(ContainerPort::Tcp(27017)).unwrap();
        let mongo_uri = format!("mongodb://root:root@{mongo_host}:{mongo_port}");

        let _server_thread = thread::spawn(move || excs_example::main(&mongo_uri));
        let http_client = reqwest::blocking::Client::builder().build().unwrap();

        let http_port = "8090";
        wait_for_server_to_start(http_port);

        let excs = Exercise {
            _id: bson::oid::ObjectId::new().to_hex(),
            name: "a-name".to_string(),
            desc: "a-desc".to_string(),
        };
        let body = serde_json::to_string(&excs).unwrap();
        let body_len = body.len();
        let res = http_client
            .post(format!("http://localhost:{http_port}/excs").as_str())
            .body(body)
            .header("Content-Length", body_len)
            .send()
            .unwrap();
        assert_eq!(res.status(), 200);

        let excs_id: Value = serde_json::from_str(res.text().unwrap().as_str()).unwrap();
        let excs_id = excs_id.get("_id").unwrap().as_str().unwrap();
        let res = http_client.get(format!("http://localhost:{http_port}/excs/{excs_id}").as_str()).send().unwrap();
        assert_eq!(res.status(), 200);
        let excs: Exercise = serde_json::from_str(&res.text().unwrap()).unwrap();
        assert_eq!(excs.name, "a-name");
    }

    fn wait_for_server_to_start(port: &str) {
        loop {
            thread::sleep(Duration::from_millis(10));
            match reqwest::blocking::get(format!("http://localhost:{port}/status").as_str()) {
                Ok(resp) => {
                    if let Ok(body) = resp.text() {
                        assert!(body.contains("ok"), "server not started ok: body = {}", body);
                        break;
                    }
                }
                Err(e) => debug!("Error: {e}"),
            };
        }
    }

    fn create_mongo_container() -> Container<GenericImage> {
        GenericImage::new("mongo", "6.0.7")
            .with_wait_for(WaitFor::message_on_stdout("server is ready"))
            .with_exposed_port(ContainerPort::Tcp(27017))
            .with_env_var("MONGO_INITDB_DATABASE", "gym-log")
            .with_env_var("MONGO_INITDB_ROOT_USERNAME", "root")
            .with_env_var("MONGO_INITDB_ROOT_PASSWORD", "root")
            .start()
            .unwrap()
    }
}

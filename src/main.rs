use lambda_http::{run, service_fn, Body, Error, Request, Response}; // Quitamos RequestPayloadExt
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
struct Nota {
    id: Option<String>,
    titulo: String,
    contenido: String,
}

async fn function_handler(event: Request, client: Client) -> Result<Response<Body>, Error> {
    tracing::info!(">>> HANDLER ALCANZADO. Método: {}", event.method());

    let method = event.method();
    let table_name = "NotasPrueba"; 

    match *method {
        lambda_http::http::Method::POST => {
            // --- PARSEO MANUAL (A PRUEBA DE BALAS) ---
            let body_bytes = event.body().as_ref(); // Obtenemos los bytes crudos
            let parse_result: Result<Nota, _> = serde_json::from_slice(body_bytes);

            match parse_result {
                Ok(nota) => {
                    tracing::info!("✅ JSON recibido correctamente: {:?}", nota);
                    let id_nuevo = uuid::Uuid::new_v4().to_string();
                    
                    let result = client.put_item()
                        .table_name(table_name)
                        .item("id", AttributeValue::S(id_nuevo.clone()))
                        .item("titulo", AttributeValue::S(nota.titulo))
                        .item("contenido", AttributeValue::S(nota.contenido))
                        .send()
                        .await;

                    match result {
                        Ok(_) => {
                            let resp = json!({ "mensaje": "Nota guardada", "id": id_nuevo });
                            Ok(Response::builder()
                                .status(200)
                                .header("content-type", "application/json")
                                .body(Body::from(resp.to_string()))
                                .unwrap())
                        },
                        Err(e) => {
                            tracing::error!("❌ Error AWS: {:?}", e);
                            Ok(Response::builder().status(500).body(format!("Error DB: {:?}", e).into()).unwrap())
                        }
                    }
                },
                Err(e) => {
                    tracing::error!("❌ Error al leer el JSON: {:?}", e);
                    Ok(Response::builder().status(400).body("JSON Inválido".into()).unwrap())
                }
            }
        },
        lambda_http::http::Method::GET => {
            let result = client.scan().table_name(table_name).send().await;
             match result {
                Ok(res) => {
                    let notas: Vec<Nota> = res.items().iter().map(|item| {
                        Nota {
                            id: item.get("id").and_then(|v| v.as_s().ok()).cloned(),
                            titulo: item.get("titulo").and_then(|v| v.as_s().ok()).cloned().unwrap_or_default(),
                            contenido: item.get("contenido").and_then(|v| v.as_s().ok()).cloned().unwrap_or_default(),
                        }
                    }).collect();
                    Ok(Response::builder().status(200).body(Body::from(serde_json::to_string(&notas)?)).unwrap())
                },
                Err(e) => Ok(Response::builder().status(500).body(format!("Error Scan: {:?}", e).into()).unwrap())
            }
        },
        _ => Ok(Response::builder().status(405).body("Method not allowed".into()).unwrap())
    }
}

#[tokio::main] // This macro is causing the error. Let's try to remove it and handle the async main manually.
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let client = Client::new(&config);

    tracing::info!(">>> SERVIDOR LISTO (Modo Manual Parse)");

    run(service_fn(move |event: Request| {
        let client = client.clone();
        async move {
            function_handler(event, client).await
        }
    })).await
}
use lambda_http::{run, service_fn, Body, Error, Request, Response, RequestExt}; 
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use serde::{Deserialize, Serialize};
use serde_json::json;

// --- ESTRUCTURAS ---
#[derive(Serialize, Deserialize, Debug)]
struct Nota {
    id: Option<String>,
    titulo: String,
    contenido: String,
}

// --- CONSTANTES ---
const TABLE_NAME: &str = "NotasPrueba";

// --- AYUDAS (HELPERS) ---

// Función para responder con CORS habilitado (Vital para que el frontend funcione)
fn respuesta_cors(status: u16, body: String) -> Result<Response<Body>, Error> {
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        // En producción podrías cambiar "*" por "https://notasrust.testingpage.store"
        .header("Access-Control-Allow-Origin", "*") 
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS") 
        .header("Access-Control-Allow-Headers", "content-type") 
        .body(Body::from(body))
        .map_err(|e| Box::new(e) as Error)
}

// --- HANDLERS POR MÉTODO ---

async fn handle_post(req: Request, client: &Client) -> Result<Response<Body>, Error> {
    let body_bytes = req.body().as_ref();
    let parse_result: Result<Nota, _> = serde_json::from_slice(body_bytes);

    match parse_result {
        Ok(nota) => {
            let id_nuevo = uuid::Uuid::new_v4().to_string();
            tracing::info!("📝 Creando nota: {}", nota.titulo);

            let result = client.put_item()
                .table_name(TABLE_NAME)
                .item("id", AttributeValue::S(id_nuevo.clone()))
                .item("titulo", AttributeValue::S(nota.titulo))
                .item("contenido", AttributeValue::S(nota.contenido))
                .send()
                .await;

            match result {
                Ok(_) => {
                    let resp = json!({ "mensaje": "Creado", "id": id_nuevo });
                    respuesta_cors(200, resp.to_string())
                },
                Err(e) => {
                    tracing::error!("Error AWS: {:?}", e);
                    respuesta_cors(500, json!({"error": "Fallo en DB"}).to_string())
                }
            }
        },
        Err(_) => respuesta_cors(400, json!({"error": "JSON inválido"}).to_string())
    }
}

async fn handle_get(client: &Client) -> Result<Response<Body>, Error> {
    match client.scan().table_name(TABLE_NAME).send().await {
        Ok(res) => {
            let notas: Vec<Nota> = res.items().iter().map(|item| {
                Nota {
                    id: item.get("id").and_then(|v| v.as_s().ok()).cloned(),
                    titulo: item.get("titulo").and_then(|v| v.as_s().ok()).cloned().unwrap_or_default(),
                    contenido: item.get("contenido").and_then(|v| v.as_s().ok()).cloned().unwrap_or_default(),
                }
            }).collect();
            respuesta_cors(200, serde_json::to_string(&notas)?)
        },
        Err(e) => {
            tracing::error!("Error AWS: {:?}", e);
            respuesta_cors(500, json!({"error": "Error al leer"}).to_string())
        }
    }
}

async fn handle_delete(req: Request, client: &Client) -> Result<Response<Body>, Error> {
    // Buscamos el ID en los Query Parameters: ?id=xyz
    let query_params = req.query_string_parameters();
    let id_a_borrar = query_params.first("id");

    if let Some(id) = id_a_borrar {
        tracing::info!("🗑️ Borrando nota ID: {}", id);
        
        match client.delete_item()
            .table_name(TABLE_NAME)
            .key("id", AttributeValue::S(id.to_string()))
            .send()
            .await 
        {
            Ok(_) => respuesta_cors(200, json!({"mensaje": "Nota eliminada"}).to_string()),
            Err(e) => {
                tracing::error!("Error al borrar: {:?}", e);
                respuesta_cors(500, json!({"error": "No se pudo borrar"}).to_string())
            }
        }
    } else {
        respuesta_cors(400, json!({"error": "Falta el parametro ?id="}).to_string())
    }
}

async fn handle_put(req: Request, client: &Client) -> Result<Response<Body>, Error> {
    // PUT actualiza (sobrescribe) una nota existente si enviamos el mismo ID
    let body_bytes = req.body().as_ref();
    let parse_result: Result<Nota, _> = serde_json::from_slice(body_bytes);

    match parse_result {
        Ok(nota) => {
            // Validamos que venga el ID para saber qué editar
            if let Some(id_existente) = &nota.id {
                tracing::info!("✏️ Editando nota ID: {}", id_existente);

                let result = client.put_item()
                    .table_name(TABLE_NAME)
                    .item("id", AttributeValue::S(id_existente.clone()))
                    .item("titulo", AttributeValue::S(nota.titulo))
                    .item("contenido", AttributeValue::S(nota.contenido))
                    .send()
                    .await;

                match result {
                    Ok(_) => respuesta_cors(200, json!({"mensaje": "Nota actualizada"}).to_string()),
                    Err(e) => {
                        tracing::error!("Error AWS: {:?}", e);
                        respuesta_cors(500, json!({"error": "Fallo update"}).to_string())
                    }
                }
            } else {
                respuesta_cors(400, json!({"error": "Para editar necesitas enviar el campo 'id'"}).to_string())
            }
        },
        Err(_) => respuesta_cors(400, json!({"error": "JSON inválido"}).to_string())
    }
}

// --- MAIN CONTROLLER ---

async fn function_handler(event: Request, client: Client) -> Result<Response<Body>, Error> {
    let method = event.method();
    tracing::info!(">>> Petición: {}", method);

    match *method {
        lambda_http::http::Method::GET => handle_get(&client).await,
        lambda_http::http::Method::POST => handle_post(event, &client).await,
        lambda_http::http::Method::DELETE => handle_delete(event, &client).await,
        lambda_http::http::Method::PUT => handle_put(event, &client).await,
        
        // OPTIONS es CRÍTICO para CORS. El navegador pregunta "puedo pasar?" antes de enviar datos.
        lambda_http::http::Method::OPTIONS => {
            respuesta_cors(200, "".to_string())
        },
        
        _ => respuesta_cors(405, json!({"error": "Método no permitido"}).to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let client = Client::new(&config);

    tracing::info!(">>> SERVER LISTO: CRUD COMPLETO + CORS");

    run(service_fn(move |event: Request| {
        let client = client.clone();
        async move {
            function_handler(event, client).await
        }
    })).await
}
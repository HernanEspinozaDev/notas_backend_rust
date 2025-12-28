# 🦀 Notas Serverless Backend (Rust + AWS Lambda)

Este proyecto es una API Serverless construida con **Rust**, diseñada para desplegarse en **AWS Lambda** y persistir datos en **Amazon DynamoDB**.

Fue desarrollado como una prueba de concepto para demostrar el rendimiento y la seguridad de tipos de Rust en entornos serverless.

## 🚀 Arquitectura

* **Runtime:** Rust (usando `cargo-lambda`).
* **Framework HTTP:** `lambda_http` (v0.13 - compatible con AWS SDK v1).
* **Base de Datos:** AWS DynamoDB.
* **Serialización:** `serde` & `serde_json` (Parseo manual robusto).
* **Logging:** `tracing` estructurado.

## 🛠️ Requisitos Previos

* [Rust](https://www.rust-lang.org/tools/install) instalado.
* [Cargo Lambda](https://www.cargo-lambda.info/) (`cargo install cargo-lambda`).
* [AWS CLI](https://aws.amazon.com/cli/) configurado con credenciales válidas.

## ⚙️ Configuración Local

1.  **Clonar el repositorio:**
    ```bash
    git clone https://github.com/HernanEspinozaDev/notas_backend_rust.git
    cd notas_backend_rust
    ```

2.  **Crear la tabla en DynamoDB (si no existe):**
    ```bash
    aws dynamodb create-table \
        --table-name NotasPrueba \
        --attribute-definitions AttributeName=id,AttributeType=S \
        --key-schema AttributeName=id,KeyType=HASH \
        --provisioned-throughput ReadCapacityUnits=1,WriteCapacityUnits=1 \
        --region us-east-1
    ```

## ▶️ Ejecución en Desarrollo

Para levantar el servidor local simulando Lambda:

```powershell
$Env:AWS_REGION="us-east-1"; cargo lambda watch
```

El servidor escuchará en `http://localhost:9000`.

## 📡 Uso de la API

### 1. Crear una Nota (POST)

```powershell
Invoke-RestMethod -Uri "http://localhost:9000/" `
  -Method POST `
  -ContentType "application/json" `
  -Body '{"titulo": "Aprender Rust", "contenido": "Rust en Lambda es veloz"}'
```

### 2. Leer todas las Notas (GET)

```powershell
Invoke-RestMethod -Uri "http://localhost:9000/" -Method GET
```

## 📦 Estructura del Proyecto

- **src/main.rs:** Lógica principal. Contiene el handler asíncrono y el parseo manual del JSON para evitar conflictos de versiones http.
- **Cargo.toml:** Gestión de dependencias (versiones pinneadas para compatibilidad).

---

Desarrollado con ❤️ y mucho debugging.

[GitHub](https://github.com/HernanEspinozaDev)
# HTTP API Reference

PolarisDB provides a REST API via the `polarisdb-server` crate.

## Running the Server

```bash
# From source
cargo run -p polarisdb-server

# Docker
docker run -p 8080:8080 hugoev/polarisdb
```

## Endpoints

### Collections

#### Create Collection

```http
POST /collections
Content-Type: application/json

{
  "name": "my_collection",
  "dimension": 384,
  "metric": "cosine"
}
```

#### List Collections

```http
GET /collections
```

#### Delete Collection

```http
DELETE /collections/{name}
```

### Vectors

#### Insert Vector

```http
POST /collections/{name}/vectors
Content-Type: application/json

{
  "id": 1,
  "vector": [0.1, 0.2, ...],
  "payload": {"category": "docs"}
}
```

#### Insert Batch

```http
POST /collections/{name}/vectors/batch
Content-Type: application/json

{
  "vectors": [
    {"id": 1, "vector": [0.1, ...], "payload": {}},
    {"id": 2, "vector": [0.2, ...], "payload": {}}
  ]
}
```

### Search

#### Search

```http
POST /collections/{name}/search
Content-Type: application/json

{
  "vector": [0.1, 0.2, ...],
  "k": 10,
  "filter": {
    "field": "category",
    "op": "eq",
    "value": "docs"
  }
}
```

**Response:**

```json
{
  "results": [
    {"id": 1, "distance": 0.05, "payload": {"category": "docs"}},
    {"id": 3, "distance": 0.12, "payload": {"category": "docs"}}
  ]
}
```

## Health Check

```http
GET /health
```

Returns `200 OK` if server is healthy.

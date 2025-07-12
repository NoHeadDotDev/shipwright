# API Reference

This document provides comprehensive documentation for the amazing-app REST API.

## Base URL

```
http://localhost:3000/api
```

## Authentication

The API uses Bearer token authentication. Include the token in the Authorization header:

```http
Authorization: Bearer <your-jwt-token>
```

## Health Check Endpoints

### Basic Health Check

**GET** `/health`

Returns basic health status of the service.

**Response:**
```json
{
  "status": "ok",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### Detailed Health Check

**GET** `/health/detailed`

Returns comprehensive health information including database status.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T12:00:00Z",
  "checks": {
    "database": "healthy",
    "config": "healthy"
  },
  "info": {
    "service": "amazing-app",
    "environment": "Development",
    "version": "0.1.0"
  }
}
```

### Readiness Check

**GET** `/health/ready`

Kubernetes readiness probe endpoint.

**Response:**
- `200 OK` - Service is ready to accept traffic
- `503 Service Unavailable` - Service is not ready

### Liveness Check

**GET** `/health/live`

Kubernetes liveness probe endpoint.

**Response:**
```json
{
  "status": "alive",
  "timestamp": "2024-01-01T12:00:00Z"
}
```
## User Management

### List Users

**GET** `/users`

Retrieve a paginated list of users with optional filtering and sorting.

**Query Parameters:**
- `page` (integer, optional): Page number (default: 1)
- `per_page` (integer, optional): Items per page (default: 20, max: 100)
- `search` (string, optional): Search term for username or email
- `sort` (string, optional): Sort field (username, email, created_at)
- `order` (string, optional): Sort order (asc, desc)

**Example Request:**
```http
GET /users?page=1&per_page=10&search=john&sort=created_at&order=desc
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "username": "johndoe",
      "email": "john@example.com",
      "created_at": "2024-01-01T12:00:00Z",
      "updated_at": "2024-01-01T12:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 10,
    "total": 1,
    "total_pages": 1,
    "has_next": false,
    "has_prev": false
  }
}
```

### Get User by ID

**GET** `/users/{id}`

Retrieve a specific user by their ID.

**Path Parameters:**
- `id` (UUID): User ID

**Example Request:**
```http
GET /users/123e4567-e89b-12d3-a456-426614174000
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "username": "johndoe",
    "email": "john@example.com",
    "created_at": "2024-01-01T12:00:00Z",
    "updated_at": "2024-01-01T12:00:00Z"
  }
}
```

**Error Responses:**
- `404 Not Found` - User not found

### Create User

**POST** `/users`

Create a new user account.

**Request Body:**
```json
{
  "username": "johndoe",
  "email": "john@example.com"
}
```

**Validation Rules:**
- `username`: Required, 3-50 characters, unique
- `email`: Required, valid email format, unique

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "username": "johndoe",
    "email": "john@example.com",
    "created_at": "2024-01-01T12:00:00Z",
    "updated_at": "2024-01-01T12:00:00Z"
  }
}
```

**Error Responses:**
- `400 Bad Request` - Invalid input data
- `409 Conflict` - Username or email already exists

### Update User

**PUT** `/users/{id}`

Update an existing user's information.

**Path Parameters:**
- `id` (UUID): User ID

**Request Body:**
```json
{
  "username": "newusername",
  "email": "newemail@example.com"
}
```

**Note:** All fields are optional. Only provided fields will be updated.

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "username": "newusername",
    "email": "newemail@example.com",
    "created_at": "2024-01-01T12:00:00Z",
    "updated_at": "2024-01-01T13:00:00Z"
  }
}
```

**Error Responses:**
- `400 Bad Request` - Invalid input data
- `404 Not Found` - User not found
- `409 Conflict` - Username or email already exists

### Delete User

**DELETE** `/users/{id}`

Delete a user account.

**Path Parameters:**
- `id` (UUID): User ID

**Response:**
- `204 No Content` - User deleted successfully
- `404 Not Found` - User not found

**Example Request:**
```http
DELETE /users/123e4567-e89b-12d3-a456-426614174000
```

## Error Handling

The API uses standard HTTP status codes and returns error details in JSON format.

### Error Response Format

```json
{
  "error": "Error type",
  "message": "Human-readable error description"
}
```

### Common Status Codes

| Status Code | Description |
|-------------|-------------|
| 200 | OK - Request successful |
| 201 | Created - Resource created successfully |
| 204 | No Content - Request successful, no content returned |
| 400 | Bad Request - Invalid request data |
| 401 | Unauthorized - Authentication required |
| 403 | Forbidden - Insufficient permissions |
| 404 | Not Found - Resource not found |
| 409 | Conflict - Resource already exists |
| 422 | Unprocessable Entity - Validation error |
| 500 | Internal Server Error - Server error |
| 503 | Service Unavailable - Service temporarily unavailable |

### Validation Errors

When validation fails, the API returns detailed error information:

```json
{
  "error": "Validation error",
  "message": "Invalid input data",
  "details": {
    "username": ["Username is required"],
    "email": ["Email must be a valid email address"]
  }
}
```

## Rate Limiting

The API implements rate limiting to prevent abuse:

- **Global Rate Limit**: 1000 requests per hour per IP
- **Authenticated Rate Limit**: 5000 requests per hour per user
- **Create Operations**: 100 requests per hour per IP

Rate limit headers are included in responses:

```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1640995200
```

## Pagination

List endpoints support pagination with the following query parameters:

- `page`: Page number (starts from 1)
- `per_page`: Number of items per page (max 100)

Pagination information is included in the response:

```json
{
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 100,
    "total_pages": 5,
    "has_next": true,
    "has_prev": false
  }
}
```

## Filtering and Sorting

### Search

Use the `search` query parameter to search across relevant fields:

```http
GET /users?search=john
```

### Sorting

Use `sort` and `order` parameters to control result ordering:

```http
GET /users?sort=created_at&order=desc
```

Available sort fields vary by endpoint and are documented in each endpoint's description.

## Request/Response Examples

### cURL Examples

#### Create a user
```bash
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{
    "username": "johndoe",
    "email": "john@example.com"
  }'
```

#### Get user with authentication
```bash
curl -X GET http://localhost:3000/users/123e4567-e89b-12d3-a456-426614174000 \
  -H "Authorization: Bearer your-jwt-token"
```

#### Update a user
```bash
curl -X PUT http://localhost:3000/users/123e4567-e89b-12d3-a456-426614174000 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-jwt-token" \
  -d '{
    "username": "newusername"
  }'
```

### JavaScript Examples

#### Using fetch API
```javascript
// Create user
const response = await fetch('http://localhost:3000/users', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    username: 'johndoe',
    email: 'john@example.com'
  })
});

const user = await response.json();
console.log(user);

// Get user with authentication
const userResponse = await fetch(`http://localhost:3000/users/${userId}`, {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});

const userData = await userResponse.json();
```

## SDKs and Client Libraries

While amazing-app doesn't provide official SDKs, the API is compatible with standard HTTP clients and OpenAPI tools.

### OpenAPI Specification

The API follows OpenAPI 3.0 specification. You can generate client libraries using tools like:

- [OpenAPI Generator](https://openapi-generator.tech/)
- [Swagger Codegen](https://swagger.io/tools/swagger-codegen/)

## WebSocket API (LiveView)
amazing-app also provides real-time functionality through WebSocket connections used by LiveView components.

### Connection

```javascript
const ws = new WebSocket('ws://localhost:3000/live/websocket');
```

### Message Format

Messages follow the LiveView protocol for real-time UI updates. This is typically handled automatically by the LiveView JavaScript client.

### Events

LiveView WebSocket connections handle:
- Component state updates
- DOM patching
- Event forwarding
- Hot reload (development only)

## Development and Testing

### Testing the API

Use tools like:
- **Postman**: GUI-based API testing
- **Insomnia**: Alternative to Postman
- **HTTPie**: Command-line HTTP client
- **curl**: Universal command-line tool

### API Documentation Tools

- **Swagger UI**: Interactive API documentation
- **Redoc**: Alternative documentation renderer
- **Postman Collections**: Shareable API collections

### Mock Data

In development mode, the API includes seeded test data. See the [Getting Started Guide](./getting-started.md) for information about running with test data.

## Security Considerations

1. **Always use HTTPS** in production
2. **Validate all input** data
3. **Implement proper authentication** and authorization
4. **Use rate limiting** to prevent abuse
5. **Log security events** for monitoring
6. **Keep dependencies updated** for security patches

## Support

For API support:
- Check the [troubleshooting guide](./troubleshooting.md)
- Review the [development documentation](./development.md)
- Submit issues on the project repository
# API Documentation

Base URL: `http://localhost:8080/api`

All endpoints return JSON responses.

## Authentication

### Register

**POST** `/auth/register`

Create a new user account.

**Note:** Registration can be disabled via the `ALLOW_REGISTRATION` environment variable. When disabled, this endpoint will return `403 Forbidden`.

**Request Body:**
```json
{
  "username": "string",
  "password": "string",
  "email": "string (optional)"
}
```

**Response:** `201 Created`
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "john_doe"
}
```

**Error Responses:**
- `400 Bad Request` - Username already exists
- `403 Forbidden` - Registration is disabled
- `500 Internal Server Error` - Server error

---

### Login

**POST** `/auth/login`

Login with existing credentials.

**Request Body:**
```json
{
  "username": "string",
  "password": "string"
}
```

**Response:** `200 OK`
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "john_doe"
}
```

**Error Responses:**
- `401 Unauthorized` - Invalid credentials
- `500 Internal Server Error` - Server error

---

## Protected Endpoints

Protected endpoints require a JWT token in the Authorization header:

```
Authorization: Bearer <token>
```

*(Additional endpoints will be documented as they are implemented)*

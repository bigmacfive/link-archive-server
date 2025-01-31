# Link Archive Service

A Rust-based service that allows users to save, preview, and automatically summarize web links using ChatGPT.

## Features

- User authentication (register/login)
- Link archiving and management
- Link preview generation
- Automatic HTML content extraction
- Content summarization using ChatGPT API
- RESTful API endpoints

## Tech Stack

- **Backend**: Rust with Actix-web framework
- **Database**: PostgreSQL
- **Authentication**: JWT (JSON Web Tokens)
- **External APIs**: OpenAI ChatGPT API
- **HTML Parsing**: reqwest + scraper crates

## API Endpoints

### Authentication

#### Register User
```
POST /api/auth/register
Content-Type: application/json

Request:
{
    "username": string,
    "email": string,
    "password": string
}

Response:
{
    "id": string,
    "username": string,
    "email": string,
    "token": string
}
```

#### Login
```
POST /api/auth/login
Content-Type: application/json

Request:
{
    "email": string,
    "password": string
}

Response:
{
    "token": string,
    "user": {
        "id": string,
        "username": string,
        "email": string
    }
}
```

### Links

#### Save Link
```
POST /api/links
Authorization: Bearer <token>
Content-Type: application/json

Request:
{
    "url": string,
    "tags": string[] (optional)
}

Response:
{
    "id": string,
    "url": string,
    "title": string,
    "preview": string,
    "summary": string,
    "tags": string[],
    "created_at": string
}
```

#### Get All Links
```
GET /api/links
Authorization: Bearer <token>

Response:
{
    "links": [
        {
            "id": string,
            "url": string,
            "title": string,
            "preview": string,
            "summary": string,
            "tags": string[],
            "created_at": string
        }
    ]
}
```

#### Get Link by ID
```
GET /api/links/{id}
Authorization: Bearer <token>

Response:
{
    "id": string,
    "url": string,
    "title": string,
    "preview": string,
    "summary": string,
    "tags": string[],
    "created_at": string
}
```

#### Update Link
```
PUT /api/links/{id}
Authorization: Bearer <token>
Content-Type: application/json

Request:
{
    "tags": string[]
}

Response:
{
    "id": string,
    "url": string,
    "title": string,
    "preview": string,
    "summary": string,
    "tags": string[],
    "updated_at": string
}
```

#### Delete Link
```
DELETE /api/links/{id}
Authorization: Bearer <token>

Response:
{
    "message": "Link successfully deleted"
}
```

## Database Schema

### Users Table
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

### Links Table
```sql
CREATE TABLE links (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    url TEXT NOT NULL,
    title TEXT,
    preview TEXT,
    summary TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

### Tags Table
```sql
CREATE TABLE tags (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    UNIQUE(name)
);
```

### Link Tags Table
```sql
CREATE TABLE link_tags (
    link_id UUID REFERENCES links(id),
    tag_id UUID REFERENCES tags(id),
    PRIMARY KEY (link_id, tag_id)
);
```

## Implementation Details

### Link Processing Flow
1. User submits a URL
2. System validates URL format
3. Fetch HTML content using reqwest
4. Extract title and relevant content using scraper
5. Generate preview (first few paragraphs or meta description)
6. Send content to ChatGPT API for summarization
7. Store all information in database
8. Return processed link data to user

### Security Considerations
- Password hashing using argon2
- JWT token expiration and refresh mechanism
- Rate limiting for API endpoints
- Input validation and sanitization
- CORS configuration
- Environment variables for sensitive data

### Error Handling
- Proper HTTP status codes
- Detailed error messages
- Request validation
- Database error handling
- External API error handling

## Getting Started

1. Clone the repository
2. Set up PostgreSQL database
3. Configure environment variables
4. Install Rust and cargo
5. Run migrations
6. Start the server

## Environment Variables
```
DATABASE_URL=postgresql://user:password@localhost/dbname
JWT_SECRET=your_jwt_secret
OPENAI_API_KEY=your_openai_api_key
SERVER_PORT=8080
```
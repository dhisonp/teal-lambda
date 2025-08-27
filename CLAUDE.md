# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Project Overview

`teal-lambda` is a Rust-based AWS Lambda function that provides an HTTP API for
a therapeutic conversation service. The application uses AI (Gemini) to provide
benevolent responses to user "tells" (personal shares) and stores conversation
data in DynamoDB.

## Key Development Commands

### Building

- **Production build**: `cargo lambda build --release`
- **Development build**: `cargo lambda build`

### Testing

- **Unit tests**: `cargo test`
- **Local development server**: `cargo lambda watch` (auto-restarts on code
  changes)
- **Invoke locally**:
  - With example data: `cargo lambda invoke --data-example apigw-request`
  - With custom JSON file: `cargo lambda invoke --data-file ./data.json`
  - Direct HTTP calls: `curl http://localhost:9000`

### Deployment

- **Deploy to AWS**: `cargo lambda deploy`

## Architecture

### Core Modules

- **`main.rs`**: Entry point, initializes database and starts Lambda runtime
- **`http_handler.rs`**: HTTP routing and request/response handling for API
  endpoints
- **`gemini.rs`**: Integration with Google Gemini API for AI responses
- **`dynamo.rs`**: DynamoDB client wrapper with table management
- **`schema.rs`**: Data structures for User and Context
- **`users.rs`**: User management functionality
- **`prompts.rs`**: Template system for AI prompts

### API Endpoints

- `POST /tell`: Main endpoint accepting user stories with `text` body and
  `username` query parameter
- `POST /user/create`: User registration with `name` and `email` in request body

### Database Schema

- **`teal-users` table**: Stores user profiles with tid (UUID), name, email,
  current_mood, created_at
- **`teal-tells` table**: Stores conversation history with tid, username, tell,
  answer, user_state, mood, created_at, summary

### AI Integration

Uses Google Gemini 2.0 Flash model with structured JSON responses containing:

- `answer`: Therapeutic response to user's tell
- `summary`: Concise summary of the user's tell (max 12 words)
- `user_state`: Current state of mind assessment (max 12 words)
- `mood`: Single word mood classification

### Environment Setup

- Requires `GEMINI_API_KEY` environment variable
- Uses `dotenvy` to load `.env` file in development (TODO: disable in
  production)
- AWS credentials configured via standard AWS SDK methods

### Dependencies

- `lambda_http`: AWS Lambda HTTP runtime
- `aws-sdk-dynamodb`: DynamoDB operations
- `reqwest`: HTTP client for Gemini API calls
- `serde`/`serde_json`: JSON serialization
- `uuid`: Unique identifier generation
- `chrono`: Date/time handling

## Development Notes

- Database tables are auto-created on startup if they don't exist
- All database operations use a global singleton DynamoDB client
- Prompts are templated using the `prompts` module with replaceable variables
- Error handling uses `anyhow` for easy error propagation
- Tests are currently outdated and need updating (see TODO comments)

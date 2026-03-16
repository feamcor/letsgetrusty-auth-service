# Let's Get Rusty - Authentication Service

This project is a complete authentication service built with Rust, featuring a two-factor authentication system. It is designed as a microservice and includes both an `auth-service` and a corresponding `app-service` to demonstrate its functionality.

## Project Structure

The repository is organized into the following main components:

- **`auth-service`**: The core authentication microservice. It handles user registration, login, and two-factor authentication.
- **`app-service`**: A web application that uses the `auth-service` to manage user sessions and protect routes.
- **`compose.yml`**: Docker Compose file to orchestrate the services and their dependencies (PostgreSQL and Redis).
- **`.github`**: Contains GitHub Actions workflows for CI/CD.

## Features

### `auth-service`

- **User Management**:
  - User registration with password hashing (Argon2).
  - Secure login and logout.
- **Two-Factor Authentication (2FA)**:
  - Generates and sends 2FA codes via email.
  - Verifies 2FA codes for enhanced security.
- **Token-Based Authentication**:
  - Uses JSON Web Tokens (JWT) for session management.
  - Includes a token blacklist to handle logouts and prevent token reuse.
- **Configuration**:
  - Highly configurable via environment variables and command-line arguments.
  - Supports different backends for storage and caching (in-memory, PostgreSQL, Redis).
- **Email Integration**:
  - Sends emails using Postmark (or a mock service for testing).
- **API Schema**:
  - The API is documented using OpenAPI (see `api_schema.yml`).

### `app-service`

- **Web Interface**:
  - Provides a simple UI for registration, login, and accessing protected content.
  - Built with Axum and Askama for templating.
- **Protected Routes**:
  - Demonstrates how to protect routes and manage user sessions with the `auth-service`.

## Dependencies

### `auth-service`

- **Web Framework**: `axum`
- **Database**: `sqlx` (with PostgreSQL)
- **Caching**: `redis`
- **Authentication**: `argon2`, `jsonwebtoken`
- **Configuration**: `clap`, `dotenvy`
- **Email**: `reqwest` (for Postmark API)
- **Other**: `tokio`, `serde`, `tracing`

### `app-service`

- **Web Framework**: `axum`
- **Templating**: `askama`
- **HTTP Client**: `reqwest`
- **Other**: `tokio`, `serde`, `tracing`

## Getting Started

### Prerequisites

- Docker and Docker Compose
- Rust toolchain

### Running the Application

1. **Clone the repository**:
   ```bash
   git clone https://github.com/FeAmCor/letsgetrusty-auth-service.git
   cd letsgetrusty-auth-service
   ```

2. **Set up environment variables**:
   - Create a `.env` file based on the provided examples in `compose.yml`.
   - You will need to provide credentials for the database and any external services (like Postmark).

3. **Build and run with Docker Compose**:
   ```bash
   docker-compose up --build
   ```

4. **Access the services**:
   - `app-service` will be available at `http://localhost:8000`.
   - `auth-service` will be available at `http://localhost:3000`.

## CI/CD

The project includes a CI/CD pipeline using GitHub Actions, which automates testing, building, and deploying the services.

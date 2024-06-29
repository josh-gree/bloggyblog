# BloggyBlog

BloggyBlog is a simple, rusty blog engine that serves articles and images from GitHub. It's built using Rust and leverages various modern web development libraries to create a fast and efficient blogging platform.

## Features

- Serves articles and images stored on GitHub
- Caching system for improved performance
- Supports Markdown-formatted articles
- Image gallery functionality
- Simple and responsive design

## Technology Stack

- Rust
- Axum (Web framework)
- Askama (Templating engine)
- Octocrab (GitHub API client)
- Tokio (Asynchronous runtime)
- Serde (Serialization/Deserialization)

## Project Structure

- `src/`: Contains the Rust source code
  - `lib.rs`: Main library file
  - `server.rs`: Server implementation
  - `article.rs`: Article-related functionality
  - `image.rs`: Image-related functionality
  - `index.rs`: Index handling
  - `config.rs`: Configuration handling
- `templates/`: HTML templates for rendering pages
- `Cargo.toml`: Rust package manifest
- `Dockerfile`: For containerizing the application

## Getting Started

1. Clone the repository
2. Install Rust and Cargo if you haven't already
3. Run `cargo build` to compile the project
4. Set up the necessary environment variables or configuration file
5. Run `cargo run` to start the server

The server will start on `http://localhost:8000` by default.

## API Endpoints

- `/`: Home page
- `/articles`: List of articles
- `/article/:uuid`: View a specific article
- `/images`: Image gallery
- `/image/:uuid`: View a specific image
- `/image/raw/:uuid`: Raw image data
- `/cache-clear`: Clear the cache

## Configuration

The application uses a configuration file to set up GitHub access and file paths. Make sure to set up the configuration correctly before running the application.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[Add appropriate license information here]

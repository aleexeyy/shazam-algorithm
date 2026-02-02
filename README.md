# 🎵 Shazam Clone – Music Recognition App

This project is an attempt to recreate the core functionality of **Shazam** — recognizing songs based on audio fingerprints.

## 🧠 Tech Stack

- **Recognition Algorithm**: Written in **Rust** to explore the language's performance and ecosystem.
- **Backend**: Built with **Actix-web (Rust)** for handling uploads, fingerprinting, and database interaction.
- **Frontend**: A simple interface built with **React** for uploading tracks and recognizing songs.
- **Database**: **PostgreSQL** stores fingerprints + song metadata.
- **Object Storage**: **MinIO** (S3-compatible) for future audio/object storage needs.
- **Audio Downloading**: Uses `yt-dlp` to fetch audio from YouTube based on Spotify track links.

## 🚀 Features

- **Two Modes**:
  - **Upload Mode**: Add new tracks to the database.
    - Users can paste a **Spotify link**.
    - The server extracts song info (title & artist), downloads the track using **yt-dlp**, and processes it for fingerprinting.
  - **Recognition Mode**: Identify a song from an audio sample.
    - Supports `.wav`, `.mp3`, and `.ogg` file uploads.

- **Switching Modes**: The mode can be toggled with a **switch** on the frontend.

## ⚠️ Limitations

- The recognition algorithm is still a **work in progress**.
- Performance may be poor on:
  - **Short samples**
  - **Low-quality recordings**


## Technology Stack

- **Recognition Algorithm**: Rust
- **Backend**: Actix-web (Rust)
- **Frontend**: React
- **Database**: PostgreSQL
- **Object Storage**: MinIO

## Setup Instructions

### Prerequisites

1. **Rust**
   - Install Rust using rustup: https://rustup.rs/
   - Follow the instructions for your operating system

2. **Docker + Docker Compose** (recommended)
   - Install Docker Desktop or Docker Engine + Compose plugin.

### Quickstart (Docker)

1. Start the full stack (app + PostgreSQL + MinIO):
   - `docker compose up --build`

2. Verify the server:
   - `curl http://localhost:8000/healthz`

### Spotify API Setup

1. Create a Spotify Developer account at [Spotify Developer Dashboard](https://developer.spotify.com/dashboard/)
2. Create a new application
3. Once created, you'll receive a **Client ID** and **Client Secret**
4. Set these as environment variables (see Environment Setup below)

### Project Setup

1. Clone the repository:
   ```
   git clone https://github.com/your-username/shazam-clone.git
   cd shazam-clone
   ```

2. Run the setup script to install dependencies:
   ```
   bash setup.sh
   ```
   
   (This will install yt-dlp, ffmpeg, and other dependencies)

3. Set up environment variables:
   - Copy `.env.example` to `.env` and adjust as needed:
   ```
   SERVER_PORT=8000
   CLIENT_ID=your_spotify_client_id
   CLIENT_SECRET=your_spotify_client_secret
   DATABASE_URL=postgres://shazam:shazam@localhost:5432/shazam
   S3_ENDPOINT=http://localhost:9000
   S3_BUCKET=shazam
   S3_ACCESS_KEY=minio
   S3_SECRET_KEY=minio123456
   ```

4. Build the Rust component:
   ```
   cd recognition
   cargo build --release
   cd ..
   ```

5. Install frontend dependencies:
   ```
   cd frontend
   npm install
   cd ..
   ```

### Running the Application

1. Start the MySQL service if not already running
   - **Windows**: Via Services app
   - **macOS**: `brew services start mysql`
   - **Linux**: `sudo systemctl start mysql`

2. Run the Rust setup script to create database tables:
   ```
   cd recognition
   cargo run --bin setup
   cd ..
   ```

3. Start the backend server:
   ```
   cargo run --bin shazam-server
   ```

4. In a new terminal, start the frontend development server:
   ```
   cd frontend
   npm start
   ```

5. Open your browser and navigate to `http://localhost:3001`

## Performance (Criterion + Flamegraphs)

See `docs/perf.md` for running benchmarks and generating flamegraphs.

## Using the Application

1. Use the toggle switch to select either "Upload" or "Recognize" mode

2. To add songs to the database:
   - Switch to "Upload" mode
   - Paste a Spotify link (e.g., https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT)
   - Click "Submit" and wait for processing

3. To recognize a song:
   - Switch to "Recognize" mode
   - Click "Upload Audio" and select a .wav, .mp3, or .ogg file
   - Wait for the analysis results

## Troubleshooting

- **Database Connection Issues**: 
  - Verify MySQL is running
  - Check your database credentials in the .env file
  - Ensure the database user has proper permissions

- **Audio Processing Errors**:
  - Make sure ffmpeg is correctly installed and accessible in your PATH
  - Check for supported audio formats (.wav, .mp3, .ogg)

- **Song Download Problems**:
  - Verify yt-dlp is installed correctly
  - Check your internet connection
  - Some songs might be unavailable on YouTube or have copyright restrictions

## How It Works

The application works similarly to Shazam:
1. When adding songs, the system creates audio fingerprints from frequency patterns
2. These fingerprints are stored in the database with song information
3. During recognition, the algorithm extracts fingerprints from the sample audio
4. It compares these fingerprints against the database to find the closest match

## 🛠️ Goals

This project was mainly built for **learning purposes**, especially:
- Diving into **Rust** for performance-critical parts.
- Exploring full-stack development across **React**, **Express**, and system-level processing.
- Understanding **audio fingerprinting** and practical database design.


## License

MIT

---

Feel free to explore, modify, or contribute!

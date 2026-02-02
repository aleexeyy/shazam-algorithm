# 🎵 Shazam Clone – Music Recognition App

This project is an attempt to recreate the core functionality of **Shazam** — recognizing songs based on audio fingerprints.

## 🧠 Tech Stack

- **Recognition Algorithm**: Written in **Rust** to explore the language's performance and ecosystem.
- **Backend**: Built with **Actix-web (Rust)** for handling uploads, fingerprinting, and database interaction.
- **Frontend**: A simple interface built with **React** for uploading tracks and recognizing songs.
- **Database**: **PostgreSQL** stores fingerprints + song metadata.
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

## Setup Instructions

### Prerequisites

1. **Docker + Docker Compose** (recommended)
2. Optional local tools (only needed when not using Docker):
   - `ffmpeg`
   - `yt-dlp`

### Quickstart (Docker)

1. Start the stack (app + PostgreSQL):
   - `docker compose up --build -d`

2. Verify the server:
   - `curl http://localhost:8000/healthz`

3. Open the app:
   - `http://localhost:8000/`

### Spotify API Setup (optional; only needed for Upload Mode)

1. Create a Spotify app in the Spotify Developer Dashboard.
2. Put `CLIENT_ID` and `CLIENT_SECRET` into your environment (or `.env`, not committed).

### Local development (no Docker)

1. Backend:
   - `cp .env.example .env`
   - `cargo run --bin shazam-server`

2. Frontend:
   - `cp front-end/.env.example front-end/.env`
   - `npm --prefix front-end install`
   - `npm --prefix front-end run dev`

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
  - Verify PostgreSQL is running (or use Docker Compose)
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
- Exploring full-stack development across **React** and system-level processing.
- Understanding **audio fingerprinting** and practical database design.


## License

MIT

---

Feel free to explore, modify, or contribute!

# 🎵 Shazam Clone – Music Recognition App

This project is an attempt to recreate the core functionality of **Shazam** — recognizing songs based on audio fingerprints.

## 🧠 Tech Stack

- **Recognition Algorithm**: Written in **Rust** to explore the language's performance and ecosystem.
- **Backend**: Built with **Express.js** for handling audio uploads, processing, and database interaction.
- **Frontend**: A simple interface built with **React** for uploading tracks and recognizing songs.
- **Database**: Local **MySQL** database stores audio fingerprints of uploaded songs.
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
- **Backend**: Express.js (Node.js)
- **Frontend**: React
- **Database**: MySQL

## Setup Instructions

### Prerequisites

1. **Rust**
   - Install Rust using rustup: https://rustup.rs/
   - Follow the instructions for your operating system

2. **Node.js and npm**
   - Download and install from: https://nodejs.org/ (LTS version recommended)
   - Verify installation:
     ```
     node --version
     npm --version
     ```

3. **MySQL**
   - **Windows**: Download MySQL Installer from the [official website](https://dev.mysql.com/downloads/installer/) and follow installation prompts
   - **macOS**: Use Homebrew: `brew install mysql` then `brew services start mysql`
   - **Linux (Ubuntu/Debian)**: `sudo apt update && sudo apt install mysql-server`
   - After installation, secure your MySQL installation:
     ```
     sudo mysql_secure_installation
     ```
   - Create a root password when prompted

### Database Setup

1. Log in to MySQL:
   ```
   mysql -u root -p
   ```

2. Create a new database:
   ```sql
   CREATE DATABASE shazam_clone;
   ```

3. Create a user and grant privileges:
   ```sql
   CREATE USER 'shazam_user'@'localhost' IDENTIFIED BY 'your_password';
   GRANT ALL PRIVILEGES ON shazam_clone.* TO 'shazam_user'@'localhost';
   FLUSH PRIVILEGES;
   EXIT;
   ```

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
   - Create a `.env` file in the project root with:
   ```
   DB_USER=shazam_user
   DB_PASSWORD=your_password
   DB_HOST=localhost
   DB_PORT=3306
   DB_NAME=shazam_clone
   
   PORT=3000
   CLIENT_PORT=3001
   
   SPOTIFY_CLIENT_ID=your_spotify_client_id
   SPOTIFY_CLIENT_SECRET=your_spotify_client_secret
   ```

4. Build the Rust component:
   ```
   cd recognition
   cargo build --release
   cd ..
   ```

5. Install backend dependencies:
   ```
   cd backend
   npm install
   cd ..
   ```

6. Install frontend dependencies:
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
   cd backend
   npm start
   ```

4. In a new terminal, start the frontend development server:
   ```
   cd frontend
   npm start
   ```

5. Open your browser and navigate to `http://localhost:3001`

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

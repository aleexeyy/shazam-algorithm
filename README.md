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

## 🛠️ Goals

This project was mainly built for **learning purposes**, especially:
- Diving into **Rust** for performance-critical parts.
- Exploring full-stack development across **React**, **Express**, and system-level processing.
- Understanding **audio fingerprinting** and practical database design.

---

Feel free to explore, modify, or contribute!

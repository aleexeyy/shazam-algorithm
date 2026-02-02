import React from 'react';
import './SongInfo.css';

const SongInfo = ({ songName, artistName }) => {
  return (
    <div className="song-info-container">
      <div className="song-info-header">
        <h2>🎶 Song Info</h2>
      </div>
      <div className="song-info-body">
        <p className="song-name">
          <strong>Song:</strong> <span>{songName}</span>
        </p>
        <p className="artist-name">
          <strong>Artist:</strong> <span>{artistName}</span>
        </p>
      </div>
    </div>
  );
};

export default SongInfo;
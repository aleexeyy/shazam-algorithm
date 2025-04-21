import React, { useState } from 'react';
import './Switch.css'; // optional for styling
import Center from "./Center"
import SongInfo from './SongInfo';

const ModeSwitch = () => {
  const [spotifyUrl, setSpotifyUrl] = useState("");
  const [toRecognize, setToRecognize] = useState(true);
  const [selectedFile, setSelectedFile] = useState(null); 
  const [songName, setSongName] = useState(null); // Store song name
  const [artistName, setArtistName] = useState(null);
  const [fileName, setFileName] = useState('');

  const handleFileChange = (event) => {
    const file = event.target.files[0];
    if (file) {
        setSelectedFile(file);
        setFileName(file.name);
        // setSongName("");
        // setArtistName("");
    }
  };

  const uploadSong = async (toRecognize) => {
    //https://open.spotify.com/track/7AuYlke4foydiCbZbqS5JP?si=f80cdba2494b45b9

    if (!toRecognize) {
    const webSite = spotifyUrl.substring(0, 31);

    if (webSite == "https://open.spotify.com/track/") {
        
        const songId = spotifyUrl.substring(31, 53);
        console.log(songId);

        try {
            const response = await fetch("http://localhost:8000/upload-song", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                  },
                body: JSON.stringify({ "songId" : songId, "toRecognize": toRecognize }),
            })
            const data = await response.json();


            console.log("Server response:", data);
            setToRecognize(false);
          window.location.reload();
        } catch(error) {
            console.error("Error sending songId:", error);
        }
    } else {
        alert("Invalid Spotify link!");
    }
        
    } else {
        // File recognition
        if (!selectedFile) {
          alert("Please upload a file first!");
          return;
        }
  
        const formData = new FormData();

        console.log("Selected file:", selectedFile);
        formData.append("audio", selectedFile);
        formData.append("toRecognize", "true");
        
        try {
          const response = await fetch("http://localhost:8000/recognize-song", {
            method: "POST",
            body: formData,
          });
            const data = await response.json();
            setSongName(data.name);
            setArtistName(data.artist);
            console.log("Recognition result:", data);
        } catch (error) {
          console.error("Error uploading file:", error);
        }
      }
    
};

  const handleToggle = () => {
    setToRecognize(!toRecognize);
    setSongName("");
    setArtistName("");
  };

  return (
    <div style={{ padding: '20px' }}>
        <Center id = 'submit-song'>
        <label className="switch">
            <input type="checkbox" checked={toRecognize} onChange={handleToggle} />
            <span className="slider" />
        </label>
        <span style={{ marginLeft: '15px' }}>
            {toRecognize ? '🎧 Recognize Song' : '📥 Upload Song'}
        </span>
      </Center>
      <div className='centering-div' style={{ marginTop: '20px' }}>
        {toRecognize ? (
            <>
          <button style={{marginBottom :'20px'}} onClick={() => {console.log("Recognizing..."), uploadSong(true) }}>Recognize</button>
          <div className="file-upload-wrapper">
          <input id="file-upload" type="file" accept=".mp3,.wav, .ogg" onChange={handleFileChange}/>
          <label htmlFor="file-upload">🎵 Upload Audio</label>
          </div>
          {fileName && <p>File selected: {fileName}</p>}
          </>
) : (<>
          <button style={{marginBottom :'20px'}} onClick={() => {console.log("Uploading..."), uploadSong(false) }}>Upload</button>
          <input type="url" id="spotify-input" placeholder="https://spotify.com/../..." value ={spotifyUrl} onChange={(e) => setSpotifyUrl(e.target.value)}/>
          </>
        )}
      </div>
      {songName && (
        <SongInfo songName={songName} artistName={artistName || 'Unknown'} />
      )}
    </div>
  );
};

export default ModeSwitch;

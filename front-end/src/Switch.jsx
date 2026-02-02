import { useState } from 'react';
import './Switch.css';
import Center from "./Center"
import SongInfo from './SongInfo';
import { apiUrl } from "./api";

const ModeSwitch = ({ onSongsChanged }) => {
  const [spotifyUrl, setSpotifyUrl] = useState("");
  const [toRecognize, setToRecognize] = useState(true);
  const [selectedFile, setSelectedFile] = useState(null); 
  const [songName, setSongName] = useState(null);
  const [artistName, setArtistName] = useState(null);
  const [fileName, setFileName] = useState('');
  const [requestError, setRequestError] = useState(null);
  const [requestSuccess, setRequestSuccess] = useState(null);

  const handleFileChange = (event) => {
    const file = event.target.files[0];
    if (file) {
        setSelectedFile(file);
        setFileName(file.name);
        setRequestError(null);
        setRequestSuccess(null);
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
            setRequestError(null);
            setRequestSuccess(null);
            const response = await fetch(apiUrl("/upload-song"), {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                  },
                body: JSON.stringify({ "songId" : songId, "toRecognize": toRecognize }),
            })
            let data = null;
            try {
              data = await response.json();
            } catch {
              // ignore non-JSON errors
            }

            if (!response.ok) {
              throw new Error(data?.error || `Upload failed (HTTP ${response.status})`);
            }

            console.log("Server response:", data);
            setRequestSuccess("Upload complete.");
            setSpotifyUrl("");
            setSongName(null);
            setArtistName(null);
            if (typeof onSongsChanged === "function") {
              onSongsChanged();
            }
        } catch(error) {
            console.error("Error sending songId:", error);
            setRequestError(error?.message || "Upload failed.");
        }
    } else {
        setRequestError("Invalid Spotify link!");
    }
        
    } else {
        // File recognition
        if (!selectedFile) {
          setRequestError("Please upload a file first!");
          return;
        }
  
        const formData = new FormData();

        console.log("Selected file:", selectedFile);
        formData.append("audio", selectedFile);
        formData.append("toRecognize", "true");
        
        try {
          setRequestError(null);
          setRequestSuccess(null);
          const response = await fetch(apiUrl("/recognize-song"), {
            method: "POST",
            body: formData,
          });
          const data = await response.json();
          if (!response.ok) {
            throw new Error(data?.error || `Recognition failed (HTTP ${response.status})`);
          }
          setSongName(data.name);
          setArtistName(data.artist);
          console.log("Recognition result:", data);
        } catch (error) {
          console.error("Error uploading file:", error);
          setRequestError(error?.message || "Recognition failed.");
        }
      }
    
};

  const handleToggle = () => {
    setToRecognize(!toRecognize);
    setSongName("");
    setArtistName("");
    setRequestError(null);
    setRequestSuccess(null);
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
      {(requestError || requestSuccess) && (
        <div
          style={{
            marginTop: '12px',
            padding: '10px 12px',
            borderRadius: '8px',
            border: '1px solid',
            borderColor: requestError ? '#ff6b6b' : '#51cf66',
            color: requestError ? '#ff6b6b' : '#51cf66',
            background: 'rgba(0,0,0,0.2)',
            maxWidth: '520px',
            marginLeft: 'auto',
            marginRight: 'auto',
            textAlign: 'center',
          }}
        >
          {requestError || requestSuccess}
        </div>
      )}
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

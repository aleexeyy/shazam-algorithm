require('dotenv').config();
const express = require('express');
const mysql = require('mysql2');
const cors = require('cors');
const { exec } = require("child_process");
const path = require("path");
const fs = require("fs");
const ffi = require('ffi-napi');
const ref = require('ref-napi');
const multer = require('multer');
const ffmpeg = require('fluent-ffmpeg');
const StructType = require('ref-struct-di')(ref);

const storage = multer.diskStorage({
    destination: (req, file, cb) => {
      cb(null, '../audio/') // ensure this folder exists!
    },
    filename: (req, file, cb) => {
      cb(null, "audio_to_recognize") // optional: rename file
    }
  });
  const upload = multer({ storage });


const port = process.env.PORT || 8000;
const client_id = process.env.CLIENT_ID;
const client_secret = process.env.CLIENT_SECRET;


const ShazamResult = StructType({
    name: ref.refType('uint8'),   // pointer to u8
    artist: ref.refType('uint8')  // pointer to u8
  });
  

const libPath = path.resolve(__dirname, '../target/release/libshazam.dylib');

    const rustLib = ffi.Library(libPath, {
        'run_shazam': [
            ShazamResult, // Return type: ShazamResult struct
            [
              ref.refType('uint8'),  // song_name: *const u8
              'size_t',              // song_name_len: usize
              ref.refType('uint8'),  // artist_name: *const u8
              'size_t',              // artist_name_len: usize
              'bool'                 // to_recognize: bool
            ]
        ],
        'free_rust_strings': ['void', [ref.refType('uint8'), ref.refType('uint8')]] // Takes two *mut u8 pointers
    });

const app = express();
app.use(cors());

app.use(express.json());


const database = mysql.createConnection({
    host: 'localhost',
    user: 'root', 
    password: process.env.DATABASE_PASSWORD, 
    database: 'shazam_db'
});

database.connect((err) => {
    if (err) {
        console.error('Error connecting to the database:', err);
        return;
    }
    console.log('Connected to the database!');
});


app.get('/ping', (req, res) => {
    res.json({ message: 'Pong' });
  });



app.get('/songs/count', (req, res) => {

    const query = "SELECT COUNT(*) AS song_count FROM songs";

    database.query(query, (err, results) => {
        if (err) {
            console.error('Error fetching data:', err);
            res.status(500).send('Database error');
            return;
        }
        res.status(200).json({ count: results[0].song_count });
    })
})
async function get_token() {
    const response = await fetch("https://accounts.spotify.com/api/token", {
        method: "POST",
        headers: {
          "Content-Type": "application/x-www-form-urlencoded",
          Authorization: "Basic " + Buffer.from(client_id + ":" + client_secret).toString("base64"),
        },
        body: "grant_type=client_credentials"
      });
      
      const data = await response.json();
      const accessToken = data.access_token;
      return accessToken;
}

function runShazam(songName, artistName, toRecognize = true) {
    console.log("Running Shazam: ", songName, artistName);

    const songNameBuffer = Buffer.from(songName, 'utf8');
    const artistNameBuffer = Buffer.from(artistName, 'utf8');

    const resultStruct = rustLib.run_shazam(
        songNameBuffer,             // song_name
        songNameBuffer.length,      // song_name_len
        artistNameBuffer,           // artist_name
        artistNameBuffer.length,    // artist_name_len
        toRecognize                 // to_recognize
    );

    const namePtr = resultStruct.name;
  const artistPtr = resultStruct.artist;
    
  const resultName = ref.readCString(namePtr);
  const resultArtist = ref.readCString(artistPtr);

    rustLib.free_rust_strings(namePtr, artistPtr);
    return { name: resultName, artist: resultArtist };
}
app.post('/upload-song', async (req, res) => {

    const songId = req.body.songId;
    const toRecognize = req.body.toRecognize;

    const accessToken = await get_token();

    try {
        const response = await fetch(`https://api.spotify.com/v1/tracks/${songId}`, {
            method: "GET",
            headers: {
              Authorization: `Bearer ${accessToken}`
            }
          });
          if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        const data = await response.json();
        
        const downloadStatus = await downloadAudio(data.name, data.artists[0].name);

        if (downloadStatus == 500) {
            res.status(500).send("Error while downloading audio");
        }
        runShazam(data.name, data.artists[0].name, toRecognize);

        return res.status(200).json({uploadStatus : "OK"});
      
    } catch(error) {
        console.error("Error fetching track data:", error);
        res.status(500).send("Error fetching data");
    }
    
});

// function insert_song(songName, artistName) {
//     const query = `INSERT IGNORE INTO songs (name, artist) VALUES (?, ?);`
//     database.query(query, [songName, artistName], (err, results) => {
//         if (err) {
//             console.error("Error inserting song:", err);
//         } else {
//             console.log("Song inserted successfully");
//         }
//     });
    
// }

async function downloadAudio(song_name, artist_name) {


    const searchUrl = `ytsearch1:${artist_name + " " + song_name}`;

    const outputFolder = path.resolve(__dirname, "../audio");


    const outputTemplate = path.join(outputFolder, "song_to_process.%(ext)s");


    console.log("Path to the file: ", path.join(outputFolder, "song_to_process.wav") );
    if (fs.existsSync(path.join(outputFolder, "song_to_process.wav"))) {
         fs.unlinkSync(path.join(outputFolder, "song_to_process.wav"));
    }
    return new Promise((resolve, reject) => {
        const cmd = `yt-dlp -x --audio-format wav -o "${outputTemplate}" "${searchUrl}"`;
        exec(cmd, (error, stdout, stderr) => {
            if (error) {
                console.error(`❌ Error: ${error.message}`);
                reject(500);
            }
            if (stderr) {
                console.error(`⚠️ stderr: ${stderr}`);
                reject(500);
            }
            console.log(`✅ Audio downloaded:\n${stdout}`);
            resolve(200);
        });
    });

}



app.post('/recognize-song', upload.single('audio'), (req, res) => {
    const tempPath = req.file.path;
    const mimetype = req.file.mimetype;
    const targetPath = path.join(__dirname, '../audio/audio_to_recognize.wav');
  
    // Make sure ../audio exists
    const audioDir = path.join(__dirname, '../audio');
    if (!fs.existsSync(audioDir)) fs.mkdirSync(audioDir, { recursive: true });
  
    if (mimetype === 'audio/mpeg') {
      // Convert MP3 to WAV
      ffmpeg(tempPath)
        .toFormat('wav')
        .save(targetPath)
        .on('end', () => {
          fs.unlinkSync(tempPath); // Remove original temp file
          console.log('MP3 converted and saved as WAV');
        //   res.json({ message: 'File uploaded and converted to WAV.' });
        })
        .on('error', (err) => {
          console.error('FFmpeg error:', err);
          res.status(500).json({ error: 'Failed to convert file' });
        });
    } else if (mimetype === 'audio/ogg') {
        // Convert OGG to WAV
        ffmpeg(tempPath)
          .toFormat('wav')
          .save(targetPath)
          .on('end', () => {
            fs.unlinkSync(tempPath); // Remove original temp file
            console.log('OGG converted and saved as WAV');
            // res.json({ message: 'File uploaded and converted to WAV.' });
          })
          .on('error', (err) => {
            console.error('FFmpeg error:', err);
            res.status(500).json({ error: 'Failed to convert OGG file' });
          });
      
      }  else if (mimetype === 'audio/wav') {
      // Move WAV file directly
      fs.rename(tempPath, targetPath, (err) => {
        if (err) {
          console.error('Rename error:', err);
          return res.status(500).json({ error: 'Failed to move WAV file' });
        }
        console.log('WAV file uploaded successfully');
        // res.json({ message: 'WAV file uploaded successfully.' });
      });
    } else {
      fs.unlinkSync(tempPath); // Delete unsupported file
      res.status(400).json({ error: 'Unsupported file type. Please upload MP3 or WAV.' });
    }


    const result = runShazam("", "", true);
    res.status(200).json(result);
  });
  
app.listen(port, () => {
    console.log(`Server running on http://localhost:${port}`);
  });
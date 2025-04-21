import "./App.css"
import ModeSwitch from "./Switch";
import Center from "./Center"
import React, { useState, useEffect } from "react";

export default function App() {
    const [totalSongs, setTotalSongs] = useState(null);
    const [error, setError] = useState(null);
    // const [spotifyUrl, setSpotifyUrl] = useState("");

    
    useEffect(() => {
        const getSongsCount = async () => {
            const apiURL = "http://localhost:8000/songs/count";
            try {
                const response = await fetch(apiURL, {
                        method: 'GET',
                        headers: {
                            'Content-Type': 'application/json',
                        },
                });
                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                const data = await response.json();
                setTotalSongs(data.count);
                setError(null); // Clear any previous errors
            } catch (error) {
                console.error("Error fetching song count:", error);
                setError("Failed to load songs count. Please try again later.");
            }
        };
    
        getSongsCount();
    }, []);

    return (
        <>

        <Center>
        <ModeSwitch></ModeSwitch>
        </Center>

            <div id="song-counter">
                <p id="number-of-songs">
                    {error ? error : (totalSongs !== null ? `${totalSongs} Songs` : "Loading...")}
                </p>
            </div>
        </>
    );
}
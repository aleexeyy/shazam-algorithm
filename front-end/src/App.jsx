import "./App.css"
import ModeSwitch from "./Switch";
import Center from "./Center"
import { useCallback, useEffect, useState } from "react";
import { apiUrl } from "./api";

export default function App() {
    const [totalSongs, setTotalSongs] = useState(null);
    const [error, setError] = useState(null);
    // const [spotifyUrl, setSpotifyUrl] = useState("");

    const refreshSongsCount = useCallback(async () => {
        try {
            const response = await fetch(apiUrl("/songs/count"), {
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
            setError(null);
        } catch (error) {
            console.error("Error fetching song count:", error);
            setError("Failed to load songs count. Please try again later.");
        }
    }, []);
    
    useEffect(() => {
        refreshSongsCount();
    }, [refreshSongsCount]);

    return (
        <>

        <Center>
        <ModeSwitch onSongsChanged={refreshSongsCount}></ModeSwitch>
        </Center>

            <div id="song-counter">
                <p id="number-of-songs">
                    {error ? error : (totalSongs !== null ? `${totalSongs} Songs` : "Loading...")}
                </p>
            </div>
        </>
    );
}

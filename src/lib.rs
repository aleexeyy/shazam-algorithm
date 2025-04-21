mod audio_processing;
mod make_spectr;
mod process_spectr;
mod create_hash;
mod database_interaction;
mod constants;
mod match_song;

#[repr(C)]
pub struct ShazamResult {
    pub name: *mut u8,
    pub artist: *mut u8
}

#[unsafe(no_mangle)]
pub extern fn run_shazam(song_name: *const u8, song_name_len: usize, artist_name: *const u8, artist_name_len: usize, to_recognize: bool) -> ShazamResult {
    let song_name = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(song_name, song_name_len)) };
    let artist_name = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(artist_name, artist_name_len)) };
    
    let (name, artist) = match run_shazam_internal(song_name, artist_name, to_recognize) {
        Ok((n, a)) => (n, a),
        Err(e) => (e.to_string(), String::new())
    };
    
    let name_cstring = std::ffi::CString::new(name).unwrap();
    let artist_cstring = std::ffi::CString::new(artist).unwrap();
    ShazamResult {
        name: name_cstring.into_raw() as *mut u8,
        artist: artist_cstring.into_raw() as *mut u8
    }
}

fn run_shazam_internal(song_name: &str, artist_name: &str, to_recognize: bool) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut conn = database_interaction::set_up_connection()?;

    // database_interaction::clear_database(&mut conn)?;

    let mut song_id: u64 = 0;
    if !to_recognize {
        song_id = database_interaction::insert_song(&mut conn, song_name, artist_name)?;
    }
    let audio: Vec<Vec<f64>>;
    if to_recognize {
        audio = audio_processing::process_audio("audio_to_recognize.wav")?;
    } else {
        audio = audio_processing::process_audio("song_to_process.wav")?;
    }
    let spectr: Vec<Vec<f64>> = make_spectr::window_audio(audio)?;
    let peaks: Vec<Vec<usize>> = process_spectr::find_spectral_peaks(&spectr)?;


    let matched_song_id = create_hash::create_pairs(&peaks, song_id, to_recognize, &mut conn)?;

    let result_message = if !to_recognize {
        (String::from(song_name), String::from(artist_name))
    } else {
        let (_, matched_name, matched_artist) = database_interaction::get_song_info(&mut conn, matched_song_id)?;
        (matched_name, matched_artist)
    };

    Ok(result_message)
}


#[unsafe(no_mangle)]
pub extern fn free_rust_strings(name_ptr: *mut u8, artist_ptr: *mut u8) {
    unsafe {
        if !name_ptr.is_null() {
            let _ = std::ffi::CString::from_raw(name_ptr as *mut std::os::raw::c_char);
        }
        if !artist_ptr.is_null() {
            let _ = std::ffi::CString::from_raw(artist_ptr as *mut std::os::raw::c_char);
        }
    }
}
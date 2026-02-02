mod fingerprinting;
mod paths;

pub mod backend;

#[repr(C)]
pub struct ShazamResult {
    pub name: *mut u8,
    pub artist: *mut u8,
}

#[unsafe(no_mangle)]
pub extern "C" fn run_shazam(
    song_name: *const u8,
    song_name_len: usize,
    artist_name: *const u8,
    artist_name_len: usize,
    to_recognize: bool,
) -> ShazamResult {
    let song_name = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(song_name, song_name_len))
    };
    let artist_name = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(artist_name, artist_name_len))
    };

    let repo = backend::repositories::MySqlRepository::new_from_env();
    let (name, artist) = match repo
        .and_then(|repo| run_shazam_with_repo(&repo, song_name, artist_name, to_recognize))
    {
        Ok((n, a)) => (n, a),
        Err(e) => (e.to_string(), String::new()),
    };

    let name_cstring = cstring_sanitize(name);
    let artist_cstring = cstring_sanitize(artist);
    ShazamResult {
        name: name_cstring.into_raw() as *mut u8,
        artist: artist_cstring.into_raw() as *mut u8,
    }
}

pub fn run_shazam_with_repo(
    repo: &dyn backend::repositories::Repository,
    song_name: &str,
    artist_name: &str,
    to_recognize: bool,
) -> Result<(String, String), backend::error::AppError> {
    let song_id = if to_recognize {
        0
    } else {
        repo.insert_song(song_name, artist_name)?
    };

    let audio = if to_recognize {
        fingerprinting::audio_processing::process_audio("audio_to_recognize.wav")
    } else {
        fingerprinting::audio_processing::process_audio("song_to_process.wav")
    }
    .map_err(|e| backend::error::AppError::internal(e.to_string()))?;

    let peaks = fingerprinting::process_spectr::find_spectral_peaks(&audio)
        .map_err(|e| backend::error::AppError::internal(e.to_string()))?;

    let matched_song_id =
        fingerprinting::create_hash::create_pairs(repo, &peaks, song_id, to_recognize)?;

    if !to_recognize {
        Ok((song_name.to_string(), artist_name.to_string()))
    } else {
        let (_, matched_name, matched_artist) = repo.get_song_info(matched_song_id)?;
        Ok((matched_name, matched_artist))
    }
}

fn cstring_sanitize(s: String) -> std::ffi::CString {
    let sanitized = s.replace('\0', "");
    match std::ffi::CString::new(sanitized) {
        Ok(cstr) => cstr,
        Err(_) => unsafe { std::ffi::CString::from_vec_unchecked(vec![0]) },
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn free_rust_strings(name_ptr: *mut u8, artist_ptr: *mut u8) {
    unsafe {
        if !name_ptr.is_null() {
            let _ = std::ffi::CString::from_raw(name_ptr as *mut std::os::raw::c_char);
        }
        if !artist_ptr.is_null() {
            let _ = std::ffi::CString::from_raw(artist_ptr as *mut std::os::raw::c_char);
        }
    }
}

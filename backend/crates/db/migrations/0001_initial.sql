-- Users
-- Username is case-insensitive for uniqueness (ci collation) but case-preserving
CREATE TABLE IF NOT EXISTS users (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    username VARCHAR(64) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL,
    twitch_id BIGINT UNSIGNED NULL,
    discord_id BIGINT UNSIGNED NULL,
    PRIMARY KEY (id),
    UNIQUE INDEX (username),
    UNIQUE INDEX (twitch_id),
    UNIQUE INDEX (discord_id)
) ENGINE = InnoDB;

-- Credentials
-- Contains credentials for user/pass auth
CREATE TABLE IF NOT EXISTS user_credentials (
    user_id INT UNSIGNED NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    PRIMARY KEY (user_id),
    CONSTRAINT FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
) ENGINE = InnoDB;

-- Capabilities
CREATE TABLE IF NOT EXISTS capabilities (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    title VARCHAR(64) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE INDEX (title)
) ENGINE = InnoDB;

-- User <-> Capability (M2M)
CREATE TABLE IF NOT EXISTS user_capabilities (
    user_id INT UNSIGNED NOT NULL REFERENCES users (id),
    capability_id INT UNSIGNED NOT NULL REFERENCES capabilities (id),
    PRIMARY KEY (user_id, capability_id)
) ENGINE = InnoDB;

-- Artists
-- Representative of both original artists of a song and any singers.
CREATE TABLE IF NOT EXISTS artists (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    name VARCHAR(256) NOT NULL,
    description TEXT NULL,
    PRIMARY KEY (id),
    INDEX (name)
) ENGINE = InnoDB;

-- Tags
CREATE TABLE IF NOT EXISTS tags (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    name VARCHAR(64) NOT NULL,
    kind VARCHAR(64) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE INDEX (name),
    INDEX (kind),
    CONSTRAINT proper_tag_kind CHECK (kind REGEXP '(genre)|(modifier)|(misc)')
) ENGINE = InnoDB;

-- Lyrics
CREATE TABLE IF NOT EXISTS lyrics (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    content TEXT NOT NULL,
    PRIMARY KEY (id)
) ENGINE = InnoDB;

-- Images
-- Can be cover art, thumbnail, etc.
CREATE TABLE IF NOT EXISTS images (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    public_url VARCHAR(512) NOT NULL,
    internal_path VARCHAR(512) NULL,
    credits TEXT NULL,
    PRIMARY KEY (id)
) ENGINE = InnoDB;

-- Playlists
CREATE TABLE IF NOT EXISTS playlists (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    title VARCHAR(256) NOT NULL,
    description TEXT NULL,
    kind VARCHAR(64) NOT NULL,
    created_by INT UNSIGNED NULL REFERENCES users (id),
    PRIMARY KEY (id),
    INDEX (title),
    INDEX (kind),
    CONSTRAINT proper_playlist_kind CHECK (kind REGEXP '(user)|(official)')
) ENGINE = InnoDB;

-- Songs
-- Has default base lyrics of a song
CREATE TABLE IF NOT EXISTS songs (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    title VARCHAR(256) NOT NULL,
    created_by INT UNSIGNED NULL REFERENCES users (id),
    lyrics_id INT UNSIGNED NULL REFERENCES lyrics (id),
    date_added DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    INDEX (title),
    INDEX (date_added)
) ENGINE = InnoDB;

-- Song <-> Image (M2M)
CREATE TABLE IF NOT EXISTS song_images (
    song_id INT UNSIGNED NOT NULL REFERENCES songs (id),
    image_id INT UNSIGNED NOT NULL REFERENCES images (id),
    PRIMARY KEY (song_id, image_id)
) ENGINE = InnoDB;

-- Song <-> Original Artist (M2M)
CREATE TABLE IF NOT EXISTS song_original_artists (
    song_id INT UNSIGNED NOT NULL REFERENCES songs (id),
    artist_id INT UNSIGNED NOT NULL REFERENCES artists (id),
    PRIMARY KEY (song_id, artist_id),
    INDEX (artist_id)
) ENGINE = InnoDB;

-- Song <-> Tag (M2M)
CREATE TABLE IF NOT EXISTS song_tags (
    song_id INT UNSIGNED NOT NULL REFERENCES songs (id),
    tag_id INT UNSIGNED NOT NULL REFERENCES tags (id),
    PRIMARY KEY (song_id, tag_id),
    INDEX (tag_id)
) ENGINE = InnoDB;

-- User <-> Favorite Song (M2M)
CREATE TABLE IF NOT EXISTS user_favorite_songs (
    user_id INT UNSIGNED NOT NULL REFERENCES users (id),
    song_id INT UNSIGNED NOT NULL REFERENCES songs (id),
    PRIMARY KEY (user_id, song_id)
) ENGINE = InnoDB;

-- Performances
-- Representative of a song/mashup performed on stream
-- Providing a lyrics means:
--  Its a custom lyrics tied to this particular performance.
--  Or its a mashup of multiple songs and needs one lyrics from the set.
CREATE TABLE IF NOT EXISTS performances (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    title VARCHAR(256) NULL,
    created_by INT UNSIGNED NULL REFERENCES users (id),
    lyrics_id INT UNSIGNED NULL REFERENCES lyrics (id),
    play_count INT NOT NULL DEFAULT 0,
    duration INT UNSIGNED NULL,
    performance_date DATETIME NOT NULL,
    PRIMARY KEY (id),
    INDEX (performance_date)
) ENGINE = InnoDB;

-- Performance <-> Song (M2M)
CREATE TABLE IF NOT EXISTS performance_songs (
    performance_id INT UNSIGNED NOT NULL REFERENCES performances (id),
    song_id INT UNSIGNED NOT NULL REFERENCES songs (id),
    PRIMARY KEY (performance_id, song_id),
    INDEX (song_id)
) ENGINE = InnoDB;

-- Performance <-> Singer (M2M)
CREATE TABLE IF NOT EXISTS performance_singers (
    performance_id INT UNSIGNED NOT NULL REFERENCES performances (id),
    artist_id INT UNSIGNED NOT NULL REFERENCES artists (id),
    PRIMARY KEY (performance_id, artist_id),
    INDEX (artist_id)
) ENGINE = InnoDB;

-- Performance audios
CREATE TABLE IF NOT EXISTS performance_audios (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    performance_id INT UNSIGNED NOT NULL REFERENCES performances (id),
    public_url VARCHAR(512) NOT NULL,
    internal_path VARCHAR(512) NULL,
    PRIMARY KEY (id),
    INDEX (performance_id)
) ENGINE = InnoDB;

-- Performance videos
CREATE TABLE IF NOT EXISTS performance_videos (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    performance_id INT UNSIGNED NOT NULL REFERENCES performances (id),
    public_url VARCHAR(512) NOT NULL,
    internal_path VARCHAR(512) NULL,
    PRIMARY KEY (id),
    INDEX (performance_id)
) ENGINE = InnoDB;

-- Playlist <-> Performance (M2M, ordered)
CREATE TABLE IF NOT EXISTS playlist_performances (
    playlist_id INT UNSIGNED NOT NULL REFERENCES playlists (id),
    performance_id INT UNSIGNED NOT NULL REFERENCES performances (id),
    sort_order INT NOT NULL DEFAULT 0,
    PRIMARY KEY (playlist_id, performance_id),
    INDEX (performance_id)
) ENGINE = InnoDB;

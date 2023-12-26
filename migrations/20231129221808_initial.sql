CREATE TABLE users (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY, -- ID of the user.
    uuid UUID NOT NULL UNIQUE, -- UUID of the user, used for passkey authentication.
    username TEXT NOT NULL UNIQUE, -- Unique username of the user.
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- When the user was created.
    last_authentication TIMESTAMPTZ -- When the user last authenticated using a passkey credential.
);

CREATE TABLE files (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY, -- ID of the file.
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE, -- ID of the user who uploaded the file, if any.
    file_name TEXT NOT NULL, -- Name of the file (example: my_file.txt)
    file_path TEXT NOT NULL, -- Path to the file on the server.
    size INTEGER NOT NULL, -- Size of the file in bytes.
    md5 VARCHAR(32) NOT NULL, -- MD5 hash of the file for integrity checking.
    sha1 VARCHAR(40) NOT NULL, -- SHA1 hash of the file for integrity checking.
    sha256 VARCHAR(32) NOT NULL, -- SHA256 hash of the file for integrity checking.
    blake3 VARCHAR(32) NOT NULL, -- BLAKE3 hash of the file for integrity checking.
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- When the file was uploaded.
    expires_at TIMESTAMPTZ -- If and when the file will be deleted.
);

CREATE TABLE pastes (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY, -- ID of the paste.
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE, -- ID of the user who uploaded the paste, if any.
    title TEXT, -- Optional name of the paste (example: My Meatloaf Recipe)
    content TEXT NOT NULL, -- Raw text content of the paste.
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- When the paste was uploaded.
    expires_at TIMESTAMPTZ -- If and when the paste will be deleted.
);

CREATE TABLE slugs (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY, -- ID of the slug.
    file_id INTEGER REFERENCES files(id) ON DELETE CASCADE, -- ID of the file associated with the slug.
    paste_id INTEGER REFERENCES pastes(id) ON DELETE CASCADE , -- ID of the paste associated with the slug.
    slug TEXT NOT NULL UNIQUE, -- Unique slug for accessing the slug in URLs (example: honest-turbo-tailor-gregory)
    enabled TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP, -- If the slug is enabled or not and when it was last enabled.
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP -- When the slug was created.
);

CREATE TABLE credentials (
    id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY, -- ID of the credential.
    user_uuid UUID NOT NULL REFERENCES users(uuid) ON DELETE CASCADE , -- ID of the user who owns the credential.
    passkey JSON NOT NULL, -- JSON string of a webauthn-rs passkey credential.
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP, -- When the credential was created.
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP -- When the credential was last updated.
);

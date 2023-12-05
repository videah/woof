INSERT INTO pastes
    ( user_id, title, content, expires_at )
VALUES
    ( $1, $2, $3, $4 )
RETURNING *
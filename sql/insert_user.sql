INSERT INTO users
    ( username, uuid )
VALUES
    ( $1, $2 )
RETURNING *
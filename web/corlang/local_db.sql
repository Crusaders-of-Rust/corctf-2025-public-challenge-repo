CREATE TABLE IF NOT EXISTS conlang_user (
    username VARCHAR(128) PRIMARY KEY,
    password VARCHAR(255),
    karma INT,
    is_admin BOOLEAN
);

-- Default admin pass is 'admin' (but not on the remote instance #security)
INSERT INTO conlang_user (username, password, karma, is_admin) VALUES ('admin', '$2a$12$CiHVw367HTHDyHDzDJKdOugob13HfJ5ZpKRBv3aFD2p9370nwdYzq', 0, TRUE);
-- The default password for user is 'user' on the remote instance as well.
INSERT INTO conlang_user (username, password, karma, is_admin) VALUES ('user', '$2a$12$gtIdsb1jjEz4Ql7r8ma0D.yFVxhqt2HPhynNBliu4LX8vzp7SIItK', 0, FALSE);

CREATE TABLE IF NOT EXISTS conlang_token (
    id INT PRIMARY KEY,
    token_name VARCHAR(128),
    token_value  VARCHAR(255)
);

INSERT INTO conlang_token (id, token_name, token_value) VALUES (1, 'approval_token', 'corctf{EXAMPLE_FLAG}');
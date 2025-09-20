CREATE TABLE IF NOT EXISTS conlang_user (
    username VARCHAR(128) PRIMARY KEY,
    password VARCHAR(255),
    karma INT,
    is_admin BOOLEAN
);


INSERT INTO conlang_user (username, password, karma, is_admin) VALUES ('admin', '$2a$12$I9rPdYS4qFk0nL0D4UEDu.rp.7Qg0Lz305GvIxZ57/yk0TptAP.gK', 0, TRUE);
-- User's password is 'user'
INSERT INTO conlang_user (username, password, karma, is_admin) VALUES ('user', '$2a$12$gtIdsb1jjEz4Ql7r8ma0D.yFVxhqt2HPhynNBliu4LX8vzp7SIItK', 0, FALSE);

CREATE TABLE IF NOT EXISTS conlang_token (
    id INT PRIMARY KEY,
    token_name VARCHAR(128),
    token_value  VARCHAR(255)
);

INSERT INTO conlang_token (id, token_name, token_value) VALUES (1, 'approval_token', 'corctf{Th3_Re4l_Ch4ll3ng3_W45_R34d1ng_J4v4}');
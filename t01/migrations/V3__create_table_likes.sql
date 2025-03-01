create table if not exists likes (
       like_id    serial primary key,
       user_id       int references users(user_id) on delete cascade,
       post_id       int references posts(post_id) on delete cascade,
    created_at timestamp default current_timestamp,

    unique(user_id, post_id)
);

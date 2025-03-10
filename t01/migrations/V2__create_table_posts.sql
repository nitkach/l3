create table if not exists posts (
       post_id    serial primary key,
       user_id       int references users(user_id) on delete cascade,
         title      text not null,
       content      text not null,
    created_at timestamp default current_timestamp
);

create table if not exists users (
          user_id    serial primary key,
            login      text unique not null,
    password_hash      text        not null,
       created_at timestamp default current_timestamp
);

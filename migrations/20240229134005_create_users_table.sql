create table users(
   user_id uuid primary key,
   username text not null unique,
   password_hash text not null
);

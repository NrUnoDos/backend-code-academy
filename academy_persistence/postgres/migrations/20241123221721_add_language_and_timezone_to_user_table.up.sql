alter table if exists users
    add column if not exists preferred_language text,
    add column if not exists timezone text;
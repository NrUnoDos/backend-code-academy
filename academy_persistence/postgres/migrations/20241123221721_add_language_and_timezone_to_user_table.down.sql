alter table if exists users
    drop column if exists preferred_language,
    drop column if exists timezone;
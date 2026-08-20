-- Persian Literary Translation Engine
-- Supabase/Postgres foundation
-- Keep manuscripts private. Enable RLS for future API exposure.

create table if not exists projects (
  id uuid primary key default gen_random_uuid(),
  name text not null,
  created_at timestamptz not null default now()
);

create table if not exists translation_memory (
  id uuid primary key default gen_random_uuid(),
  project_id uuid references projects(id) on delete cascade,
  source_text text not null,
  translated_text text not null,
  source_language text not null default 'en',
  target_language text not null default 'fa',
  created_at timestamptz not null default now()
);

create table if not exists glossary_entries (
  id uuid primary key default gen_random_uuid(),
  project_id uuid references projects(id) on delete cascade,
  term text not null,
  preferred_translation text not null,
  notes text,
  created_at timestamptz not null default now()
);

create table if not exists character_profiles (
  id uuid primary key default gen_random_uuid(),
  project_id uuid references projects(id) on delete cascade,
  character_name text not null,
  personality_notes text,
  speech_style text,
  created_at timestamptz not null default now()
);

alter table projects enable row level security;
alter table translation_memory enable row level security;
alter table glossary_entries enable row level security;
alter table character_profiles enable row level security;

-- Policies will be added together with Auth integration.

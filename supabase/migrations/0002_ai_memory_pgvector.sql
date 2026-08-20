-- Semantic memory layer for Persian Literary Translation Engine
-- Uses pgvector-compatible design for future RAG retrieval.

create extension if not exists vector;

create table if not exists semantic_memory (
  id uuid primary key default gen_random_uuid(),
  project_id uuid references projects(id) on delete cascade,
  memory_type text not null check (memory_type in ('translation','character','style','research')),
  source_text text not null,
  metadata jsonb default '{}'::jsonb,
  embedding vector(1536),
  created_at timestamptz not null default now()
);

create index if not exists semantic_memory_embedding_idx
on semantic_memory
using ivfflat (embedding vector_cosine_ops)
with (lists = 100);

alter table semantic_memory enable row level security;

-- Ownership policies will be added with authentication layer.

create table if not exists style_guides (
  id uuid primary key default gen_random_uuid(),
  project_id uuid references projects(id) on delete cascade,
  voice_rules text,
  formatting_rules text,
  forbidden_patterns text,
  created_at timestamptz not null default now()
);

alter table style_guides enable row level security;

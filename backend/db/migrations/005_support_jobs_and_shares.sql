-- Migration: 005_support_jobs_and_shares.sql
-- Description: Add shares_outstanding to companies and create job_runs table
-- Date: 2026-01-21

-- Add shares_outstanding to companies
ALTER TABLE companies ADD COLUMN IF NOT EXISTS shares_outstanding BIGINT;

-- Create job_runs table if it doesn't exist
CREATE TABLE IF NOT EXISTS job_runs (
    id                  UUID PRIMARY KEY,
    job_name            VARCHAR(50) NOT NULL,
    status              VARCHAR(20) NOT NULL,
    started_at          TIMESTAMPTZ NOT NULL,
    ended_at            TIMESTAMPTZ,
    result              JSONB,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for job_runs
CREATE INDEX IF NOT EXISTS idx_job_runs_name ON job_runs(job_name);
CREATE INDEX IF NOT EXISTS idx_job_runs_created ON job_runs(created_at DESC);

-- Migration: 004_update_documents_table.sql
-- Description: Add fiscal year/quarter and make storage_key optional
-- Created: 2026-01-18

ALTER TABLE documents ADD COLUMN fiscal_year INTEGER;
ALTER TABLE documents ADD COLUMN fiscal_quarter INTEGER;

-- Make storage_key nullable to support not-yet-downloaded documents
ALTER TABLE documents ALTER COLUMN storage_key DROP NOT NULL;

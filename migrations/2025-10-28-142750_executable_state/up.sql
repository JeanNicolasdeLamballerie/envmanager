ALTER TABLE executables ADD COLUMN mode TEXT NOT NULL DEFAULT 'wait';

-- Optional: mark known GUI tools as detached by default
UPDATE executables SET mode = 'detach'
WHERE name IN ('Neovide');

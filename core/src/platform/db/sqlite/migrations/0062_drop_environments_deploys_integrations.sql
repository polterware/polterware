DELETE FROM canon_entities WHERE kind = 'environment' OR ref_kind = 'environment';
-- Environments, deploys, and platform integrations have been removed.
DROP TABLE IF EXISTS deploy_checks;
DROP TABLE IF EXISTS node_deployments;
DROP TABLE IF EXISTS integration_resources;
DROP TABLE IF EXISTS integration_sync_runs;
DROP TABLE IF EXISTS integration_connections;
ALTER TABLE project_settings DROP COLUMN default_environment_id;
DROP TABLE IF EXISTS environments;

-- Kill branch K2–K4: drop Operations / Workflows / Schedules / process sessions / ops logs / Nodes.
DROP TABLE IF EXISTS log_chunks;
DROP TABLE IF EXISTS process_sessions;
DROP TABLE IF EXISTS operation_schedules;
DROP TABLE IF EXISTS operations;
DROP TABLE IF EXISTS workflows;
DROP TABLE IF EXISTS nodes;

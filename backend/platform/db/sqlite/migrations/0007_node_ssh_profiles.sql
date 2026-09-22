-- SSH host profile fields on nodes (metadata only; no private key material).

ALTER TABLE nodes ADD COLUMN connection_kind TEXT NOT NULL DEFAULT 'local';
ALTER TABLE nodes ADD COLUMN ssh_user TEXT NOT NULL DEFAULT '';
ALTER TABLE nodes ADD COLUMN ssh_port INTEGER NOT NULL DEFAULT 22;
ALTER TABLE nodes ADD COLUMN ssh_identity_file TEXT NOT NULL DEFAULT '';

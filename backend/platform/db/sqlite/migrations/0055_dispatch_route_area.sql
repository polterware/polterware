-- The area a routing decision priced its candidates in: the kinds and
-- technologies of the run's Structure component, as JSON. Learning reads it
-- back after the run to move each model's per-area offsets.

ALTER TABLE dispatch_route_decisions ADD COLUMN area_json TEXT NOT NULL DEFAULT '{}';

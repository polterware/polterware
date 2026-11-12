-- A canvas is Mermaid source text. The semantic graph is read from the source,
-- and layout_json keeps the positions people gave nodes (each pinned).
ALTER TABLE canvases ADD COLUMN source TEXT NOT NULL DEFAULT '';

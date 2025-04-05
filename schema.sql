
DROP TABLE IF EXISTS global_mcp_configs;
CREATE TABLE global_mcp_configs (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    url TEXT NOT NULL
);

DROP TABLE IF EXISTS project_mcp_configs;
CREATE TABLE project_mcp_configs (
    session_id TEXT NOT NULL,
    id TEXT NOT NULL,
    label TEXT NOT NULL,
    url TEXT NOT NULL,
    PRIMARY KEY (session_id, id)
);

INSERT INTO global_mcp_configs (id, label, url) VALUES
  ('default', 'Default (127.0.0.1)', 'http://127.0.0.1:8080/mcp'),
  ('dev', 'Dev (192.168.1.50)', 'http://192.168.1.50:8080/mcp'),
  ('test', 'Test (localhost:8081)', 'http://localhost:8081/mcp'),
  ('prod', 'Prod (10.0.0.2)', 'http://10.0.0.2:8080/mcp');

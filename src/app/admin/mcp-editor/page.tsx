'use client';
import { useEffect, useState } from 'react';

export default function AdminMCPEditor() {
  const [globalConfigs, setGlobalConfigs] = useState([]);
  const [sessionConfigs, setSessionConfigs] = useState([]);
  const [sessionId, setSessionId] = useState('');
  const [scope, setScope] = useState('global');

  useEffect(() => {
    fetch('/api/mcp/config?scope=global')
      .then(res => res.json())
      .then(data => setGlobalConfigs(data || []));
  }, []);

  const handleSave = async (entry: any) => {
    await fetch('/api/mcp/config', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ scope, sessionId, entry })
    });
    alert('Saved');
  };

  return (
    <div className="p-6 space-y-4">
      <h2 className="text-xl font-bold">MCP Config Editor</h2>

      <div>
        <label className="mr-2 font-medium">Scope:</label>
        <select value={scope} onChange={e => setScope(e.target.value)} className="border rounded p-1">
          <option value="global">Global</option>
          <option value="project">Project</option>
        </select>

        {scope === 'project' && (
          <input
            className="ml-2 border p-1 rounded"
            placeholder="Session ID"
            value={sessionId}
            onChange={e => setSessionId(e.target.value)}
          />
        )}
      </div>

      <form
        onSubmit={e => {
          e.preventDefault();
          const entry = Object.fromEntries(new FormData(e.currentTarget));
          handleSave(entry);
        }}
        className="space-y-2"
      >
        <input name="id" className="border p-1 rounded w-full" placeholder="MCP ID" required />
        <input name="label" className="border p-1 rounded w-full" placeholder="Label" required />
        <input name="url" className="border p-1 rounded w-full" placeholder="URL" required />
        <button type="submit" className="bg-blue-600 text-white px-4 py-1 rounded">Save</button>
      </form>
    </div>
  );
}

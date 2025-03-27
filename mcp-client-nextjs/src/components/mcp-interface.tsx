'use client';

import { useState, useEffect, useRef } from 'react';

export default function MCPInterface() {
  const [query, setQuery] = useState('');
  const [response, setResponse] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);
  const [serverStatus, setServerStatus] = useState<'checking' | 'connected' | 'error'>('checking');
  const [logs, setLogs] = useState<string[]>([]);
  const [logsPanelWidth, setLogsPanelWidth] = useState(300);
  const [tools, setTools] = useState<string[]>([]);
  const [selectedTool, setSelectedTool] = useState<string>('');
  const dividerRef = useRef<HTMLDivElement>(null);
  const isDraggingRef = useRef(false);

  useEffect(() => {
    const initClient = async () => {
      log('checking mcp server status...');
      try {
        // Use the API route instead of direct import
        const response = await fetch('/api/mcp/initialize');
        const data = await response.json();
        
        if (data.status === 'connected') {
          setServerStatus('connected');
          log('connected to mcp server');
          fetchTools();
        } else {
          setServerStatus('error');
          log(`failed to connect to mcp server: ${data.error || 'unknown error'}`);
        }
      } catch (error: unknown) {
        setServerStatus('error');
        log(`server error: ${error instanceof Error ? error.message : String(error)}`);
      }
    };
    
    initClient();
  }, []);

  // Fetch available tools from the server
  const fetchTools = async () => {
    log('fetching available tools...');
    try {
      const response = await fetch('/api/mcp/tools');
      const data = await response.json();
      
      if (data.tools && Array.isArray(data.tools)) {
        setTools(data.tools);
        log(`loaded ${data.tools.length} tools`);
        if (data.tools.length > 0) {
          setSelectedTool(data.tools[0]);
          log(`default tool selected: ${data.tools[0]}`);
        }
      } else {
        log('no tools available or invalid response format');
      }
    } catch (error) {
      log(`error fetching tools: ${error instanceof Error ? error.message : String(error)}`);
    }
  };

  useEffect(() => {
    const handleMouseDown = (e: MouseEvent) => {
      isDraggingRef.current = true;
      document.body.style.userSelect = 'none';
      document.body.style.cursor = 'ew-resize';
    };

    const handleMouseMove = (e: MouseEvent) => {
      if (!isDraggingRef.current) return;
      
      // Calculate new width based on mouse position
      const containerRect = document.querySelector('.container')?.getBoundingClientRect();
      if (containerRect) {
        const newWidth = Math.max(200, Math.min(600, containerRect.right - e.clientX));
        setLogsPanelWidth(newWidth);
      }
    };

    const handleMouseUp = () => {
      isDraggingRef.current = false;
      document.body.style.userSelect = '';
      document.body.style.cursor = '';
    };

    const divider = dividerRef.current;
    divider?.addEventListener('mousedown', handleMouseDown);
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      divider?.removeEventListener('mousedown', handleMouseDown);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, []);

  const log = (message: string) => {
    setLogs(prev => [...prev, message]);
    console.log(message);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!query.trim() || isProcessing) return;
    
    setIsProcessing(true);
    log(`sending query: ${query}`);
    if (selectedTool) {
      log(`using tool: ${selectedTool}`);
    }
    
    try {
      const response = await fetch('/api/mcp/query', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ query, tool: selectedTool }),
      });
      
      if (!response.ok) {
        throw new Error(`http error ${response.status}: ${response.statusText}`);
      }
      
      const data = await response.json();
      log('received response from server');
      setResponse(data.response);
    } catch (error) {
      log(`query error: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setIsProcessing(false);
    }
  };

  const handleToolChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const tool = e.target.value;
    setSelectedTool(tool);
    log(`selected tool: ${tool}`);
  };

  return (
    <div className="container mx-auto p-4 h-[90vh] max-w-7xl">
      <h1 className="text-xl font-mono mb-4">mcp client</h1>
      
      <div className="flex gap-0 h-[calc(100%-3rem)]">
        {/* Main chat area */}
        <div className="flex-1 flex flex-col border border-gray-300 rounded">
          {/* Server status and tools */}
          <div className="p-2 border-b border-gray-300 flex justify-between items-center">
            <div>
              <span className="mr-2 font-mono text-xs">server status:</span>
              {serverStatus === 'checking' && <span className="text-yellow-600 text-xs">checking...</span>}
              {serverStatus === 'connected' && <span className="text-green-600 text-xs">connected</span>}
              {serverStatus === 'error' && <span className="text-red-600 text-xs">disconnected</span>}
            </div>
            
            {/* Tool selector dropdown */}
            {serverStatus === 'connected' && tools.length > 0 && (
              <div className="flex items-center">
                <span className="mr-2 font-mono text-xs">tool list:</span>
                <select
                  value={selectedTool}
                  onChange={handleToolChange}
                  className="border border-gray-300 rounded p-1 text-xs font-mono"
                  disabled={isProcessing}
                >
                  {tools.map((tool) => (
                    <option key={tool} value={tool}>
                      {tool}
                    </option>
                  ))}
                </select>
              </div>
            )}
          </div>
          
          {/* Message history area */}
          <div className="flex-1 overflow-y-auto p-4">
            {response && (
              <div className="mb-4 p-3 border border-gray-300 rounded">
                <h2 className="text-xs font-mono mb-1">response:</h2>
                <pre className="whitespace-pre-wrap font-mono text-xs">{response}</pre>
              </div>
            )}
          </div>
          
          {/* Message input area */}
          <div className="p-3 border-t border-gray-300">
            <form onSubmit={handleSubmit} className="flex gap-2">
              <input
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                className="flex-grow p-2 border border-gray-300 rounded font-mono text-xs"
                placeholder="enter your query..."
                disabled={isProcessing || serverStatus !== 'connected'}
              />
              <button
                type="submit"
                className="px-4 py-2 bg-black text-white font-mono rounded disabled:opacity-50 text-xs"
                disabled={isProcessing || serverStatus !== 'connected'}
              >
                {isProcessing ? 'processing...' : 'submit'}
              </button>
            </form>
          </div>
        </div>
        
        {/* Resizable divider */}
        <div 
          ref={dividerRef}
          className="w-1 bg-gray-300 hover:bg-gray-500 cursor-ew-resize"
          title="drag to resize"
        />
        
        {/* Logs panel (right side) */}
        <div 
          className="border border-gray-300 rounded overflow-hidden"
          style={{ width: `${logsPanelWidth}px` }}
        >
          <h2 className="text-sm font-mono p-2 border-b border-gray-300">logs:</h2>
          <div className="h-[calc(100%-2.5rem)] overflow-y-auto font-mono text-xs p-2">
            {logs.map((log, i) => (
              <div key={i} className="mb-1">{log}</div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
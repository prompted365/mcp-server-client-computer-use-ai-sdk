'use client';

import { useState, useEffect, useRef } from 'react';
import { clearLogs } from '../lib/mcp/log-buffer';
import { Moon, Sun } from 'lucide-react';

export default function MCPInterface() {
  const [query, setQuery] = useState('');
  const [response, setResponse] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);
  const [serverStatus, setServerStatus] = useState<'checking' | 'connected' | 'error'>('checking');
  const [logs, setLogs] = useState<string[]>([]);
  const [logsPanelWidth, setLogsPanelWidth] = useState(300);
  const [theme, setTheme] = useState<'light' | 'dark'>('light');
  const dividerRef = useRef<HTMLDivElement>(null);
  const isDraggingRef = useRef(false);
  const [lastLogTimestamp, setLastLogTimestamp] = useState<number>(0);
  const logsEndRef = useRef<HTMLDivElement>(null);
  
  // predefined prompts array
  const predefinedPrompts = [
    "send message to first dialogie in messages app. message is 'i'm testing computer-use-sdk'",
    "go to discord, click 'direct messages' dialogue, then send message 'i'm testing computer-use-sdk'"
  ];

  useEffect(() => {
    const initClient = async () => {
      log('checking mcp server status...');
      try {
        // Clear logs when connecting to server
        clearLogs();
        setLogs([]); // Also clear the local logs state
        
        // Use the API route instead of direct import
        const response = await fetch('/api/mcp/initialize');
        const data = await response.json();
        
        if (data.status === 'connected') {
          setServerStatus('connected');
          log('connected to mcp server');
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

  const submitQuery = async (text: string) => {
    if (!text.trim() || isProcessing || serverStatus !== 'connected') {
      log(`submission skipped: ${!text.trim() ? 'empty query' : isProcessing ? 'already processing' : 'server disconnected'}`);
      return;
    }
    
    // Clear logs when submitting a new query
    clearLogs();
    setLogs([]); // Also clear the local logs state
    
    setIsProcessing(true);
    log(`sending query: ${text}`);
    
    try {
      const response = await fetch('/api/mcp/query', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({ query: text }),
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

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    submitQuery(query);
  };

  // Fetch logs from server periodically
  useEffect(() => {
    if (serverStatus !== 'connected') return;
    
    // Function to fetch logs
    const fetchServerLogs = async () => {
      try {
        const response = await fetch(`/api/mcp/logs?since=${lastLogTimestamp}`);
        if (!response.ok) throw new Error(`HTTP error ${response.status}`);
        
        const data = await response.json();
        if (data.logs && data.logs.length > 0) {
          // Add server logs to our logs state
          const serverLogs = data.logs.map((log: any) => ({
            message: log.message,
            level: log.level
          }));
          
          setLogs(prev => [...prev, ...serverLogs]);
          
          // Update timestamp for next fetch
          const latestTimestamp = Math.max(...data.logs.map((l: any) => l.timestamp));
          setLastLogTimestamp(latestTimestamp);
        }
      } catch (error) {
        console.error('Error fetching server logs:', error);
      }
    };
    
    // Fetch immediately on connect
    fetchServerLogs();
    
    // Then poll every 1 second
    const interval = setInterval(fetchServerLogs, 1000);
    return () => clearInterval(interval);
  }, [serverStatus, lastLogTimestamp]);
  
  // Auto-scroll logs to bottom when new logs arrive
  useEffect(() => {
    if (logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs]);

  // Add this effect after the other useEffects
  useEffect(() => {
    // Set initial 50:50 split on component mount
    const containerEl = document.querySelector('.container');
    if (containerEl) {
      const containerWidth = containerEl.getBoundingClientRect().width;
      log(`setting initial 50:50 split with container width: ${containerWidth}px`);
      setLogsPanelWidth(Math.floor(containerWidth / 2) - 10); // account for divider width and borders
    }
  }, []);

  // Toggle theme function
  const toggleTheme = () => {
    const newTheme = theme === 'light' ? 'dark' : 'light';
    setTheme(newTheme);
    log(`switched to ${newTheme} theme - updating container and child elements`);
    
    // Apply theme class to html element (not document)
    if (newTheme === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  };

  // Set initial theme on mount
  useEffect(() => {
    // Check if user has a preference in localStorage or use system preference
    const savedTheme = localStorage.getItem('mcp-theme');
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    
    const initialTheme = savedTheme === 'dark' || (!savedTheme && prefersDark) ? 'dark' : 'light';
    setTheme(initialTheme);
    
    if (initialTheme === 'dark') {
      document.documentElement.classList.add('dark');
      log('initializing with dark theme - setting .dark class on html element');
    } else {
      document.documentElement.classList.remove('dark');
      log('initializing with light theme - removing .dark class from html element');
    }
    
    log(`initialized with ${initialTheme} theme based on ${savedTheme ? 'saved preference' : 'system preference'}`);
  }, []);
  
  // Save theme preference when it changes
  useEffect(() => {
    localStorage.setItem('mcp-theme', theme);
  }, [theme]);

  return (
    <div className="container mx-auto p-4 h-[90vh] max-w-7xl bg-[var(--background)]">
      <div className="flex justify-between items-center mb-4">
        <h1 className="text-xl font-mono text-[var(--foreground)]">mcp client</h1>
        
        {/* Theme toggle button */}
        <button 
          onClick={toggleTheme}
          className="p-2 rounded-full bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
          aria-label={`Switch to ${theme === 'light' ? 'dark' : 'light'} mode`}
        >
          {theme === 'light' ? (
            <Moon size={16} className="text-gray-200" />
          ) : (
            <Sun size={16} className="text-yellow-200" />
          )}
        </button>
      </div>
      
      <div className="flex gap-0 h-[calc(100%-3rem)]">
        {/* Main chat area */}
        <div className="flex-1 flex flex-col border border-gray-300 dark:border-gray-700 rounded bg-[var(--background)] text-[var(--foreground)]">
          {/* Server status */}
          <div className="p-2 border-b border-gray-300 dark:border-gray-700">
            <span className="mr-2 font-mono text-xs text-[var(--foreground)]">server status:</span>
            {serverStatus === 'checking' && <span className="text-yellow-600 dark:text-yellow-400 text-xs">checking...</span>}
            {serverStatus === 'connected' && <span className="text-green-600 dark:text-green-400 text-xs">connected</span>}
            {serverStatus === 'error' && <span className="text-red-600 dark:text-red-400 text-xs">disconnected</span>}
          </div>
          
          {/* Message history area */}
          <div className="flex-1 overflow-y-auto p-4 text-[var(--foreground)]">
            {response && (
              <div className="mb-4 p-3 border border-gray-300 dark:border-gray-700 rounded bg-[var(--background)]">
                <h2 className="text-xs font-mono mb-1 text-[var(--foreground)]">response:</h2>
                <pre className="whitespace-pre-wrap font-mono text-xs text-[var(--foreground)]">{response}</pre>
              </div>
            )}
          </div>
          
          {/* Message input area */}
          <div className="p-3 border-t border-gray-300 dark:border-gray-700">
            <form onSubmit={handleSubmit} className="flex flex-col gap-2">
              <div className="mb-3">
                <div className="space-y-2">
                  {predefinedPrompts.map((prompt, index) => (
                    <div 
                      key={index}
                      className="text-xs font-mono p-2 border border-gray-200 dark:border-gray-700 rounded cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-150 break-words text-[var(--foreground)] bg-[var(--background)]"
                      onClick={() => {
                        const promptText = prompt;
                        log(`selected prompt: ${promptText.substring(0, 30)}...`);
                        setQuery(promptText);
                        submitQuery(promptText);
                      }}
                    >
                      {prompt}
                    </div>
                  ))}
                </div>
              </div>
              
              <div className="flex gap-2">
                <input
                  type="text"
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  className="flex-grow p-2 border border-gray-300 dark:border-gray-700 rounded font-mono text-xs bg-[var(--background)] text-[var(--foreground)]"
                  placeholder="enter your query..."
                  disabled={isProcessing || serverStatus !== 'connected'}
                />
                <button
                  type="submit"
                  className="px-4 py-2 bg-black dark:bg-white text-white dark:text-black font-mono rounded disabled:opacity-50 text-xs"
                  disabled={isProcessing || serverStatus !== 'connected'}
                >
                  {isProcessing ? 'processing...' : 'submit'}
                </button>
              </div>
            </form>
          </div>
        </div>
        
        {/* Resizable divider */}
        <div 
          ref={dividerRef}
          className="w-1 bg-gray-300 dark:bg-gray-600 hover:bg-gray-500 dark:hover:bg-gray-500 cursor-ew-resize"
          title="drag to resize"
        />
        
        {/* Logs panel (right side) */}
        <div 
          className="border border-gray-300 dark:border-gray-700 rounded overflow-hidden bg-[var(--background)]"
          style={{ width: `${logsPanelWidth}px` }}
        >
          <h2 className="text-sm font-mono p-2 border-b border-gray-300 dark:border-gray-700 text-[var(--foreground)]">logs:</h2>
          <div className="h-[calc(100%-2.5rem)] overflow-y-auto font-mono text-xs p-2 text-[var(--foreground)]">
            {logs.map((log, i) => {
              // Handle both string logs (from client) and object logs (from server)
              const isLogObject = typeof log !== 'string';
              const logLevel = isLogObject ? log.level : '';
              const logMessage = isLogObject ? log.message : log;
              
              return (
                <div key={i} className={`mb-1 ${
                  logLevel === 'error' || logLevel === 'tool-error' ? 'text-red-500 dark:text-red-400' : 
                  logLevel === 'success' ? 'text-green-500 dark:text-green-400' : 
                  logLevel === 'info' ? 'text-blue-500 dark:text-blue-400' : 
                  logLevel === 'debug' ? 'text-gray-500 dark:text-gray-400' : 
                  logLevel === 'highlight' ? 'text-purple-500 dark:text-purple-400' : 
                  ''
                }`}>
                  {logMessage}
                </div>
              );
            })}
            <div ref={logsEndRef} />
          </div>
        </div>
      </div>
    </div>
  );
}
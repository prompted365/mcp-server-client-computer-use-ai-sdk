// Simple in-memory buffer to store logs for client retrieval
class LogBuffer {
    private logs: { timestamp: number; level: string; message: string }[] = [];
    private maxLogs = 1000; // Limit buffer size to prevent memory issues
  
    addLog(level: string, message: string) {
      this.logs.push({
        timestamp: Date.now(),
        level,
        message
      });
      
      // Trim old logs if buffer gets too large
      if (this.logs.length > this.maxLogs) {
        this.logs = this.logs.slice(-this.maxLogs);
      }
    }
  
    getLogs(since?: number): { timestamp: number; level: string; message: string }[] {
      if (since) {
        return this.logs.filter(log => log.timestamp > since);
      }
      return [...this.logs];
    }
  
    clear() {
      this.logs = [];
    }
  }
  
  // Export a singleton instance
  export const logBuffer = new LogBuffer();

  // Export a utility function to clear logs when needed
  export const clearLogs = () => {
    console.log("clearing logs buffer");
    logBuffer.clear();
  };
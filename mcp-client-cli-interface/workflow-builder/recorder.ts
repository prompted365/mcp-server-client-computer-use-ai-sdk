// ... existing code ...
class WorkflowRecorder {
    private mcp: McpClient;
    private recording: WorkflowStep[] = [];
    private isRecording: boolean = false;
    
    constructor(mcpClient: McpClient) {
      this.mcp = mcpClient;
      // Intercept all tool calls
      this.interceptToolCalls();
    }
    
    startRecording() {
      this.isRecording = true;
      this.recording = [];
      console.log("recording started");
    }
    
    stopRecording(): WorkflowDefinition {
      this.isRecording = false;
      console.log("recording stopped");
      return {
        steps: [...this.recording],
        metadata: {
          createdAt: new Date().toISOString(),
          version: "1.0"
        }
      };
    }
    
    private interceptToolCalls() {
      // Store original callTool method
      const originalCallTool = this.mcp.callTool.bind(this.mcp);
      
      // Override with our recording version
      this.mcp.callTool = async (request) => {
        const startTime = Date.now();
        
        // Execute the actual tool call
        const result = await originalCallTool(request);
        
        // Record if we're in recording mode
        if (this.isRecording) {
          this.recording.push({
            type: 'TOOL_CALL',
            request,
            result,
            timestamp: startTime,
            duration: Date.now() - startTime
          });
        }
        
        return result;
      };
    }
    
    // Similar interceptors for other MCP operations
    // ...
  }
  // ... existing code ...
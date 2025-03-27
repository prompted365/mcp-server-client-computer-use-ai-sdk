// ... existing code ...
class WorkflowPlayer {
    private mcp: McpClient;
    
    constructor(mcpClient: McpClient) {
      this.mcp = mcpClient;
    }
    
    async playWorkflow(workflow: WorkflowDefinition, options: PlaybackOptions = {}): Promise<PlaybackResult> {
      console.log(`playing workflow: ${workflow.metadata.name || 'unnamed'}`);
      
      const results = [];
      
      for (const step of workflow.steps) {
        // Handle different step types
        if (step.type === 'TOOL_CALL') {
          // Execute the tool call
          try {
            const result = await this.mcp.callTool(step.request);
            results.push({
              step,
              result,
              success: true
            });
            
            // Optionally validate result matches recorded result
            if (options.strictValidation && !this.resultsMatch(result, step.result)) {
              throw new Error('Result mismatch detected');
            }
          } catch (error) {
            results.push({
              step,
              error,
              success: false
            });
            
            if (options.stopOnError) {
              break;
            }
          }
        } else if (step.type === 'USER_INPUT') {
          // Handle user input (might require interaction)
          if (options.interactive) {
            // Prompt for real user input
            const input = await this.promptUser(step.userInput);
            results.push({
              step,
              userInput: input,
              success: true
            });
          } else {
            // Use recorded input
            results.push({
              step,
              userInput: step.userInput,
              success: true
            });
          }
        }
        
        // Add delay between steps if specified
        if (options.stepDelay) {
          await new Promise(resolve => setTimeout(resolve, options.stepDelay));
        }
      }
      
      return {
        workflow,
        stepResults: results,
        completed: results.length === workflow.steps.length,
        success: results.every(r => r.success)
      };
    }
    
    private resultsMatch(a: any, b: any): boolean {
      // Implement comparison logic for results
      // Consider implementing fuzzy matching for certain result types
      return JSON.stringify(a) === JSON.stringify(b);
    }
    
    private async promptUser(prompt: string): Promise<string> {
      // Implement user prompting logic
      console.log(`user prompt: ${prompt}`);
      // This is a placeholder - replace with actual UI interaction
      return prompt;
    }
  }
  // ... existing code ...
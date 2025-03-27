interface WorkflowStep {
    type: 'TOOL_CALL' | 'USER_INPUT' | 'LLM_RESPONSE';
    timestamp: number;
    duration?: number;
    
    // For tool calls
    request?: any;
    result?: any;
    
    // For user inputs
    userInput?: string;
    
    // For LLM responses
    llmResponse?: string;
  }
  
  interface WorkflowDefinition {
    steps: WorkflowStep[];
    metadata: {
      name?: string;
      description?: string;
      createdAt: string;
      version: string;
      tags?: string[];
    };
  }
import { desktopClient, log } from './start-here';
import Anthropic from "@anthropic-ai/sdk";

// Initialize Anthropic client
const anthropic = new Anthropic({
  apiKey: process.env.ANTHROPIC_API_KEY,
});

// Use the correct type from Anthropic SDK
let conversationHistory: {role: "user" | "assistant"; content: any}[] = [];

export async function processUserQuery(query: string, maxTokens = 1000000, maxIterations = 100) {
  // Get available tools
  const toolsResponse = await desktopClient.listTools();
  const tools = toolsResponse.tools.map(tool => {
    // Make sure tool.parameters exists and is correctly used
    
    return {
      name: tool.name,
      description: tool.description || "",
      input_schema: {
        type: "object",
        properties: tool.parameters?.properties || {},
        required: tool.parameters?.required || []
      }
    };
  });
  
  // Add new user message with correct literal type
  conversationHistory.push({ 
    role: "user" as const, 
    content: query 
  });
  
  // Implement proper agent loop
  let isProcessing = true;
  let finalResponse = "";
  let totalTokensUsed = 0;
  let iterations = 0;
  
  log.highlight("starting agent loop with query:", query);
  
  while (isProcessing) {
    // Safety check - prevent infinite loops or excessive token usage
    iterations++;
    if (iterations > maxIterations) {
      log.warn(`reached maximum iterations (${maxIterations}), stopping loop`);
      finalResponse += "\n[maximum iterations reached. process stopped.]";
      break;  
    }
    
    if (totalTokensUsed > maxTokens) {
      log.warn(`exceeded maximum token limit (${maxTokens}), stopping loop`);
      finalResponse += "\n[maximum token limit reached. process stopped.]";
      break;
    }
    
    // Call Claude with tools and history
    const response = await anthropic.messages.create({
      model: "claude-3-7-sonnet-20250219",
      max_tokens: 1024,
      messages: conversationHistory,
      tools,
    });
    
    // Track token usage
    totalTokensUsed += response.usage.output_tokens + response.usage.input_tokens;
    log.iteration(`iteration ${iterations}: total tokens used: ${totalTokensUsed}`);
    
    // Add Claude's response to conversation history
    conversationHistory.push({
      role: "assistant" as const,
      content: response.content
    });
    
    // Check if any tool calls were made
    let hasToolCalls = false;
    let toolResultContent: Array<{
      type: string;
      tool_use_id: string;
      content: string;
      is_error?: boolean;
    }> = [];
    
    for (const content of response.content) {
      if (content.type === "text") {
        finalResponse += content.text;
      } else if (content.type === "tool_use") {
        hasToolCalls = true;
        // Extract tool call information
        const toolName = content.name;
        const toolArgs = content.input;
                
        // Execute the tool via MCP
        try {
          const result = await desktopClient.callTool(toolName, toolArgs as Record<string, any>);
          
          // Format tool result for conversation history
          // Convert object results to strings to match Anthropic's API requirements
          const resultContent = typeof result === 'object' ? 
            JSON.stringify(result) : 
            String(result);
          
          toolResultContent.push({
            type: "tool_result",
            tool_use_id: content.id,
            content: resultContent
          });
          
        } catch (error) {
          // Add error result as string
          toolResultContent.push({
            type: "tool_result",
            tool_use_id: content.id,
            content: `Error: ${error}`,
            is_error: true
          });
        }
      }
    }
    
    // If tools were used, add results to history and continue loop
    if (hasToolCalls) {
      // First, check for any previous tool results in history and replace them with minimal placeholders
      for (let i = 0; i < conversationHistory.length; i++) {
        const msg = conversationHistory[i];
        if (msg.role === "user" && Array.isArray(msg.content) && msg.content.length > 0 
            && typeof msg.content[0] === "object" && msg.content[0].type === "tool_result") {
          // Replace with minimal placeholder that preserves structure but reduces token usage
          // We must keep the tool_use_id to maintain the pairing with previous tool calls
          conversationHistory[i] = {
            role: "user" as const,
            content: msg.content.map(item => ({
              type: "tool_result",
              tool_use_id: item.tool_use_id,
              content: "[Previous tool result removed]" // Minimal placeholder
            }))
          };
          log.info("replaced previous tool result with minimal placeholder");
        }
      }
      
      // Now add the current tool results to history
      conversationHistory.push({
        role: "user" as const,
        content: toolResultContent
      });
      log.info("added new tool results to conversation history");
    } else {
      // No tools used, we're done
      isProcessing = false;
      log.success("agent loop complete, no more tool calls");
    }
  }
  
  return finalResponse;
}

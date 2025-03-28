import { desktopClient, log } from './start-here';
import Anthropic from "@anthropic-ai/sdk";
import type { Message } from "@anthropic-ai/sdk/resources/messages";

// Initialize Anthropic client
const anthropic = new Anthropic({
  apiKey: process.env.ANTHROPIC_API_KEY,
});

// Use the correct type from Anthropic SDK
let conversationHistory: {role: "user" | "assistant"; content: any}[] = [];

// Add this function to estimate tokens and trim history
function trimConversationHistory(history: typeof conversationHistory, maxTokens = 180000) {
  // Simple token estimator (very rough approximation)
  const estimateTokens = (text: string): number => Math.ceil(text.length / 4);
  
  // Estimate total tokens in history
  let totalTokens = 0;
  for (const msg of history) {
    if (typeof msg.content === 'string') {
      totalTokens += estimateTokens(msg.content);
    } else if (Array.isArray(msg.content)) {
      for (const item of msg.content) {
        if (item.type === 'text') {
          totalTokens += estimateTokens(item.text);
        } else if (item.type === 'tool_result') {
          totalTokens += estimateTokens(item.content);
        }
      }
    }
  }
  
  log.info(`estimated token count in conversation history: ${totalTokens}`);
  
  // If under the limit, return the original history
  if (totalTokens <= maxTokens) {
    return history;
  }
  
  // Need to trim - keep removing oldest messages until under limit
  log.warn(`conversation history exceeds token limit (${totalTokens}/${maxTokens}), trimming oldest messages`);
  
  const trimmedHistory = [...history];
  while (totalTokens > maxTokens && trimmedHistory.length > 2) {
    // Always keep at least the latest user query and response
    const removed = trimmedHistory.shift(); // Remove oldest message
    
    // Estimate tokens in removed message
    let removedTokens = 0;
    if (typeof removed.content === 'string') {
      removedTokens = estimateTokens(removed.content);
    } else if (Array.isArray(removed.content)) {
      for (const item of removed.content) {
        if (item.type === 'text') {
          removedTokens += estimateTokens(item.text);
        } else if (item.type === 'tool_result') {
          removedTokens += estimateTokens(item.content);
        }
      }
    }
    
    totalTokens -= removedTokens;
    log.info(`removed message with ~${removedTokens} tokens, new total: ${totalTokens}`);
  }
  
  // Add a message indicating history was trimmed
  if (trimmedHistory.length < history.length) {
    trimmedHistory.unshift({
      role: "assistant" as const,
      content: "[Some conversation history was trimmed to stay within token limits]"
    });
    log.info(`trimmed ${history.length - trimmedHistory.length} messages from history`);
  }
  
  return trimmedHistory;
}

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
    
    // Trim history before sending to Claude
    const trimmedHistory = trimConversationHistory(conversationHistory);
    
    // Call Claude with tools and trimmed history
    const response = await anthropic.messages.create({
      model: "claude-3-7-sonnet-20250219",
      max_tokens: 1024,
      messages: trimmedHistory,
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

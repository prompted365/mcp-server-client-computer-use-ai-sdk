import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import Anthropic from "@anthropic-ai/sdk";
import { CLAUDE_MODEL } from "./server.js";

// Log utility for better debugging
const log = {
  info: (msg: string, ...args: any[]) => console.error(`[info] ${msg}`, ...args),
  success: (msg: string, ...args: any[]) => console.error(`[success] ${msg}`, ...args),
  error: (msg: string, ...args: any[]) => console.error(`[error] ${msg}`, ...args),
  step: (msg: string, ...args: any[]) => console.error(`[step] ${msg}`, ...args),
};

export function registerSendDiscordMessageTool(
  server: McpServer,
  rustClient: any,
  anthropic: Anthropic
) {
  server.tool(
    "send-discord-message",
    "Send a Discord message with AI-generated content based on your prompt",
    {
      destination: z.enum(["server", "dm"]).describe("Where to send the message"),
      name: z.string().describe("Server or username"),
      prompt: z.string().describe("What to say"),
      dry_run: z.boolean().optional().describe("If true, won't actually send the message")
    },
    async ({ destination, name, prompt, dry_run = false }) => {
      const steps: string[] = [];
      steps.push(`Starting to send a ${destination === "server" ? "server" : "DM"} message to "${name}"`);
      
      try {
        log.step("opening discord application");
        steps.push("Opening Discord application");
        
        const openAppResponse = await rustClient.callTool("openApplication", {
          app_name: "Discord"
        });
        
        if (!openAppResponse || !openAppResponse.success) {
          steps.push("Failed to open Discord");
          return {
            content: [{
              type: "text",
              text: `Error: Could not open Discord. Make sure Discord is installed.`
            }],
            isError: true
          };
        }
        
        // Wait for Discord to open
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        // Get the window elements
        log.step("listing discord elements");
        steps.push("Getting Discord UI elements");
        
        // Use listInteractableElementsByIndex instead of getWindowElements
        const elementsResponse = await rustClient.callTool("listInteractableElementsByIndex", {
          app_name: "Discord",
          with_text_only: false,
          interactable_only: false,
          include_sometimes_interactable: true
        });
        
        if (!elementsResponse || !elementsResponse.elements || elementsResponse.elements.length === 0) {
          steps.push("Failed to get Discord UI elements");
          return {
            content: [{
              type: "text",
              text: `Error: Failed to get Discord UI elements.`
            }],
            isError: true
          };
        }
        
        // Log elements to help with debugging
        const elements = elementsResponse.elements;
        log.info(`found ${elements.length} elements`);
        log.info(`top 50 elements: 
${elements.slice(0, 50).map((e: any) => `[${e.index}] ${e.role}: ${e.text.substring(0, 50)}`).join('\n')}`);
        
        // Extract conversation context from Discord elements
        log.step("extracting conversation context");
        steps.push("Extracting conversation context from Discord");
        
        // Format all elements directly into conversation context
        const conversationContext = elements.map((e: any) => 
          `[${e.index}] ${e.role}: ${e.text}`
        ).join('\n');
        
        log.info(`formatted conversation context: ${conversationContext.substring(0, 200)}...`);
        steps.push("Conversation context formatted directly from elements");
        
        // Now generate message content using the conversation context
        log.info(`generating message content from prompt with conversation context`);
        steps.push("Generating message content with conversation context");
        
        // Generate message content using Claude
        const response = await anthropic.messages.create({
          model: CLAUDE_MODEL,
          max_tokens: 1000,
          messages: [
            {
              role: "user", 
              content: prompt
            }
          ],
          system: `You are generating a message that will be sent directly to Discord without modification.
Keep messages very short and concise - Discord works best with brief messages.
Use emojis appropriately to enhance your message tone and meaning.
You can use markdown formatting (bold, italic, code blocks) and mentions when needed.
Maintain a conversational, casual tone suitable for Discord.
Do not include explanations or notes - just output the exact message text to be sent.
Do not include quotes or any metadata.
The message should be ready to copy-paste into Discord.

DISCORD APP CONTEXT:
${conversationContext}

Consider this conversation context when crafting your response. Make your message relevant to the ongoing conversation, but don't explicitly reference that you've seen the context.`
        });
        
        const messageContent = response.content[0].text;
        steps.push("Message content generated successfully");
        
        if (dry_run) {
          log.success("dry run mode: would send message to Discord");
          steps.push("Dry run mode - skipping actual message sending");
          
          return {
            content: [{
              type: "text",
              text: `[DRY RUN] Would send message to ${destination} "${name}":\n\n${messageContent}`
            }]
          };
        }
        
        // Step 1: Navigate to the server or DM
        log.step("navigating to destination");
        steps.push(`Navigating to ${destination} "${name}"`);
        
        // Use LLM to help analyze the UI
        const uiAnalysisResponse = await anthropic.messages.create({
          model: CLAUDE_MODEL,
          max_tokens: 1000,
          messages: [
            {
              role: "user", 
              content: `I'm trying to send a Discord message to ${destination === "server" ? `a server channel named "${name}"` : `a user named "${name}" in my DMs`}.
              
Here are the UI elements I can see in the Discord window:

${elements.slice(0, 50).map((e: any) => `[${e.index}] ${e.role}: ${e.text.substring(0, 50)}`).join('\n')}

Please help me:
1. Find which element I should click to navigate to ${destination === "server" ? `the "${name}" channel` : `the DM with "${name}"`}
2. How should I execute this navigation?

Just respond with the element index and brief instructions, like "Click on element 12, which is the server name".`
            }
          ],
        });
        
        // Parse navigation instructions from LLM response
        const navigationInstructions = uiAnalysisResponse.content[0].text;
        steps.push(`Navigation instructions: ${navigationInstructions}`);
        
        // Extract element index from LLM response using regex
        const elementIndexMatch = navigationInstructions.match(/\[(\d+)\]|element (\d+)|index (\d+)/i);
        if (!elementIndexMatch) {
          steps.push("Failed to identify navigation element");
          return {
            content: [{
              type: "text",
              text: `Error: Could not identify the Discord element to click for navigation.`
            }],
            isError: true
          };
        }
        
        // Find the actual index in the regex match groups
        const elementIndex = parseInt(elementIndexMatch[1] || elementIndexMatch[2] || elementIndexMatch[3], 10);
        steps.push(`Using element ${elementIndex} for navigation`);
        
        // Find the element in the list
        const element = elements.find((e: any) => e.index === elementIndex);
        if (!element) {
          steps.push(`Element ${elementIndex} not found`);
          return {
            content: [{
              type: "text",
              text: `Error: Element ${elementIndex} not found in the Discord window.`
            }],
            isError: true
          };
        }
        
        // Click on the navigation element
        await rustClient.callTool("clickByIndex", {
          element_index: elementIndex
        });
        
        steps.push(`Clicked on element ${elementIndex}`);
        log.success(`navigated to ${destination} "${name}"`);
        
        // Step 6: Get updated elements after navigation
        log.step("getting updated elements");
        steps.push("Getting updated Discord UI elements after navigation");
        
        // Wait a moment for the UI to update
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        const updatedElements = await rustClient.callTool("listInteractableElementsByIndex", {
          app_name: "Discord",
          with_text_only: false,
          interactable_only: false,
          include_sometimes_interactable: true
        });
        
        if (!updatedElements || !updatedElements.elements) {
          steps.push("Failed to get updated Discord UI elements");
          return {
            content: [{
              type: "text",
              text: `Error: Failed to get updated Discord window elements.`
            }],
            isError: true
          };
        }
        
        log.info(`found ${updatedElements.elements.length} updated elements`);
        log.info(`top 50 updated elements: 
${updatedElements.elements.slice(0, 50).map((e: any) => `[${e.index}] ${e.role}: ${e.text.substring(0, 50)}`).join('\n')}`);
        
        // Step 7: Identify the text input field
        log.step("identifying text input field");
        steps.push("Identifying message input field");
        
        // Ask LLM to help find the text input
        const inputFieldResponse = await anthropic.messages.create({
          model: CLAUDE_MODEL,
          max_tokens: 100,
          messages: [
            {
              role: "user", 
              content: `Which element index is most likely the text input field for typing a message? Respond with only the index number.`
            }
          ],
          system: "You are a UI automation expert. Identify the most likely text input field index from the available elements. Only return the index number, nothing else."
        });
        
        // Parse input field index from response
        const inputIndexMatch = inputFieldResponse.content[0].text.match(/\d+/);
        if (!inputIndexMatch) {
          steps.push("Failed to identify message input field");
          return {
            content: [{
              type: "text",
              text: `Error: Could not identify Discord message input field`
            }],
            isError: true
          };
        }
        
        const inputFieldIndex = parseInt(inputIndexMatch[0], 10);
        steps.push(`Identified message input field at index ${inputFieldIndex}`);
        
        // Step 7: Type the message
        log.step("typing message");
        steps.push("Typing message content");
        
        await rustClient.callTool("typeByIndex", {
          element_index: inputFieldIndex,
          text: messageContent
        });
        
        // Final confirmation check with LLM
        const confirmResponse = await anthropic.messages.create({
          model: CLAUDE_MODEL,
          max_tokens: 100,
          messages: [
            {
              role: "user",
              content: `I've typed the message "${messageContent}" into Discord and am about to press Enter to send it to ${destination} "${name}". Is this still the right action to take? Answer with YES or NO only.`
            }
          ],
          system: "You are a careful assistant providing a final verification before sending messages."
        });
        
        const finalCheck = confirmResponse.content[0].text.trim();
        steps.push(`Final verification: ${finalCheck}`);
        
        if (finalCheck.startsWith("YES")) {
          // Send the message by pressing Enter
          await rustClient.callTool("pressKeyByIndex", {
            element_index: inputFieldIndex,
            key_combo: "Return"
          });
          
          steps.push("Message sent successfully");
          log.success("discord message sent successfully");
          
          return {
            content: [{
              type: "text",
              text: `Message successfully sent to ${destination} "${name}":\n\n"${messageContent}"\n\nProcess complete ✓`
            }]
          };
        } else {
          steps.push("Aborted sending at final verification step");
          return {
            content: [{
              type: "text",
              text: `Message was typed but not sent due to final verification check.\nContent: "${messageContent}"\n\nFinal verification indicated the message should not be sent.`
            }],
            isError: true
          };
        }
        
      } catch (error) {
        log.error("error during Discord navigation:", error);
        return {
          content: [{
            type: "text",
            text: `Error navigating Discord interface: ${error}\n\nSteps completed:\n${steps.join("\n")}`
          }],
          isError: true
        };
      }
    }
  );
  
  log.success("registered send-discord-message tool");
}

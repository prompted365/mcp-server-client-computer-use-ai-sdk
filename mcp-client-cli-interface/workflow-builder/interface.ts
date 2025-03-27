// Initialize components
const mcpClient = new McpClient(/* config */);
const llmClient = new LlmClient(/* config */);
const workflowAgent = new WorkflowAgent(mcpClient, llmClient);

// Record a new workflow
async function recordLoginWorkflow() {
  const workflow = await workflowAgent.createWorkflow(
    "Create a workflow that logs into the example.com website, checks for new notifications, and takes a screenshot of the dashboard"
  );
  
  // Save the workflow
  await workflowAgent.saveWorkflow(workflow, "daily-example-login");
  
  console.log("workflow recorded and saved!");
}

// Run an existing workflow
async function runSavedWorkflow() {
  const result = await workflowAgent.runWorkflow("daily-example-login", {
    interactive: false,
    stepDelay: 500,
    stopOnError: true
  });
  
  if (result.success) {
    console.log("workflow completed successfully");
  } else {
    console.error("workflow failed:", 
      result.stepResults.find(r => !r.success)
    );
  }
}
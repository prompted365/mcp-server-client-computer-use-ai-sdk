import { processUserQuery } from './query-processing-engine';
import readline from 'readline';
import { desktopClient } from './start-here';
import { log } from './start-here'; // Import the log utility

// Create interface for user input
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

// Start interactive session
console.log("=== desktop control chat ===");
console.log("(type \"exit\" to quit)");

function askQuestion() {
  // Styled prompt
  rl.question("\n\x1b[36mquery\x1b[0m: ", async (input) => {
    if (input.toLowerCase() === 'exit') {
      log.info("shutting down...");
      await desktopClient.disconnect();
      rl.close();
      process.exit(0);
    }
    
    try {
      log.highlight("\nprocessing...");
      const response = await processUserQuery(input);
      log.response(response);
    } catch (error) {
      log.error("error processing query:", error);
    }
    
    askQuestion();
  });
}

// Start the conversation loop
askQuestion(); 
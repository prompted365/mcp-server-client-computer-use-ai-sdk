import { desktopClient, log } from './start-here';
import { setupEnvironment } from './setup';
import { processUserQuery } from './query-processing-engine';
import readline from 'readline';
import inquirer from 'inquirer';

async function main() {
  // setup environment and check server
  await setupEnvironment();
  
  // connect to rust mcp server
  await desktopClient.connect('http://localhost:8080/mcp');
  
  // list available tools
  await desktopClient.listTools();
  
  // create readline interface
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
  });
  
  // start chat loop
  console.log('\n=== desktop control chat ===');
  console.log('(type "exit" to quit)');
  
  // show initial options
  showInitialOptions(rl);
}

// Show initial options
function showInitialOptions(rl: readline.Interface) {
  console.log("\nselect how to start:");
  
  const choices = [
    "[type your own]",
    "send message to first dialogie in messages app. message is 'i'm testing computer-use-sdk'",
    "go to discord, click 'direct messages' dialogue, then send message 'i'm testing computer-use-sdk'"
  ];
  
  inquirer.prompt([
    {
      type: 'list',
      name: 'option',
      message: 'choose an option:',
      choices: choices
    }
  ]).then(answers => {
    log.debug(`selected option: ${answers.option}`);
    
    if (answers.option === "[type your own]") {
      // Ask for custom input
      askQuestion(rl);
    } else {
      // Use the selected option directly as the prompt
      log.highlight(`using prompt: "${answers.option}"`);
      processQuery(answers.option, rl);
    }
  });
}

function processQuery(input: string, rl: readline.Interface) {
  if (input.toLowerCase() === 'exit') {
    log.info("shutting down...");
    desktopClient.disconnect()
      .then(() => {
        rl.close();
        process.exit(0);
      });
    return;
  }
  
  log.highlight("\nprocessing...");
  processUserQuery(input)
    .then(response => {
      log.response(response);
      askQuestion(rl); // Continue with normal flow
    })
    .catch(error => {
      log.error("error processing query:", error);
      askQuestion(rl); // Continue with normal flow
    });
}

function askQuestion(rl: readline.Interface) {
  inquirer.prompt([
    {
      type: 'input',
      name: 'query',
      message: 'query:',
      prefix: ''
    }
  ]).then(answers => {
    log.debug(`received input: "${answers.query}"`);
    processQuery(answers.query, rl);
  }).catch(err => {
    log.error("error getting input:", err);
    askQuestion(rl); // Try again
  });
}

main().catch(error => log.error("fatal error:", error)); 
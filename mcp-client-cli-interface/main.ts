import { desktopClient, log } from './start-here';
import { setupEnvironment } from './setup';
import { processUserQuery } from './query-processing-engine';
import readline from 'readline';
import inquirer from 'inquirer';

// Predefined prompts
const predefinedPrompts = [
  "go to discord then call listInteractableElementsByIndex, then typebyindex word test, and then call pressbyindex with return key",
  "send hello world message to partiful in messages app"
];

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
  
  inquirer.prompt([
    {
      type: 'list',
      name: 'option',
      message: 'choose an option:',
      choices: [
        { name: "1. go to discord then call listInteractableElementsByIndex, open any dm dialogue, then typebyindex word test,  and then call pressbyindex with return key", value: 1 },
        { name: "2. send hello world message to partiful in messages app", value: 2 },
        { name: "3. custom query (type your own)", value: 3 }
      ]
    }
  ]).then(answers => {
    log.debug(`selected option: ${answers.option}`);
    let input = "";
    
    switch(answers.option) {
      case 1:
        input = predefinedPrompts[0];
        log.highlight(`using predefined prompt: "${input}"`);
        processQuery(input, rl);
        break;
      case 2:
        input = predefinedPrompts[1];
        log.highlight(`using predefined prompt: "${input}"`);
        processQuery(input, rl);
        break;
      case 3:
        // Ask for custom input
        askQuestion(rl);
        break;
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
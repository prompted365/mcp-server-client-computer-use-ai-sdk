import { desktopClient } from './start-here';
import { setupEnvironment } from './setup';
import { processUserQuery } from './query-processing-engine';
import readline from 'readline';

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
  console.log('(type "exit" to quit)\n');
  
  // recursive function to keep asking questions
  const askQuestion = () => {
    rl.question('> ', async (query) => {
      if (query.toLowerCase() === 'exit') {
        await desktopClient.disconnect();
        rl.close();
        return;
      }
      
      try {
        const response = await processUserQuery(query);
        console.log('\n' + response + '\n');
      } catch (error) {
        console.error('error processing query:', error);
      }
      
      askQuestion();
    });
  };
  
  askQuestion();
}

main().catch(console.error); 
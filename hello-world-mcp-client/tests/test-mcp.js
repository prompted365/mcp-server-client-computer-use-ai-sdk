import axios from 'axios';

// Helper function to create an MCP request
function createMCPRequest(method, params) {
  return {
    jsonrpc: '2.0',
    id: Math.floor(Math.random() * 10000),
    method,
    params
  };
}

async function testMCP() {
  const mcpUrl = 'http://127.0.0.1:8080/mcp';
  console.log('starting mcp test suite');

  try {
    // 1. Initialize to get capabilities
    console.log('testing initialize...');
    const initResponse = await axios.post(mcpUrl, createMCPRequest('initialize', {
      capabilities: {
        tools: { execution: true },
        resources: {}
      }
    }));
    console.log('initialize response capabilities:', initResponse.data.result.capabilities.tools.functions.map(f => f.name));

    // 2. Open an application (e.g., browser)
    console.log('\ntesting openApplication...');
    const openAppResponse = await axios.post(mcpUrl, createMCPRequest('executeToolFunction', {
      function: 'openApplication',
      arguments: {
        app_name: 'Arc'
      }
    }));
    console.log('open application response:', openAppResponse.data);

    // Wait a moment for the app to open
    await new Promise(resolve => setTimeout(resolve, 2000));

    // 3. Open a URL
    console.log('\ntesting openUrl...');
    const openUrlResponse = await axios.post(mcpUrl, createMCPRequest('executeToolFunction', {
      function: 'openUrl',
      arguments: {
        url: 'https://example.com',
        browser: 'Arc'
      }
    }));
    console.log('open url response:', openUrlResponse.data);

    // Wait for page to load
    await new Promise(resolve => setTimeout(resolve, 3000));

    // 4. List interactable elements
    console.log('\ntesting listInteractableElements...');
    const listElementsResponse = await axios.post(mcpUrl, createMCPRequest('executeToolFunction', {
      function: 'listInteractableElements',
      arguments: {
        app_name: 'Arc',
        interactable_only: true,
        max_elements: 10
      }
    }));
    console.log('list elements response stats:', listElementsResponse.data.result.stats);
    
    if (listElementsResponse.data.result.elements.length > 0) {
      console.log('first element:', listElementsResponse.data.result.elements[0]);

      // 5. Click element by index
      console.log('\ntesting clickByIndex...');
      const clickByIndexResponse = await axios.post(mcpUrl, createMCPRequest('executeToolFunction', {
        function: 'clickByIndex',
        arguments: {
          element_index: 0
        }
      }));
      console.log('click by index response:', clickByIndexResponse.data);

      // 6. Type text by index (only if we have a text field)
      const textField = listElementsResponse.data.result.elements.findIndex(el => 
        el.role === 'AXTextField' || el.role === 'AXTextArea');
      
      if (textField >= 0) {
        console.log('\ntesting typeByIndex...');
        const typeByIndexResponse = await axios.post(mcpUrl, createMCPRequest('executeToolFunction', {
          function: 'typeByIndex',
          arguments: {
            element_index: textField,
            text: 'Hello from MCP test'
          }
        }));
        console.log('type by index response:', typeByIndexResponse.data);

        // 7. Press key by index
        console.log('\ntesting pressKeyByIndex...');
        const pressKeyByIndexResponse = await axios.post(mcpUrl, createMCPRequest('executeToolFunction', {
          function: 'pressKeyByIndex',
          arguments: {
            element_index: textField,
            key_combo: 'Enter'
          }
        }));
        console.log('press key by index response:', pressKeyByIndexResponse.data);
      } else {
        console.log('no text fields found, skipping type and press key tests');
      }
    } else {
      console.log('no elements found, skipping index-based operations');
    }

    // 8. Scroll element
    console.log('\ntesting scrollElement...');
    const scrollResponse = await axios.post(mcpUrl, createMCPRequest('executeToolFunction', {
      function: 'scrollElement',
      arguments: {
        selector: {
          app_name: 'Arc',
          locator: 'main'
        },
        direction: 'down',
        amount: 100
      }
    }));
    console.log('scroll response:', scrollResponse.data);

    // 9. Input control
    console.log('\ntesting inputControl...');
    const inputControlResponse = await axios.post(mcpUrl, createMCPRequest('executeToolFunction', {
      function: 'inputControl',
      arguments: {
        action: {
          type: 'KeyPress',
          data: 'Escape'
        }
      }
    }));
    console.log('input control response:', inputControlResponse.data);

    console.log('\nall tests completed');
  } catch (error) {
    console.error('error during testing:', error.response?.data || error.message);
  }
}

testMCP();
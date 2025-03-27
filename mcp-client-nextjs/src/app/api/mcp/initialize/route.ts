import { NextResponse } from 'next/server';
import { desktopClient } from '@/lib/mcp/start-here';
import { checkMCPServer } from '@/lib/mcp/setup';

// Shared state can be moved to a separate file if needed
let isInitialized = false;

export async function GET() {
  // Skip actual MCP connection during server startup
  if (process.env.NEXT_PHASE === 'phase-production-build') {
    return NextResponse.json({ status: 'skipped-during-build' });
  }
  
  try {
    if (isInitialized) {
      console.log('mcp client already initialized');
      return NextResponse.json({ status: 'connected' });
    }
    
    console.log('initializing mcp client connection...');
    
    // check if server is available
    const serverRunning = await checkMCPServer();
    if (!serverRunning) {
      throw new Error('mcp server is not available');
    }
    
    // connect to rust mcp server using ipv4
    await desktopClient.connect('http://127.0.0.1:8080/mcp');
    
    // list available tools
    await desktopClient.listTools();
    
    isInitialized = true;
    console.log('mcp client initialized successfully');
    
    return NextResponse.json({ 
      status: 'connected',
      message: 'mcp client initialized successfully'
    });
  } catch (error) {
    console.error('failed to initialize mcp client:', error);
    return NextResponse.json(
      { 
        status: 'error',
        error: `failed to initialize mcp client: ${error instanceof Error ? error.message : String(error)}` 
      },
      { status: 503 }
    );
  }
}

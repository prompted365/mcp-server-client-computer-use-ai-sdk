import { NextRequest, NextResponse } from 'next/server';
import { desktopClient } from '@/lib/mcp/start-here';
import { processUserQuery } from '@/lib/mcp/query-processing-engine';
import { checkMCPServer } from '@/lib/mcp/setup';

let isInitialized = false;

async function initialize() {
  if (isInitialized) return true;
  
  console.log('initializing mcp client connection...');
  
  try {
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
    return true;
  } catch (error) {
    console.error('failed to initialize mcp client:', error);
    return false;
  }
}

export async function POST(request: NextRequest) {
  try {
    const initialized = await initialize();
    if (!initialized) {
      return NextResponse.json(
        { error: 'failed to initialize mcp client' },
        { status: 503 }
      );
    }
    
    const { query } = await request.json();
    console.log('processing query:', query);
    
    if (!query) {
      return NextResponse.json(
        { error: 'query is required' },
        { status: 400 }
      );
    }
    
    const response = await processUserQuery(query);
    return NextResponse.json({ response });
  } catch (error) {
    console.error('error in mcp api route:', error);
    return NextResponse.json(
      { error: 'failed to process query' },
      { status: 500 }
    );
  }
}
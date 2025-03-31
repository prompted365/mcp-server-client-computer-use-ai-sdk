import { NextRequest, NextResponse } from 'next/server';
import { processUserQuery } from '@/lib/mcp/query-processing-engine';
import { desktopClient, log } from '@/lib/mcp/start-here';
import { checkMCPServer } from '@/lib/mcp/setup';

export async function POST(request: NextRequest) {
  try {
    // Parse the request body
    const body = await request.json();
    const { query } = body;
    
    if (!query) {
      return NextResponse.json(
        { status: 'error', error: 'query is required' },
        { status: 400 }
      );
    }
    
    log.info('received mcp query:', query);
    
    // Check if server is available
    const serverRunning = await checkMCPServer();
    if (!serverRunning) {
      throw new Error('mcp server is not available');
    }
    
    // Use the advanced query processing engine instead of direct client call
    log.highlight('processing query through agent loop');
    
    try {
      const response = await processUserQuery(query);
      return NextResponse.json({ response });
    } catch (error) {
      log.error(`failed to process query: ${error.message}`);
      
      // Return proper error response with status code
      return NextResponse.json(
        { 
          error: error.message,
          status: 'error',
          details: error.toString()
        }, 
        { status: 500 }
      );
    }
  } catch (error) {
    log.error(`error handling request: ${error}`);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}

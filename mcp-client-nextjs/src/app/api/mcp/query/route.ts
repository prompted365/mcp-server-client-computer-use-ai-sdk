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
    const response = await processUserQuery(query);
    
    log.success('query processed successfully');
    
    return NextResponse.json({
      status: 'success',
      response
    });
  } catch (error) {
    log.error('failed to process query:', error);
    return NextResponse.json(
      {
        status: 'error',
        error: `failed to process query: ${error instanceof Error ? error.message : String(error)}`
      },
      { status: 500 }
    );
  }
}

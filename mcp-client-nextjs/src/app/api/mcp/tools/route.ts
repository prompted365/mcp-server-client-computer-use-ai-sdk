import { desktopClient } from '@/lib/mcp/start-here';
import { NextResponse } from 'next/server';

export async function GET() {
  try {
    // The listTools method already exists in your desktopClient
    const toolsResponse = await desktopClient.listTools();
    
    // Format the tools into a simple array of tool names
    const toolNames = toolsResponse.tools.map((tool: any) => tool.name);
    
    console.log(`api/mcp/tools: returning ${toolNames.length} tools`);
    
    return NextResponse.json({ tools: toolNames });
  } catch (error) {
    console.error('failed to get tools:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Unknown error' },
      { status: 500 }
    );
  }
}

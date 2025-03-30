import { NextResponse } from 'next/server';
import { logBuffer } from '../../../../lib/mcp/log-buffer';

export async function GET(request: Request) {
  const url = new URL(request.url);
  const since = url.searchParams.get('since');
  
  const logs = logBuffer.getLogs(since ? parseInt(since, 10) : undefined);
  
  return NextResponse.json({ logs });
}

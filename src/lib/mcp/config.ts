import { cookies } from 'next/headers';
import { NextRequest, NextResponse } from 'next/server';
import sqlite3 from 'sqlite3';
import { open } from 'sqlite';
import path from 'path';

export const MCP_COOKIE_KEY = 'mcp_session_id';

const CloudDeploy = process.env.CLOUD_DEPLOY === 'true';
const dbPath = CloudDeploy
  ? '/data/mcp_config.db'
  : path.resolve(process.cwd(), 'mcp_config.db');


export const getMCPBaseURL = async (): Promise<string> => {
  const defaultURL = 'http://127.0.0.1:8080/mcp';
  const cookieStore = cookies();
  const sessionId = cookieStore.get(MCP_COOKIE_KEY)?.value || '';

  const db = await open({ filename: dbPath, driver: sqlite3.Database });
  const projectRow = sessionId ? await db.get(
    'SELECT url FROM project_mcp_configs WHERE session_id = ? LIMIT 1', sessionId
  ) : null;
  if (projectRow?.url) return projectRow.url;

  const globalRow = await db.get(
    'SELECT url FROM global_mcp_configs WHERE id = ? LIMIT 1', sessionId || 'default'
  );
  await db.close();

  return globalRow?.url || defaultURL;
};

export const setMCPSessionCookie = (req: NextRequest, res: NextResponse, sessionId: string): NextResponse => {
  const response = res || NextResponse.next();
  response.cookies.set(MCP_COOKIE_KEY, sessionId, {
    path: '/',
    httpOnly: false,
    maxAge: 60 * 60 * 24 * 30
  });
  return response;
};

import { NextRequest, NextResponse } from 'next/server';
import sqlite3 from 'sqlite3';
import { open } from 'sqlite';
import path from 'path';

const dbPath = path.resolve(process.cwd(), 'mcp_config.db');

export async function GET(req: NextRequest) {
  const db = await open({ filename: dbPath, driver: sqlite3.Database });
  const scope = req.nextUrl.searchParams.get('scope');
  const sessionId = req.nextUrl.searchParams.get('sessionId') ?? '';

  let result = [];
  if (scope === 'project') {
    result = await db.all('SELECT * FROM project_mcp_configs WHERE session_id = ?', sessionId);
  } else {
    result = await db.all('SELECT * FROM global_mcp_configs');
  }
  await db.close();
  return NextResponse.json(result);
}

export async function POST(req: NextRequest) {
  const body = await req.json();
  const { scope, sessionId = '', entry } = body;
  const db = await open({ filename: dbPath, driver: sqlite3.Database });

  if (scope === 'project') {
    await db.run(
      `INSERT OR REPLACE INTO project_mcp_configs (session_id, id, label, url) VALUES (?, ?, ?, ?)`,
      sessionId, entry.id, entry.label, entry.url
    );
  } else {
    await db.run(
      `INSERT OR REPLACE INTO global_mcp_configs (id, label, url) VALUES (?, ?, ?)`,
      entry.id, entry.label, entry.url
    );
  }

  await db.close();
  return NextResponse.json({ status: 'ok' });
}

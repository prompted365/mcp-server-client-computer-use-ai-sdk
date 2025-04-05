# Build Stage
FROM node:20-alpine AS builder
WORKDIR /app
COPY . .
RUN npm install
RUN npm run build

# Production Image
FROM node:20-alpine AS runner
WORKDIR /app

RUN apk add --no-cache sqlite

COPY --from=builder /app/.next .next
COPY --from=builder /app/public public
COPY --from=builder /app/package.json package.json
COPY --from=builder /app/package-lock.json package-lock.json
COPY --from=builder /app/schema.sql schema.sql

ENV NODE_ENV=production
ENV PORT=3000
ENV MCP_DB_PATH=/data/mcp_config.db
ENV CLOUD_DEPLOY=true

RUN npm ci --omit=dev

EXPOSE 3000

CMD ["npm", "start"]
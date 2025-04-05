# Build Stage
FROM node:20-alpine AS builder

WORKDIR /app
COPY . .

RUN npm install
RUN npm run build

# Production Image
FROM node:20-alpine AS runner

WORKDIR /app

# Install SQLite for runtime
RUN apk add --no-cache sqlite

# Copy only necessary files
COPY --from=builder /app/.next .next
COPY --from=builder /app/public public
COPY --from=builder /app/package.json package.json
COPY --from=builder /app/schema.sql schema.sql
COPY --from=builder /app/mcp_config.db mcp_config.db

ENV NODE_ENV=production

EXPOSE 3000

CMD ["npm", "start"]

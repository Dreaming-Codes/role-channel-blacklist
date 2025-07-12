# Docker Setup for Role Channel Blacklist Bot

This guide explains how to run the Role Channel Blacklist Discord bot using Docker and Docker Compose.

## Prerequisites

- Docker and Docker Compose installed on your system
- A Discord bot token (see [Discord Developer Portal](https://discord.com/developers/applications))

## Quick Start

1. **Clone the repository** (if you haven't already):
   ```bash
   git clone <repository-url>
   cd role-channel-blacklist
   ```

2. **Set up environment variables**:
   ```bash
   cp .env.example .env
   ```
   
   Edit the `.env` file and add your Discord bot token:
   ```
   DISCORD_TOKEN=your_discord_bot_token_here
   ```

3. **Start the services**:
   ```bash
   docker-compose up -d
   ```

   This will:
   - Build the bot Docker image
   - Start a PostgreSQL database
   - Run the bot with automatic restarts

4. **Check the logs**:
   ```bash
   # View bot logs
   docker-compose logs bot
   
   # View database logs
   docker-compose logs postgres
   
   # Follow logs in real-time
   docker-compose logs -f bot
   ```

## Service Details

### Bot Service
- **Container name**: `role_blacklist_bot`
- **Build**: Uses the local Dockerfile
- **Dependencies**: Waits for PostgreSQL to be healthy before starting
- **Restart policy**: `unless-stopped`

### Database Service
- **Container name**: `role_blacklist_db`
- **Image**: `postgres:16-alpine`
- **Port**: `5432` (exposed to host)
- **Database**: `role_blacklist`
- **User**: `blacklist_user`
- **Password**: `blacklist_pass`
- **Data persistence**: Uses Docker volume `postgres_data`

## Environment Variables

The bot requires the following environment variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `DISCORD_TOKEN` | Your Discord bot token | `MTIzNDU2Nzg5...` |
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://blacklist_user:blacklist_pass@postgres:5432/role_blacklist` |

> **Note**: The `DATABASE_URL` is automatically configured in docker-compose.yml and doesn't need to be set manually.

## Docker Commands

### Build and start services:
```bash
docker-compose up -d
```

### Stop services:
```bash
docker-compose down
```

### Rebuild the bot image:
```bash
docker-compose build bot
docker-compose up -d bot
```

### View logs:
```bash
# All services
docker-compose logs

# Specific service
docker-compose logs bot
docker-compose logs postgres

# Follow logs
docker-compose logs -f bot
```

### Access PostgreSQL database:
```bash
docker-compose exec postgres psql -U blacklist_user -d role_blacklist
```

### Restart services:
```bash
# Restart all services
docker-compose restart

# Restart specific service
docker-compose restart bot
```

## Development

For development, you can:

1. **Mount source code as volume** (add to docker-compose.yml):
   ```yaml
   volumes:
     - ./src:/app/src:ro
   ```

2. **Use cargo watch** for auto-rebuilding (modify Dockerfile CMD):
   ```dockerfile
   CMD ["cargo", "watch", "-x", "run"]
   ```

3. **Access container shell**:
   ```bash
   docker-compose exec bot /bin/bash
   ```

## Troubleshooting

### Bot won't start
1. Check if the Discord token is correct in `.env`
2. Verify database connectivity:
   ```bash
   docker-compose logs postgres
   ```

### Database connection issues
1. Ensure PostgreSQL is healthy:
   ```bash
   docker-compose ps
   ```
2. Check if the database is accessible:
   ```bash
   docker-compose exec postgres pg_isready -U blacklist_user
   ```

### Permission issues
If you encounter permission issues, ensure Docker has proper permissions and the `appuser` in the container has access to the application files.

### Clean restart
To completely reset everything:
```bash
docker-compose down -v  # This removes volumes too!
docker-compose up -d
```

> **Warning**: The `-v` flag will delete all database data!

## Security Notes

- The bot runs as a non-root user (`appuser`) inside the container
- Database credentials are configured in docker-compose.yml (consider using Docker secrets for production)
- The `.env` file should never be committed to version control
- Consider using a `.env.production` file for production deployments
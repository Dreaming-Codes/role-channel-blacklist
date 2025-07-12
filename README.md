# Role Channel Blacklist Bot

A Discord bot that allows administrators to blacklist specific roles from posting in channels, with exception roles that can bypass the blacklist.

## Features

- **Role Blacklisting**: Prevent users with specific roles from posting in channels
- **Custom Messages**: Set custom DM messages when a user's message is deleted
- **Exception Roles**: Allow certain roles to bypass blacklist restrictions
- **Per-Channel Configuration**: Different blacklist settings for each channel
- **PostgreSQL Backend**: Efficient database queries with proper indexing
- **Real-time Processing**: Fast message checking without blocking the bot

## Database Schema

The bot uses PostgreSQL with two main tables:
- `blacklist_entries`: Stores role blacklist rules per channel
- `exception_entries`: Stores exception roles that bypass blacklists

Both tables are optimized with indexes for fast lookups by channel_id and role_id.

## Setup

### Prerequisites

- Rust (latest stable)
- Docker and Docker Compose
- PostgreSQL development libraries (for compilation)

### Environment Variables

Create a `.env` file with:

```env
DATABASE_URL=postgres://blacklist_user:blacklist_pass@localhost/role_blacklist
DISCORD_TOKEN=your_discord_bot_token_here
```

## Commands

All commands require Administrator permissions.

### Blacklist Management

- `/add_role_to_blacklist <role> [custom_message]` - Add a role to the blacklist for this channel
- `/remove_role_from_blacklist <role>` - Remove a role from the blacklist for this channel
- `/list_blacklisted_roles` - Show all blacklisted roles for this channel

### Exception Management

- `/add_exception_role <role>` - Add a role that can bypass blacklists in this channel
- `/remove_exception_role <role>` - Remove an exception role for this channel

## How It Works

1. **Message Processing**: When a user sends a message, the bot checks their roles
2. **Exception Check**: If the user has any exception roles, the message is allowed
3. **Blacklist Check**: If the user has any blacklisted roles (and no exceptions), the message is deleted
4. **Custom DM**: If configured, a custom message is sent to the user via DM

## Performance Optimizations

- **Indexed Database Queries**: PostgreSQL indexes on channel_id and role_id for O(log n) lookups
- **Efficient Role Checking**: Single query checks all user roles against blacklist/exceptions
- **Connection Pooling**: BB8 connection pool for database efficiency
- **Async Processing**: Non-blocking message processing
